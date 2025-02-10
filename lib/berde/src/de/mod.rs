/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;

mod json;
// urlencoded
// toml
// yaml

// MARK: Traits

/// Deserialize error
#[derive(Debug)]
pub struct DeserializeError;

/// Deserializer trait
pub trait Deserializer {
    /// Deserialize a boolean
    fn deserialize_bool(&mut self) -> Result<bool, DeserializeError>;
    /// Deserialize a signed 8-bit integer
    fn deserialize_i8(&mut self) -> Result<i8, DeserializeError>;
    /// Deserialize a signed 16-bit integer
    fn deserialize_i16(&mut self) -> Result<i16, DeserializeError>;
    /// Deserialize a signed 32-bit integer
    fn deserialize_i32(&mut self) -> Result<i32, DeserializeError>;
    /// Deserialize a signed 64-bit integer
    fn deserialize_i64(&mut self) -> Result<i64, DeserializeError>;
    /// Deserialize an unsigned 8-bit integer
    fn deserialize_u8(&mut self) -> Result<u8, DeserializeError>;
    /// Deserialize an unsigned 16-bit integer
    fn deserialize_u16(&mut self) -> Result<u16, DeserializeError>;
    /// Deserialize an unsigned 32-bit integer
    fn deserialize_u32(&mut self) -> Result<u32, DeserializeError>;
    /// Deserialize an unsigned 64-bit integer
    fn deserialize_u64(&mut self) -> Result<u64, DeserializeError>;
    /// Deserialize a 32-bit floating point number
    fn deserialize_f32(&mut self) -> Result<f32, DeserializeError>;
    /// Deserialize a 64-bit floating point number
    fn deserialize_f64(&mut self) -> Result<f64, DeserializeError>;
    /// Deserialize a string
    fn deserialize_str(&mut self) -> Result<&str, DeserializeError>;
    /// Deserialize a byte array
    fn deserialize_bytes(&mut self) -> Result<&[u8], DeserializeError>;
    /// Deserialize a none value
    fn deserialize_none(&mut self) -> Result<(), DeserializeError>;

    /// Deserialize a start of a sequence
    fn deserialize_start_seq(&mut self) -> Result<usize, DeserializeError>;
    /// Deserialize a end of a sequence
    fn deserialize_end_seq(&mut self) -> Result<(), DeserializeError>;

    /// Deserialize a start of a struct
    fn deserialize_start_struct(&mut self, name: &str) -> Result<usize, DeserializeError>;
    /// Deserialize a field
    fn deserialize_field(&mut self) -> Option<&str>;
    /// Deserialize a end of a struct
    fn deserialize_end_struct(&mut self) -> Result<(), DeserializeError>;

    /// Deserialize skip next item
    fn deserialize_skip(&mut self) -> Result<(), DeserializeError>;
}

/// Deserialize trait
pub trait Deserialize: Sized {
    /// Deserialize a value
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError>;
}

// MARK: Implementations
impl Deserialize for bool {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_bool()
    }
}

impl Deserialize for i8 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_i8()
    }
}

impl Deserialize for i16 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_i16()
    }
}

impl Deserialize for i32 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_i32()
    }
}

impl Deserialize for i64 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_i64()
    }
}

impl Deserialize for u8 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_u8()
    }
}

impl Deserialize for u16 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_u16()
    }
}

impl Deserialize for u32 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_u32()
    }
}

impl Deserialize for u64 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_u64()
    }
}

impl Deserialize for f32 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_f32()
    }
}

impl Deserialize for f64 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_f64()
    }
}

impl Deserialize for String {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_str().map(|s| s.to_string())
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        match deserializer.deserialize_none() {
            Ok(()) => Ok(None),
            Err(_) => Ok(Some(T::deserialize(deserializer)?)),
        }
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        let len = deserializer.deserialize_start_seq()?;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::deserialize(deserializer)?);
        }
        deserializer.deserialize_end_seq()?;
        Ok(vec)
    }
}

impl<T: Deserialize> Deserialize for HashMap<String, T> {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        let len = deserializer.deserialize_start_struct("HashMap")?;
        let mut map = HashMap::with_capacity(len);
        while let Some(key) = deserializer.deserialize_field() {
            map.insert(key.to_string(), T::deserialize(deserializer)?);
        }
        deserializer.deserialize_end_struct()?;
        Ok(map)
    }
}
