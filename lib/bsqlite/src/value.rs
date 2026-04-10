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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_roundtrips_for_all_supported_widths() {
        assert_eq!(i8::try_from(Value::from(-8_i8)).unwrap(), -8);
        assert_eq!(i16::try_from(Value::from(-16_i16)).unwrap(), -16);
        assert_eq!(i32::try_from(Value::from(-32_i32)).unwrap(), -32);
        assert_eq!(i64::try_from(Value::from(-64_i64)).unwrap(), -64);

        assert_eq!(
            Option::<i8>::try_from(Value::from(Some(-8_i8))).unwrap(),
            Some(-8)
        );
        assert_eq!(
            Option::<i16>::try_from(Value::from(Some(-16_i16))).unwrap(),
            Some(-16)
        );
        assert_eq!(
            Option::<i32>::try_from(Value::from(Some(-32_i32))).unwrap(),
            Some(-32)
        );
        assert_eq!(
            Option::<i64>::try_from(Value::from(Some(-64_i64))).unwrap(),
            Some(-64)
        );
        assert_eq!(Option::<i64>::try_from(Value::Null).unwrap(), None);
    }

    #[test]
    fn test_bool_float_text_and_blob_roundtrips() {
        assert!(bool::try_from(Value::from(true)).unwrap());
        assert!(!Option::<bool>::try_from(Value::from(Some(false)))
            .unwrap()
            .unwrap());
        assert_eq!(Option::<bool>::try_from(Value::Null).unwrap(), None);

        assert_eq!(f64::try_from(Value::from(1.5_f64)).unwrap(), 1.5);
        assert_eq!(
            Option::<f64>::try_from(Value::from(Some(2.5_f64))).unwrap(),
            Some(2.5)
        );
        assert_eq!(Option::<f64>::try_from(Value::Null).unwrap(), None);

        assert_eq!(
            String::try_from(Value::from("hello".to_string())).unwrap(),
            "hello"
        );
        assert_eq!(
            Option::<String>::try_from(Value::from(Some("world".to_string()))).unwrap(),
            Some("world".to_string())
        );
        assert_eq!(Option::<String>::try_from(Value::Null).unwrap(), None);

        assert_eq!(
            Vec::<u8>::try_from(Value::from(vec![1_u8, 2_u8, 3_u8])).unwrap(),
            vec![1, 2, 3]
        );
        assert_eq!(
            Option::<Vec<u8>>::try_from(Value::from(Some(vec![4_u8, 5_u8]))).unwrap(),
            Some(vec![4, 5])
        );
        assert_eq!(Option::<Vec<u8>>::try_from(Value::Null).unwrap(), None);
    }

    #[test]
    fn test_value_type_mismatch_errors() {
        assert_eq!(
            bool::try_from(Value::Text("true".to_string()))
                .unwrap_err()
                .to_string(),
            "Value error: expected integer"
        );
        assert_eq!(
            Option::<String>::try_from(Value::Integer(1))
                .unwrap_err()
                .to_string(),
            "Value error: expected text or null"
        );
        assert_eq!(
            Vec::<u8>::try_from(Value::Integer(1))
                .unwrap_err()
                .to_string(),
            "Value error: expected blob"
        );
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_uuid_value_roundtrips_and_errors() {
        use uuid::Uuid;

        let uuid = Uuid::from_bytes([
            0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4,
            0x30, 0xc8,
        ]);

        assert_eq!(Uuid::try_from(Value::from(uuid)).unwrap(), uuid);
        assert_eq!(
            Option::<Uuid>::try_from(Value::from(Some(uuid))).unwrap(),
            Some(uuid)
        );
        assert_eq!(Option::<Uuid>::try_from(Value::Null).unwrap(), None);
        assert!(Uuid::try_from(Value::Blob(vec![1_u8, 2_u8])).is_err());
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono_value_roundtrips_and_type_errors() {
        use chrono::{DateTime, NaiveDate, Utc};

        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let timestamp = 1_700_000_000_i64;
        let datetime = DateTime::<Utc>::from_timestamp_secs(timestamp).unwrap();

        assert_eq!(NaiveDate::try_from(Value::from(date)).unwrap(), date);
        assert_eq!(
            Option::<NaiveDate>::try_from(Value::from(Some(date))).unwrap(),
            Some(date)
        );
        assert_eq!(Option::<NaiveDate>::try_from(Value::Null).unwrap(), None);

        assert_eq!(
            DateTime::<Utc>::try_from(Value::from(datetime)).unwrap(),
            datetime
        );
        assert_eq!(
            Option::<DateTime<Utc>>::try_from(Value::from(Some(datetime))).unwrap(),
            Some(datetime)
        );
        assert_eq!(
            Option::<DateTime<Utc>>::try_from(Value::Null).unwrap(),
            None
        );

        assert_eq!(
            NaiveDate::try_from(Value::Text("2024-06-15".to_string()))
                .unwrap_err()
                .to_string(),
            "Value error: expected integer"
        );
        assert_eq!(
            DateTime::<Utc>::try_from(Value::Text("2024-06-15T00:00:00Z".to_string()))
                .unwrap_err()
                .to_string(),
            "Value error: expected integer"
        );
    }
}
