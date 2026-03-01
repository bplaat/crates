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
    Float(f64),
    /// A text value
    Text(String),
    /// A blob value
    Blob(Vec<u8>),
}

// MARK: ValueError
type Result<T> = std::result::Result<T, ValueError>;

/// A value error
#[derive(Debug)]
pub struct ValueError {
    msg: String,
}

impl ValueError {
    #[doc(hidden)]
    pub fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

impl Display for ValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Value error: {}", self.msg)
    }
}

impl Error for ValueError {}

// MARK: From T
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Integer(if value { 1 } else { 0 })
    }
}
impl TryFrom<Value> for bool {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(v) => Ok(v != 0),
            _ => Err(ValueError {
                msg: "expected integer".to_string(),
            }),
        }
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Value::Integer(value as i64)
    }
}
impl TryFrom<Value> for i8 {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(v) => Ok(v as i8),
            _ => Err(ValueError {
                msg: "expected integer".to_string(),
            }),
        }
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::Integer(value as i64)
    }
}
impl TryFrom<Value> for i16 {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(v) => Ok(v as i16),
            _ => Err(ValueError {
                msg: "expected integer".to_string(),
            }),
        }
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Integer(value as i64)
    }
}
impl TryFrom<Value> for i32 {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(v) => Ok(v as i32),
            _ => Err(ValueError {
                msg: "expected integer".to_string(),
            }),
        }
    }
}

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
            _ => Err(ValueError {
                msg: "expected integer".to_string(),
            }),
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}
impl TryFrom<Value> for f64 {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Float(v) => Ok(v),
            _ => Err(ValueError {
                msg: "expected float".to_string(),
            }),
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
            _ => Err(ValueError {
                msg: "expected text".to_string(),
            }),
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
            _ => Err(ValueError {
                msg: "expected blob".to_string(),
            }),
        }
    }
}

// MARK: From Option<T>
impl From<Option<bool>> for Value {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(v) => Value::Integer(if v { 1 } else { 0 }),
            None => Value::Null,
        }
    }
}
impl TryFrom<Value> for Option<bool> {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(v) => Ok(Some(v != 0)),
            Value::Null => Ok(None),
            _ => Err(ValueError {
                msg: "expected integer or null".to_string(),
            }),
        }
    }
}

impl From<Option<i8>> for Value {
    fn from(value: Option<i8>) -> Self {
        match value {
            Some(v) => Value::Integer(v as i64),
            None => Value::Null,
        }
    }
}
impl TryFrom<Value> for Option<i8> {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(v) => Ok(Some(v as i8)),
            Value::Null => Ok(None),
            _ => Err(ValueError {
                msg: "expected integer or null".to_string(),
            }),
        }
    }
}

impl From<Option<i16>> for Value {
    fn from(value: Option<i16>) -> Self {
        match value {
            Some(v) => Value::Integer(v as i64),
            None => Value::Null,
        }
    }
}
impl TryFrom<Value> for Option<i16> {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(v) => Ok(Some(v as i16)),
            Value::Null => Ok(None),
            _ => Err(ValueError {
                msg: "expected integer or null".to_string(),
            }),
        }
    }
}

impl From<Option<i32>> for Value {
    fn from(value: Option<i32>) -> Self {
        match value {
            Some(v) => Value::Integer(v as i64),
            None => Value::Null,
        }
    }
}
impl TryFrom<Value> for Option<i32> {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(v) => Ok(Some(v as i32)),
            Value::Null => Ok(None),
            _ => Err(ValueError {
                msg: "expected integer or null".to_string(),
            }),
        }
    }
}

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
            _ => Err(ValueError {
                msg: "expected integer or null".to_string(),
            }),
        }
    }
}

impl From<Option<f64>> for Value {
    fn from(value: Option<f64>) -> Self {
        match value {
            Some(v) => Value::Float(v),
            None => Value::Null,
        }
    }
}
impl TryFrom<Value> for Option<f64> {
    type Error = ValueError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Float(v) => Ok(Some(v)),
            Value::Null => Ok(None),
            _ => Err(ValueError {
                msg: "expected float or null".to_string(),
            }),
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
            _ => Err(ValueError {
                msg: "expected text or null".to_string(),
            }),
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
            _ => Err(ValueError {
                msg: "expected blob or null".to_string(),
            }),
        }
    }
}

// MARK: Uuid
#[cfg(feature = "uuid")]
mod uuid_impls {
    use uuid::Uuid;

    use super::*;

