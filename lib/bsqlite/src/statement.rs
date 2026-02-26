/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{c_char, c_void, CStr, CString};
use std::marker::PhantomData;

use libsqlite3_sys::*;

use crate::{Bind, FromRow, Value};

// MARK: Column Type
/// Column type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    /// Null type
    Null,
    /// Integer type
    Integer,
    /// Float type
    Float,
    /// Text type
    Text,
    /// Blob type
    Blob,
}

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

    /// Bind value to the statement
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
            panic!("bsqlite: Can't bind value to statement!\n  Query: {query}\n  Error: {error}");
        }
    }

    /// Bind named value to the statement
    pub fn bind_named_value(&mut self, name: &str, value: Value) {
        let c_name = CString::new(name).expect("Can't convert to CString");
        let index = unsafe { sqlite3_bind_parameter_index(self.0, c_name.as_ptr()) };
        if index == 0 {
            panic!("bsqlite: Can't find named parameter: {name}");
        }
        self.bind_value(index - 1, value);
    }

    /// Step the statement
    pub fn step(&mut self) -> Option<()> {
        let result = unsafe { sqlite3_step(self.0) };
        if result == SQLITE_ROW {
            Some(())
        } else if result == SQLITE_DONE {
            None
        } else {
            let query = unsafe { CStr::from_ptr(sqlite3_sql(self.0)) }.to_string_lossy();
            let error = unsafe { CStr::from_ptr(sqlite3_errmsg(sqlite3_db_handle(self.0))) }
                .to_string_lossy();
            panic!("bsqlite: Can't step statement!\n  Query: {query}\n  Error: {error}");
        }
    }

    /// Get the number of columns in the statement
    pub fn column_count(&self) -> i32 {
        unsafe { sqlite3_column_count(self.0) }
    }

    /// Get the name of a column
    pub fn column_name(&self, index: i32) -> String {
        let name = unsafe { sqlite3_column_name(self.0, index) };
        unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .to_string()
    }

    /// Get the type of a column
    pub fn column_type(&self, index: i32) -> ColumnType {
        match unsafe { sqlite3_column_type(self.0, index) } {
            SQLITE_NULL => ColumnType::Null,
            SQLITE_INTEGER => ColumnType::Integer,
            SQLITE_FLOAT => ColumnType::Float,
            SQLITE_TEXT => ColumnType::Text,
            SQLITE_BLOB => ColumnType::Blob,
            r#type => unreachable!("Unknown column type: {}", r#type),
        }
    }

    /// Get the declared type of a column
    pub fn column_declared_type(&self, index: i32) -> Option<String> {
        let decl_type = unsafe { sqlite3_column_decltype(self.0, index) };
        if !decl_type.is_null() {
            Some(
                unsafe { CStr::from_ptr(decl_type) }
                    .to_string_lossy()
                    .to_string(),
            )
        } else {
            None
        }
    }

    /// Get the value of a column
    pub fn column_value(&self, index: i32) -> Value {
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

    /// Bind value to the statement
    pub fn bind_value(&mut self, index: i32, value: impl Into<Value>) {
        self.0.bind_value(index, value.into());
    }

    /// Bind named value to the statement
    pub fn bind_named_value(&mut self, name: &str, value: impl Into<Value>) {
        self.0.bind_named_value(name, value.into());
    }

    /// Step the statement
    pub fn step(&mut self) -> Option<()> {
        self.0.step()
    }

    /// Get the number of columns in the statement
    pub fn column_count(&self) -> i32 {
        self.0.column_count()
    }

    /// Get the name of a column
    pub fn column_name(&self, index: i32) -> String {
        self.0.column_name(index)
    }

    /// Get the type of a column
    pub fn column_type(&self, index: i32) -> ColumnType {
        self.0.column_type(index)
    }

    /// Get the declared type of a column
    pub fn column_declared_type(&self, index: i32) -> Option<String> {
        self.0.column_declared_type(index)
    }

    /// Get the value of a column
    pub fn column_value(&self, index: i32) -> Value {
        self.0.column_value(index)
    }
}

impl<T: FromRow> Iterator for Statement<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(()) = self.step() {
            Some(T::from_row(&mut self.0))
        } else {
            None
        }
    }
}
