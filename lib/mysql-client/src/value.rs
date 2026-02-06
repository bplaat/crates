/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::error::{Error, Result};

/// MySQL value representation.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// NULL value.
    Null,
    /// Integer value.
    Int(i64),
    /// Unsigned integer value.
    UInt(u64),
    /// Floating point value.
    Float(f64),
    /// String/text value.
    Bytes(Vec<u8>),
}

impl Value {
    /// Convert value to bytes, consuming it.
    pub fn as_bytes(self) -> Result<Vec<u8>> {
        match self {
            Value::Bytes(b) => Ok(b),
            Value::Null => Err(Error::InvalidParameter(
                "NULL value cannot be converted to bytes".into(),
            )),
            _ => Err(Error::InvalidParameter("Value type mismatch".into())),
        }
    }

    /// Borrow value as bytes.
    pub fn as_bytes_ref(&self) -> Result<&[u8]> {
        match self {
            Value::Bytes(b) => Ok(b),
            Value::Null => Err(Error::InvalidParameter(
                "NULL value cannot be converted to bytes".into(),
            )),
            _ => Err(Error::InvalidParameter("Value type mismatch".into())),
        }
    }

    /// Convert to string, consuming the value.
    pub fn as_string(self) -> Result<String> {
        match self {
            Value::Bytes(b) => String::from_utf8(b)
                .map_err(|_| Error::InvalidParameter("Invalid UTF-8 in string value".into())),
            Value::Null => Err(Error::InvalidParameter(
                "NULL value cannot be converted to string".into(),
            )),
            Value::Int(i) => Ok(i.to_string()),
            Value::UInt(u) => Ok(u.to_string()),
            Value::Float(f) => Ok(f.to_string()),
        }
    }

    /// Convert to i64, consuming the value.
    pub fn as_int(self) -> Result<i64> {
        match self {
            Value::Int(i) => Ok(i),
            Value::UInt(u) => i64::try_from(u)
                .map_err(|_| Error::InvalidParameter("Unsigned value too large for i64".into())),
            Value::Bytes(b) => {
                let s = String::from_utf8(b)
                    .map_err(|_| Error::InvalidParameter("Invalid UTF-8".into()))?;
                s.parse()
                    .map_err(|_| Error::InvalidParameter("Cannot parse as integer".into()))
            }
            _ => Err(Error::InvalidParameter(
                "Value cannot be converted to i64".into(),
            )),
        }
    }

    /// Convert to u64, consuming the value.
    pub fn as_uint(self) -> Result<u64> {
        match self {
            Value::UInt(u) => Ok(u),
            Value::Int(i) => u64::try_from(i).map_err(|_| {
                Error::InvalidParameter("Negative value cannot be converted to u64".into())
            }),
            Value::Bytes(b) => {
                let s = String::from_utf8(b)
                    .map_err(|_| Error::InvalidParameter("Invalid UTF-8".into()))?;
                s.parse()
                    .map_err(|_| Error::InvalidParameter("Cannot parse as unsigned integer".into()))
            }
            _ => Err(Error::InvalidParameter(
                "Value cannot be converted to u64".into(),
            )),
        }
    }

    /// Convert to f64, consuming the value.
    pub fn as_float(self) -> Result<f64> {
        match self {
            Value::Float(f) => Ok(f),
            Value::Int(i) => Ok(i as f64),
            Value::UInt(u) => Ok(u as f64),
            Value::Bytes(b) => {
                let s = String::from_utf8(b)
                    .map_err(|_| Error::InvalidParameter("Invalid UTF-8".into()))?;
                s.parse()
                    .map_err(|_| Error::InvalidParameter("Cannot parse as float".into()))
            }
            _ => Err(Error::InvalidParameter(
                "Value cannot be converted to f64".into(),
            )),
        }
    }

    /// Check if value is NULL.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

/// A row returned from a query.
pub type Row = Vec<Value>;
