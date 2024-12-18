/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{c_char, c_void, CStr};
use std::marker::PhantomData;

use crate::sys::*;
use crate::value::Value;

// MARK: Bind
/// A trait for binding values to a statement
pub trait Bind {
    /// Bind values to a statement
    fn bind(self, statement: &mut RawStatement);
}

impl Bind for () {
    fn bind(self, _statement: &mut RawStatement) {}
}

impl<T> Bind for T
where
    T: Into<Value>,
{
    fn bind(self, statement: &mut RawStatement) {
        statement.bind_value(self, 0);
    }
}

macro_rules! impl_bind_for_tuple {
    ($($n:tt: $t:ident),*) => (
        impl<$($t,)*> Bind for ($($t,)*)
        where
            $($t: Into<Value>,)+
        {
            fn bind(self, statement: &mut RawStatement)  {
                $( statement.bind_value(self.$n, $n); )*
            }
        }
    );
}
impl_bind_for_tuple!(0: A);
impl_bind_for_tuple!(0: A, 1: B);
impl_bind_for_tuple!(0: A, 1: B, 2: C);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
impl_bind_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);

// MARK: FromRow
/// A trait for converting read values from a statement to a row
pub trait FromRow: Sized {
    /// Convert read values from a statement to a row
    fn from_row(statement: &mut RawStatement) -> Self;
}

impl FromRow for () {
    fn from_row(_statement: &mut RawStatement) -> Self {}
}

impl<T> FromRow for T
where
    T: TryFrom<Value>,
{
    fn from_row(statement: &mut RawStatement) -> Self {
        match T::try_from(statement.read_value(0)) {
            Ok(value) => value,
            Err(_) => panic!("Can't convert Value"),
        }
    }
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

    /// Bind a value to the statement
    pub fn bind_value(&mut self, value: impl Into<Value>, index: i32) {
        let index = index + 1;
        let result = match value.into() {
            Value::Null => unsafe { sqlite3_bind_null(self.0, index) },
            Value::Integer(i) => unsafe { sqlite3_bind_int64(self.0, index, i) },
            Value::Real(f) => unsafe { sqlite3_bind_double(self.0, index, f) },
            Value::Text(s) => unsafe {
                sqlite3_bind_text(
                    self.0,
                    index,
                    s.as_ptr() as *const c_char,
                    s.as_bytes().len() as i32,
                    SQLITE_TRANSIENT,
                )
            },
            Value::Blob(b) => unsafe {
                sqlite3_bind_blob(
                    self.0,
                    index,
                    b.as_ptr() as *const c_void,
                    b.len() as i32,
                    SQLITE_TRANSIENT,
                )
            },
        };
        if result != SQLITE_OK {
            let error = unsafe { CStr::from_ptr(sqlite3_errmsg(self.0)) }.to_string_lossy();
            panic!("Can't bind value to statement: {}", error);
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
                let text: String = unsafe { CStr::from_ptr(text as *const c_char) }
                    .to_string_lossy()
                    .into_owned();
                Value::Text(text)
            }
            SQLITE_BLOB => {
                let blob = unsafe { sqlite3_column_blob(self.0, index) };
                let len = unsafe { sqlite3_column_bytes(self.0, index) };
                let slice = unsafe { std::slice::from_raw_parts(blob as *const u8, len as usize) };
                Value::Blob(slice.to_vec())
            }
            _ => panic!("Can't read unknown value type from statement"),
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
    pub fn bind_value(&mut self, value: impl Into<Value>, index: i32) {
        self.0.bind_value(value, index);
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
            panic!("Can't step statement")
        }
    }
}
