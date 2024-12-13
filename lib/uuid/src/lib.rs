/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal UUID library

use std::error::Error;
use std::fmt::{self, Display, Formatter, Write};
use std::str::FromStr;

// MARK: Uuid
/// UUID
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Uuid([u8; 16]);

impl Uuid {
    /// Create zero UUID
    pub fn nil() -> Uuid {
        Uuid([0; 16])
    }

    /// Create UUID from bytes
    pub fn from_bytes(bytes: [u8; 16]) -> Uuid {
        Uuid(bytes)
    }

    /// Create UUID from slice
    pub fn from_slice(slice: &[u8]) -> Result<Uuid, InvalidError> {
        if slice.len() != 16 {
            return Err(InvalidError);
        }
        let mut bytes = [0; 16];
        bytes.copy_from_slice(slice);
        Ok(Uuid(bytes))
    }

    /// Get bytes from UUID
    pub fn into_bytes(self) -> [u8; 16] {
        self.0
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for i in 0..16 {
            f.write_char(match self.0[i] >> 4 {
                0..=9 => (b'0' + (self.0[i] >> 4)) as char,
                _ => (b'a' + (self.0[i] >> 4) - 10) as char,
            })?;
            f.write_char(match self.0[i] & 0x0f {
                0..=9 => (b'0' + (self.0[i] & 0x0f)) as char,
                _ => (b'a' + (self.0[i] & 0x0f) - 10) as char,
            })?;
            if i == 3 || i == 5 || i == 7 || i == 9 {
                f.write_char('-')?;
            }
        }
        Ok(())
    }
}

impl FromStr for Uuid {
    type Err = InvalidError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 36 {
            return Err(InvalidError);
        }
        let mut bytes = [0; 16];
        let mut n = 0;
        for (i, c) in s.chars().enumerate() {
            if i == 8 || i == 13 || i == 18 || i == 23 {
                if c != '-' {
                    return Err(InvalidError);
                }
                continue;
            }
            let x = match c {
                '0'..='9' => c as u8 - b'0',
                'a'..='f' => c as u8 - b'a' + 10,
                'A'..='F' => c as u8 - b'A' + 10,
                _ => return Err(InvalidError),
            };
            if n % 2 == 0 {
                bytes[n / 2] = x << 4;
            } else {
                bytes[n / 2] |= x;
            }
            n += 1;
        }
        Ok(Uuid(bytes))
    }
}

#[cfg(feature = "v4")]
impl Uuid {
    /// Create UUID v4
    pub fn new_v4() -> Uuid {
        let mut bytes = [0; 16];
        getrandom::getrandom(&mut bytes).unwrap();
        bytes[6] = bytes[6] & 0x0f | 0x40;
        bytes[8] = bytes[8] & 0x3f | 0x80;
        Uuid(bytes)
    }
}

#[cfg(feature = "v7")]
impl Uuid {
    /// Create UUID v7 with time
    pub fn new_v7(time: std::time::SystemTime) -> Uuid {
        let mut bytes = [0; 16];
        let timestamp = time
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        bytes[0] = (timestamp >> 40) as u8;
        bytes[1] = (timestamp >> 32) as u8;
        bytes[2] = (timestamp >> 24) as u8;
        bytes[3] = (timestamp >> 16) as u8;
        bytes[4] = (timestamp >> 8) as u8;
        bytes[5] = timestamp as u8;
        getrandom::getrandom(&mut bytes[6..]).unwrap();
        bytes[6] = bytes[6] & 0x0f | 0x70;
        bytes[8] = bytes[8] & 0x3f | 0x80;
        Uuid(bytes)
    }

    /// Create UUID v7 with current time
    pub fn now_v7() -> Uuid {
        Self::new_v7(std::time::SystemTime::now())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Uuid {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Uuid {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Uuid::from_str(&s).map_err(serde::de::Error::custom)
    }
}

// MARK: InvalidError
/// Invalid UUID error
#[derive(Debug)]
pub struct InvalidError;

impl Display for InvalidError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid UUID")
    }
}

