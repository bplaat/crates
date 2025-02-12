/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{c_char, c_void, CStr};
use std::marker::PhantomData;

use libsqlite3_sys::*;

use crate::{Bind, FromRow, Value};

// MARK: Raw Statement
/// Raw SQLite statement without type information
pub struct RawStatement(*mut sqlite3_stmt);

impl RawStatement {
    pub(crate) fn new(statement: *mut sqlite3_stmt) -> Self {
        Self(statement)
    }

    /// Reset the statement
    pub fn reset(&mut self) {
        unsafe { sqlite3_reset(self.0) };
    }

    /// Bind values to the statement
    pub fn bind(&mut self, params: impl Bind) {
        params.bind(self);
    }

    /// Bind a value to the statement
    pub fn bind_value(&mut self, index: i32, value: Value) {
        let index = index + 1;
        let result = match value {
            Value::Null => unsafe { sqlite3_bind_null(self.0, index) },
            Value::Integer(i) => unsafe { sqlite3_bind_int64(self.0, index, i) },
            Value::Real(f) => unsafe { sqlite3_bind_double(self.0, index, f) },
            Value::Text(s) => unsafe {
                sqlite3_bind_text(
                    self.0,
                    index,
                    s.as_ptr() as *const c_char,
                    s.len() as i32,
                    SQLITE_TRANSIENT(),
                )
            },
            Value::Blob(b) => unsafe {
                sqlite3_bind_blob(
                    self.0,
                    index,
                    b.as_ptr() as *const c_void,
                    b.len() as i32,
                    SQLITE_TRANSIENT(),
                )
            },
        };
        if result != SQLITE_OK {
            let query = unsafe { CStr::from_ptr(sqlite3_sql(self.0)) }.to_string_lossy();
            let error = unsafe { CStr::from_ptr(sqlite3_errmsg(sqlite3_db_handle(self.0))) }
                .to_string_lossy();
            panic!(
                "bsqlite: Can't bind value to statement!\n  Query: {}\n  Error: {}",
                query, error
            );
        }
    }

    /// Read a value from the statement
    pub fn read_value(&self, index: i32) -> Value {
        match unsafe { sqlite3_column_type(self.0, index) } {
            SQLITE_NULL => Value::Null,
            SQLITE_INTEGER => Value::Integer(unsafe { sqlite3_column_int64(self.0, index) }),
            SQLITE_FLOAT => Value::Real(unsafe { sqlite3_column_double(self.0, index) }),
            SQLITE_TEXT => {
                let text = unsafe { sqlite3_column_text(self.0, index) };
                let text = unsafe { CStr::from_ptr(text as *const c_char) }
                    .to_string_lossy()
                    .to_string();
                Value::Text(text)
            }
            SQLITE_BLOB => {
                let blob = unsafe { sqlite3_column_blob(self.0, index) };
                let len = unsafe { sqlite3_column_bytes(self.0, index) };
                let slice = unsafe { std::slice::from_raw_parts(blob as *const u8, len as usize) };
                Value::Blob(slice.to_vec())
            }
            r#type => unreachable!("Unknown column type: {}", r#type),
        }
    }
}

impl Drop for RawStatement {
    fn drop(&mut self) {
        unsafe { sqlite3_finalize(self.0) };
    }
}

// MARK: Statement
/// A SQLite statement with type information
pub struct Statement<T>(RawStatement, PhantomData<T>);

impl<T> Statement<T> {
    pub(crate) fn new(statement: *mut sqlite3_stmt) -> Self {
        Self(RawStatement::new(statement), PhantomData)
    }

    /// Reset the statement
    pub fn reset(&mut self) {
        self.0.reset();
    }

    /// Bind values to the statement
    pub fn bind(&mut self, params: impl Bind) {
        self.0.bind(params);
    }

    /// Bind a value to the statement
    pub fn bind_value(&mut self, index: i32, value: impl Into<Value>) {
        self.0.bind_value(index, value.into());
    }

    /// Read a value from the statement
    pub fn read_value(&self, index: i32) -> Value {
        self.0.read_value(index)
    }
}

impl<T> Iterator for Statement<T>
where
    T: FromRow,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let result = unsafe { sqlite3_step(self.0 .0) };
        if result == SQLITE_ROW {
            Some(T::from_row(&mut self.0))
        } else if result == SQLITE_DONE {
            None
        } else {
            let query = unsafe { CStr::from_ptr(sqlite3_sql(self.0 .0)) }.to_string_lossy();
            let error = unsafe { CStr::from_ptr(sqlite3_errmsg(sqlite3_db_handle(self.0 .0))) }
                .to_string_lossy();
            panic!(
                "bsqlite: Can't step statement!\n  Query: {}\n  Error: {}",
                query, error
            );
        }
    }
}
