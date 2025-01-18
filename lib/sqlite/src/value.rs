/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter};

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

/// A value error
#[derive(Debug)]
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

// MARK: From time
#[cfg(feature = "time")]
impl From<time::Date> for Value {
    fn from(value: time::Date) -> Self {
        Value::Integer(value.timestamp())
    }
}
#[cfg(feature = "time")]
impl TryFrom<Value> for time::Date {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(i) => Ok(time::Date::from_timestamp(i)),
            _ => Err(ValueError),
        }
    }
}

#[cfg(feature = "time")]
impl From<time::DateTime> for Value {
    fn from(value: time::DateTime) -> Self {
        Value::Integer(value.timestamp())
    }
}
#[cfg(feature = "time")]
impl TryFrom<Value> for time::DateTime {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(i) => Ok(time::DateTime::from_timestamp(i)),
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

// MARK: From time
#[cfg(feature = "time")]
impl From<Option<time::Date>> for Value {
    fn from(value: Option<time::Date>) -> Self {
        match value {
            Some(v) => Value::Integer(v.timestamp()),
            None => Value::Null,
        }
    }
}
#[cfg(feature = "time")]
impl TryFrom<Value> for Option<time::Date> {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            #[allow(deprecated)]
            Value::Integer(i) => Ok(Some(time::Date::from_timestamp(i))),
            Value::Null => Ok(None),
            _ => Err(ValueError),
        }
    }
}

#[cfg(feature = "time")]
impl From<Option<time::DateTime>> for Value {
    fn from(value: Option<time::DateTime>) -> Self {
        match value {
            Some(v) => Value::Integer(v.timestamp()),
            None => Value::Null,
        }
    }
}
#[cfg(feature = "time")]
impl TryFrom<Value> for Option<time::DateTime> {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(i) => Ok(Some(time::DateTime::from_timestamp(i))),
            Value::Null => Ok(None),
            _ => Err(ValueError),
        }
    }
}
