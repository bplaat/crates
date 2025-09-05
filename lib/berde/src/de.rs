/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::Hash;

// MARK: Traits

/// Deserialize error
#[derive(Debug)]
pub struct DeserializeError;

/// Deserialize result
pub type Result<T> = std::result::Result<T, DeserializeError>;

/// Deserializer trait
pub trait Deserializer {
    // Primitives
    /// Deserialize a boolean
    fn deserialize_bool(&mut self) -> Result<bool>;
    /// Deserialize a signed 8-bit integer
    fn deserialize_i8(&mut self) -> Result<i8>;
    /// Deserialize a signed 16-bit integer
    fn deserialize_i16(&mut self) -> Result<i16>;
    /// Deserialize a signed 32-bit integer
    fn deserialize_i32(&mut self) -> Result<i32>;
    /// Deserialize a signed 64-bit integer
    fn deserialize_i64(&mut self) -> Result<i64>;
    /// Deserialize an unsigned 8-bit integer
    fn deserialize_u8(&mut self) -> Result<u8>;
    /// Deserialize an unsigned 16-bit integer
    fn deserialize_u16(&mut self) -> Result<u16>;
    /// Deserialize an unsigned 32-bit integer
    fn deserialize_u32(&mut self) -> Result<u32>;
    /// Deserialize an unsigned 64-bit integer
    fn deserialize_u64(&mut self) -> Result<u64>;
    /// Deserialize a 32-bit floating point number
    fn deserialize_f32(&mut self) -> Result<f32>;
    /// Deserialize a 64-bit floating point number
    fn deserialize_f64(&mut self) -> Result<f64>;
    /// Deserialize a string
    fn deserialize_str(&mut self) -> Result<&str>;
    /// Deserialize a byte array
    fn deserialize_bytes(&mut self) -> Result<&[u8]>;

    // Option
    /// Deserialize a option
    fn deserialize_option(&mut self) -> Result<Option<&mut dyn Deserializer>>;

    // Seq
    /// Deserialize a start of a sequence
    fn deserialize_start_seq(&mut self) -> Result<()>;
    /// Deserialize a end of a sequence
    fn deserialize_end_seq(&mut self) -> Result<()>;
    /// Deserialize a element
    fn deserialize_element(&mut self) -> Result<Option<&mut dyn Deserializer>>;

    // Struct
    /// Deserialize a start of a struct
    fn deserialize_start_struct(&mut self, name: &str) -> Result<()>;
    /// Deserialize a end of a struct
    fn deserialize_end_struct(&mut self) -> Result<()>;
    /// Deserialize a field
    fn deserialize_field(&mut self) -> Result<Option<(&str, &mut dyn Deserializer)>>;
}

/// Deserialize trait
pub trait Deserialize: Sized {
    /// Deserialize a value
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self>;
}

// MARK: Implementations
impl Deserialize for bool {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_bool()
    }
}

impl Deserialize for i8 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_i8()
    }
}

impl Deserialize for i16 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_i16()
    }
}

impl Deserialize for i32 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_i32()
    }
}

impl Deserialize for i64 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_i64()
    }
}

impl Deserialize for u8 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_u8()
    }
}

impl Deserialize for u16 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_u16()
    }
}

impl Deserialize for u32 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_u32()
    }
}

impl Deserialize for u64 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_u64()
    }
}

impl Deserialize for f32 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_f32()
    }
}

impl Deserialize for f64 {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_f64()
    }
}

impl Deserialize for String {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_str().map(|s| s.to_string())
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        match deserializer.deserialize_option()? {
            Some(value_deserializer) => Ok(Some(T::deserialize(value_deserializer)?)),
            None => Ok(None),
        }
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_start_seq()?;
        let mut vec = Vec::new();
        while let Some(value_deserializer) = deserializer.deserialize_element()? {
            vec.push(T::deserialize(value_deserializer)?);
        }
        deserializer.deserialize_end_seq()?;
        Ok(vec)
    }
}

impl<T: Eq + Hash + Deserialize> Deserialize for HashSet<T> {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_start_seq()?;
        let mut vec = HashSet::new();
        while let Some(value_deserializer) = deserializer.deserialize_element()? {
            vec.insert(T::deserialize(value_deserializer)?);
        }
        deserializer.deserialize_end_seq()?;
        Ok(vec)
    }
}

impl<T: Deserialize> Deserialize for HashMap<String, T> {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_start_struct("HashMap")?;
        let mut map = HashMap::new();
        while let Some((key, value_deserializer)) = deserializer.deserialize_field()? {
            map.insert(key.to_string(), T::deserialize(value_deserializer)?);
        }
        deserializer.deserialize_end_struct()?;
        Ok(map)
    }
}

impl<T: Deserialize> Deserialize for BTreeMap<String, T> {
    fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self> {
        deserializer.deserialize_start_struct("BTreeMap")?;
        let mut map = BTreeMap::new();
        while let Some((key, value_deserializer)) = deserializer.deserialize_field()? {
            map.insert(key.to_string(), T::deserialize(value_deserializer)?);
        }
        deserializer.deserialize_end_struct()?;
        Ok(map)
    }
}