    // MARK: From Uuid
    impl From<Uuid> for Value {
        fn from(value: Uuid) -> Self {
            Value::Blob(value.into_bytes().to_vec())
        }
    }
    impl TryFrom<Value> for Uuid {
        type Error = ValueError;
        fn try_from(value: Value) -> Result<Self> {
            match value {
                Value::Blob(v) => {
                    Ok(Uuid::from_slice(&v).map_err(|e| ValueError { msg: e.to_string() })?)
                }
                _ => Err(ValueError {
                    msg: "expected blob".to_string(),
                }),
            }
        }
    }

    impl From<Option<Uuid>> for Value {
        fn from(value: Option<Uuid>) -> Self {
            match value {
                Some(v) => Value::Blob(v.into_bytes().to_vec()),
                None => Value::Null,
            }
        }
    }
    impl TryFrom<Value> for Option<Uuid> {
        type Error = ValueError;
        fn try_from(value: Value) -> Result<Self> {
            match value {
                Value::Blob(v) => Ok(Some(
                    Uuid::from_slice(&v).map_err(|e| ValueError { msg: e.to_string() })?,
                )),
                Value::Null => Ok(None),
                _ => Err(ValueError {
                    msg: "expected blob or null".to_string(),
                }),
            }
        }
    }
}

// MARK: Chrono
#[cfg(feature = "chrono")]
mod chrono_impls {
    use chrono::{DateTime, NaiveDate, Utc};

    use super::*;

    // MARK: From NaiveDate
    impl From<NaiveDate> for Value {
        fn from(value: NaiveDate) -> Self {
            Value::Integer(
                value
                    .and_hms_opt(0, 0, 0)
                    .expect("Should be some")
                    .and_utc()
                    .timestamp(),
            )
        }
    }
    impl TryFrom<Value> for NaiveDate {
        type Error = ValueError;
        fn try_from(value: Value) -> Result<Self> {
            match value {
                Value::Integer(i) => Ok(DateTime::<Utc>::from_timestamp_secs(i)
                    .ok_or_else(|| ValueError {
                        msg: format!("invalid timestamp: {i}"),
                    })?
                    .naive_utc()
                    .date()),
                _ => Err(ValueError {
                    msg: "expected integer".to_string(),
                }),
            }
        }
    }

    impl From<Option<NaiveDate>> for Value {
        fn from(value: Option<NaiveDate>) -> Self {
            match value {
                Some(v) => Value::Integer(
                    v.and_hms_opt(0, 0, 0)
                        .expect("Should be some")
                        .and_utc()
                        .timestamp(),
                ),
                None => Value::Null,
            }
        }
    }
    impl TryFrom<Value> for Option<NaiveDate> {
        type Error = ValueError;
        fn try_from(value: Value) -> Result<Self> {
            match value {
                Value::Integer(i) => Ok(Some(
                    DateTime::<Utc>::from_timestamp_secs(i)
                        .ok_or_else(|| ValueError {
                            msg: format!("invalid timestamp: {i}"),
                        })?
                        .naive_utc()
                        .date(),
                )),
                Value::Null => Ok(None),
                _ => Err(ValueError {
                    msg: "expected integer or null".to_string(),
                }),
            }
        }
    }

    // MARK: From DateTime<Utc>
    impl From<DateTime<Utc>> for Value {
        fn from(value: DateTime<Utc>) -> Self {
            Value::Integer(value.timestamp())
        }
    }
    impl TryFrom<Value> for DateTime<Utc> {
        type Error = ValueError;
        fn try_from(value: Value) -> Result<Self> {
            match value {
                Value::Integer(i) => {
                    Ok(Self::from_timestamp_secs(i).ok_or_else(|| ValueError {
                        msg: format!("invalid timestamp: {i}"),
                    })?)
                }
                _ => Err(ValueError {
                    msg: "expected integer".to_string(),
                }),
            }
        }
    }

    impl From<Option<DateTime<Utc>>> for Value {
        fn from(value: Option<DateTime<Utc>>) -> Self {
            match value {
                Some(v) => Value::Integer(v.timestamp()),
                None => Value::Null,
            }
        }
    }
    impl TryFrom<Value> for Option<DateTime<Utc>> {
        type Error = ValueError;
        fn try_from(value: Value) -> Result<Self> {
            match value {
                Value::Integer(i) => Ok(DateTime::<Utc>::from_timestamp_secs(i)),
                Value::Null => Ok(None),
                _ => Err(ValueError {
                    msg: "expected integer or null".to_string(),
                }),
            }
        }
    }
}