impl Error for InvalidError {}

// MARK: Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_slice() {
        let uuid = Uuid::from_slice(&[
            0xa0, 0xb1, 0xc2, 0xd3, 0xe4, 0xf5, 0x67, 0x89, 0x9a, 0x0b, 0xcd, 0xef, 0x01, 0x23,
            0x45, 0x67,
        ])
        .unwrap();
        assert_eq!(uuid.to_string(), "a0b1c2d3-e4f5-6789-9a0b-cdef01234567");
    }

    #[test]
    fn from_slice_invalid() {
        let uuid =
            Uuid::from_slice(&[0xa0, 0xb1, 0xc2, 0xd3, 0xe4, 0xf5, 0x67, 0x89, 0x9a]).unwrap_err();
        assert!(matches!(uuid, InvalidError));
    }

    #[test]
    fn to_string() {
        let uuid = Uuid::from_bytes([
            0xa0, 0xb1, 0xc2, 0xd3, 0xe4, 0xf5, 0x67, 0x89, 0x9a, 0x0b, 0xcd, 0xef, 0x01, 0x23,
            0x45, 0x67,
        ]);
        assert_eq!(uuid.to_string(), "a0b1c2d3-e4f5-6789-9a0b-cdef01234567");
    }

    #[test]
    fn parse_string() {
        let uuid = "a0b1c2d3-e4f5-6789-9a0b-cdef01234567"
            .parse::<Uuid>()
            .unwrap();
        assert_eq!(
            uuid,
            Uuid::from_bytes([
                0xa0, 0xb1, 0xc2, 0xd3, 0xe4, 0xf5, 0x67, 0x89, 0x9a, 0x0b, 0xcd, 0xef, 0x01, 0x23,
                0x45, 0x67,
            ])
        );
    }

    #[test]
    fn parse_invalid_string() {
        let uuid = "a0b1c2d3e4f567899a0bcdef01234567"
            .parse::<Uuid>()
            .unwrap_err();
        assert!(matches!(uuid, InvalidError));

        let uuid = "a0b1c2d3-e4f5-6789-9a0".parse::<Uuid>().unwrap_err();
        assert!(matches!(uuid, InvalidError));
    }

    #[test]
    fn generate_v4() {
        let uuid = Uuid::new_v4();
        let bytes = uuid.into_bytes();
        assert_eq!(bytes.len(), 16);
        assert_eq!(bytes[6] >> 4, 4);
        assert!(matches!(bytes[8] >> 6, 2 | 3));
    }

    #[test]
    fn generate_v7() {
        let uuid = Uuid::now_v7();
        let bytes = uuid.into_bytes();
        assert_eq!(bytes.len(), 16);
        assert_eq!(bytes[6] >> 4, 7);
        assert!(matches!(bytes[8] >> 6, 2 | 3));
    }

    #[test]
    fn serde_serialization() {
        let uuid = Uuid::nil();
        let serialized = serde_json::to_string(&uuid).unwrap();
        assert_eq!(serialized, "\"00000000-0000-0000-0000-000000000000\"");
    }

    #[test]
    fn serde_deserialization() {
        let data = "\"a0b1c2d3-e4f5-6789-9a0b-cdef01234567\"";
        let uuid: Uuid = serde_json::from_str(data).unwrap();
        assert_eq!(
            uuid,
            Uuid::from_bytes([
                0xa0, 0xb1, 0xc2, 0xd3, 0xe4, 0xf5, 0x67, 0x89, 0x9a, 0x0b, 0xcd, 0xef, 0x01, 0x23,
                0x45, 0x67,
            ])
        );
    }

    #[test]
    fn serde_invalid_deserialization() {
        let data = "\"invalid-uuid-string\"";
        let result: Result<Uuid, _> = serde_json::from_str(data);
        assert!(result.is_err());
    }
}
