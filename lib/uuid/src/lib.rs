/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter, Write};
use std::str::FromStr;

#[derive(Clone, Copy)]
pub struct Uuid([u8; 16]);

impl Uuid {
    pub fn from_bytes(bytes: [u8; 16]) -> Uuid {
        Uuid(bytes)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Uuid, InvalidError> {
        if slice.len() != 16 {
            return Err(InvalidError);
        }
        let mut bytes = [0; 16];
        bytes.copy_from_slice(slice);
        Ok(Uuid(bytes))
    }

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
#[derive(Debug)]
pub struct InvalidError;

impl Display for InvalidError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid UUID")
    }
}

impl Error for InvalidError {}
