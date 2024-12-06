/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{c_char, c_void, CStr};

use crate::error::{Error, Result};
use crate::sys::*;

#[derive(Debug)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl Value {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn bind_to_statement(&self, statement: *mut sqlite3_stmt, index: i32) -> Result<()> {
        let result = match self {
            Value::Null => unsafe { sqlite3_bind_null(statement, index) },
            Value::Integer(i) => unsafe { sqlite3_bind_int64(statement, index, *i) },
            Value::Real(f) => unsafe { sqlite3_bind_double(statement, index, *f) },
            Value::Text(s) => unsafe {
                sqlite3_bind_text(
                    statement,
                    index,
                    s.as_ptr() as *const c_char,
                    s.as_bytes().len() as i32,
                    SQLITE_TRANSIENT,
                )
            },
            Value::Blob(b) => unsafe {
                sqlite3_bind_blob(
                    statement,
                    index,
                    b.as_ptr() as *const c_void,
                    b.len() as i32,
                    SQLITE_TRANSIENT,
                )
            },
        };
        if result != SQLITE_OK {
            return Err(Error::new("Can't bind value"));
        }
        Ok(())
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn read_from_statement(statement: *mut sqlite3_stmt, index: i32) -> Result<Self> {
        match unsafe { sqlite3_column_type(statement, index) } {
            SQLITE_NULL => Ok(Value::Null),
            SQLITE_INTEGER => Ok(Value::Integer(unsafe {
                sqlite3_column_int64(statement, index)
            })),
            SQLITE_FLOAT => Ok(Value::Real(unsafe {
                sqlite3_column_double(statement, index)
            })),
            SQLITE_TEXT => {
                let text = unsafe { sqlite3_column_text(statement, index) };
                let text: String = unsafe { CStr::from_ptr(text as *const c_char) }
                    .to_string_lossy()
                    .into_owned();
                Ok(Value::Text(text))
            }
            SQLITE_BLOB => {
                let blob = unsafe { sqlite3_column_blob(statement, index) };
                let len = unsafe { sqlite3_column_bytes(statement, index) };
                let slice = unsafe { std::slice::from_raw_parts(blob as *const u8, len as usize) };
                Ok(Value::Blob(slice.to_vec()))
            }
            _ => Err(Error::new("Can't read value")),
        }
    }
}

// MARK: From T
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}
impl TryFrom<Value> for i64 {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(v) => Ok(v),
            _ => Err(Error::new("Value is not an integer")),
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Real(value)
    }
}
impl TryFrom<Value> for f64 {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Real(v) => Ok(v),
            _ => Err(Error::new("Value is not a real")),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value)
    }
}
impl TryFrom<Value> for String {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Text(v) => Ok(v),
            _ => Err(Error::new("Value is not a text")),
        }
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Blob(value)
    }
}
impl TryFrom<Value> for Vec<u8> {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Blob(v) => Ok(v),
            _ => Err(Error::new("Value is not a blob")),
        }
    }
}

// MARK: From Uuid
#[cfg(feature = "uuid")]
impl From<uuid::Uuid> for Value {
    fn from(value: uuid::Uuid) -> Self {
        Value::Blob(value.into_bytes().to_vec())
    }
}
#[cfg(feature = "uuid")]
impl TryFrom<Value> for uuid::Uuid {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Blob(v) => {
                Ok(uuid::Uuid::from_slice(&v).map_err(|_| Error::new("Can't convert to Uuid"))?)
            }
            _ => Err(Error::new("Value is not a blob")),
        }
    }
}

// MARK: From DateTime
#[cfg(feature = "chrono")]
impl From<chrono::DateTime<chrono::Utc>> for Value {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Value::Text(value.to_rfc3339())
    }
}
#[cfg(feature = "chrono")]
impl TryFrom<Value> for chrono::DateTime<chrono::Utc> {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Text(v) => Ok(chrono::DateTime::parse_from_rfc3339(&v)
                .map_err(|_| Error::new("Can't convert to DateTime"))?
                .with_timezone(&chrono::Utc)),
            _ => Err(Error::new("Value is not a text")),
        }
    }
}

// MARK: From Option<T>
impl From<Option<i64>> for Value {
    fn from(value: Option<i64>) -> Self {
        match value {
            Some(v) => Value::Integer(v),
            None => Value::Null,
        }
    }
}
impl TryFrom<Value> for Option<i64> {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(v) => Ok(Some(v)),
            Value::Null => Ok(None),
            _ => Err(Error::new("Value is not an integer or null")),
        }
    }
}

impl From<Option<f64>> for Value {
    fn from(value: Option<f64>) -> Self {
        match value {
            Some(v) => Value::Real(v),
            None => Value::Null,
        }
    }
}
impl TryFrom<Value> for Option<f64> {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Real(v) => Ok(Some(v)),
            Value::Null => Ok(None),
            _ => Err(Error::new("Value is not a real or null")),
        }
    }
}

impl From<Option<String>> for Value {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(v) => Value::Text(v),
            None => Value::Null,
        }
    }
}
impl TryFrom<Value> for Option<String> {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Text(v) => Ok(Some(v)),
            Value::Null => Ok(None),
            _ => Err(Error::new("Value is not a text or null")),
        }
    }
}

impl From<Option<Vec<u8>>> for Value {
    fn from(value: Option<Vec<u8>>) -> Self {
        match value {
            Some(v) => Value::Blob(v),
            None => Value::Null,
        }
    }
}
impl TryFrom<Value> for Option<Vec<u8>> {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Blob(v) => Ok(Some(v)),
            Value::Null => Ok(None),
            _ => Err(Error::new("Value is not a blob or null")),
        }
    }
}

// MARK: From Option<Uuid>
#[cfg(feature = "uuid")]
impl From<Option<uuid::Uuid>> for Value {
    fn from(value: Option<uuid::Uuid>) -> Self {
        match value {
            Some(v) => Value::Blob(v.into_bytes().to_vec()),
            None => Value::Null,
        }
    }
}
#[cfg(feature = "uuid")]
impl TryFrom<Value> for Option<uuid::Uuid> {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Blob(v) => Ok(Some(
                uuid::Uuid::from_slice(&v).map_err(|_| Error::new("Can't convert to Uuid"))?,
            )),
            Value::Null => Ok(None),
            _ => Err(Error::new("Value is not a blob or null")),
        }
    }
}

// MARK: From Option<DateTime>
#[cfg(feature = "chrono")]
impl From<Option<chrono::DateTime<chrono::Utc>>> for Value {
    fn from(value: Option<chrono::DateTime<chrono::Utc>>) -> Self {
        match value {
            Some(v) => Value::Text(v.to_rfc3339()),
            None => Value::Null,
        }
    }
}
#[cfg(feature = "chrono")]
impl TryFrom<Value> for Option<chrono::DateTime<chrono::Utc>> {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Text(v) => Ok(Some(
                chrono::DateTime::parse_from_rfc3339(&v)
                    .map_err(|_| Error::new("Can't convert to DateTime"))?
                    .with_timezone(&chrono::Utc),
            )),
            Value::Null => Ok(None),
            _ => Err(Error::new("Value is not a text or null")),
        }
    }
}
