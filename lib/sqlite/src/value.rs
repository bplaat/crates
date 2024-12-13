/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
/// A SQLite value
pub enum Value {
    /// A NULL value
    Null,
    /// An 64-bit integer value
    Integer(i64),
    /// A 64-bit floating point value
    Real(f64),
    /// A text value
    Text(String),
    /// A blob value
    Blob(Vec<u8>),
}

// MARK: ValueError
type Result<T> = std::result::Result<T, ValueError>;

#[derive(Debug)]
/// A value error
pub struct ValueError;

impl Display for ValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Value error")
    }
}

impl Error for ValueError {}

// MARK: From T
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}
impl TryFrom<Value> for i64 {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(v) => Ok(v),
            _ => Err(ValueError),
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Real(value)
    }
}
impl TryFrom<Value> for f64 {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Real(v) => Ok(v),
            _ => Err(ValueError),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value)
    }
}
impl TryFrom<Value> for String {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Text(v) => Ok(v),
            _ => Err(ValueError),
        }
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Blob(value)
    }
}
impl TryFrom<Value> for Vec<u8> {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Blob(v) => Ok(v),
            _ => Err(ValueError),
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
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Blob(v) => Ok(uuid::Uuid::from_slice(&v).map_err(|_| ValueError)?),
            _ => Err(ValueError),
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
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Text(v) => Ok(chrono::DateTime::parse_from_rfc3339(&v)
                .map_err(|_| ValueError)?
                .with_timezone(&chrono::Utc)),
            _ => Err(ValueError),
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
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(v) => Ok(Some(v)),
            Value::Null => Ok(None),
            _ => Err(ValueError),
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
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Real(v) => Ok(Some(v)),
            Value::Null => Ok(None),
            _ => Err(ValueError),
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
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Text(v) => Ok(Some(v)),
            Value::Null => Ok(None),
            _ => Err(ValueError),
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
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Blob(v) => Ok(Some(v)),
            Value::Null => Ok(None),
            _ => Err(ValueError),
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
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Blob(v) => Ok(Some(uuid::Uuid::from_slice(&v).map_err(|_| ValueError)?)),
            Value::Null => Ok(None),
            _ => Err(ValueError),
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
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Text(v) => Ok(Some(
                chrono::DateTime::parse_from_rfc3339(&v)
                    .map_err(|_| ValueError)?
                    .with_timezone(&chrono::Utc),
            )),
            Value::Null => Ok(None),
            _ => Err(ValueError),
        }
    }
}
