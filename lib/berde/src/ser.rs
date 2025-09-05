/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::{BTreeMap, HashMap, HashSet};

// MARK: Traits

/// Serializer trait
pub trait Serializer {
    // Primitive types
    /// Serialize a boolean
    fn serialize_bool(&mut self, value: bool);
    /// Serialize a signed 8-bit integer
    fn serialize_i8(&mut self, value: i8);
    /// Serialize a signed 16-bit integer
    fn serialize_i16(&mut self, value: i16);
    /// Serialize a signed 32-bit integer
    fn serialize_i32(&mut self, value: i32);
    /// Serialize a signed 64-bit integer
    fn serialize_i64(&mut self, value: i64);
    /// Serialize an unsigned 8-bit integer
    fn serialize_u8(&mut self, value: u8);
    /// Serialize an unsigned 16-bit integer
    fn serialize_u16(&mut self, value: u16);
    /// Serialize an unsigned 32-bit integer
    fn serialize_u32(&mut self, value: u32);
    /// Serialize an unsigned 64-bit integer
    fn serialize_u64(&mut self, value: u64);
    /// Serialize a 32-bit floating point number
    fn serialize_f32(&mut self, value: f32);
    /// Serialize a 64-bit floating point number
    fn serialize_f64(&mut self, value: f64);
    /// Serialize a character
    fn serialize_char(&mut self, value: char);
    /// Serialize a string
    fn serialize_str(&mut self, value: &str);
    /// Serialize a byte array
    fn serialize_bytes(&mut self, value: &[u8]);

    // Option
    /// Serialize a some value
    fn serialize_some(&mut self, value: &dyn Serialize);
    /// Serialize a none value
    fn serialize_none(&mut self);

    // Seq
    /// Serialize a start of a sequence
    fn serialize_start_seq(&mut self, len: usize);
    /// Serialize a end of a sequence
    fn serialize_end_seq(&mut self);
    /// Serialize a element
    fn serialize_element(&mut self, value: &dyn Serialize);

    // Struct
    /// Serialize a start of a struct
    fn serialize_start_struct(&mut self, name: &str, len: usize);
    /// Serialize a end of a struct
    fn serialize_end_struct(&mut self);
    /// Serialize a field
    fn serialize_field(&mut self, name: &str, value: &dyn Serialize);
}

/// Serialize trait
pub trait Serialize {
    /// Serialize the value into the serializer
    fn serialize(&self, serializer: &mut dyn Serializer);
}

// MARK: Implementations
impl Serialize for bool {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_bool(*self);
    }
}

impl Serialize for i8 {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_i8(*self);
    }
}

impl Serialize for i16 {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_i16(*self);
    }
}

impl Serialize for i32 {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_i32(*self);
    }
}

impl Serialize for i64 {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_i64(*self);
    }
}

impl Serialize for u8 {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_u8(*self);
    }
}

impl Serialize for u16 {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_u16(*self);
    }
}

impl Serialize for u32 {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_u32(*self);
    }
}

impl Serialize for u64 {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_u64(*self);
    }
}

impl Serialize for f32 {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_f32(*self);
    }
}

impl Serialize for f64 {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_f64(*self);
    }
}

impl Serialize for char {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_char(*self);
    }
}

impl Serialize for &str {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_str(self);
    }
}

impl Serialize for String {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_str(self);
    }
}

impl Serialize for &[u8] {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_bytes(self);
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        if let Some(value) = self {
            serializer.serialize_some(value);
        } else {
            serializer.serialize_none();
        }
    }
}

macro_rules! impl_serialize_for_seq {
    ($ty:ident) => {
        impl<T: Serialize> Serialize for $ty<T> {
            fn serialize(&self, serializer: &mut dyn Serializer) {
                serializer.serialize_start_seq(self.len());
                for value in self {
                    serializer.serialize_element(value);
                }
                serializer.serialize_end_seq();
            }
        }
    };
}
impl_serialize_for_seq!(Vec);
impl_serialize_for_seq!(HashSet);

macro_rules! impl_serialize_for_map {
    ($ty:ident) => {
        impl<V: Serialize> Serialize for $ty<String, V> {
            fn serialize(&self, serializer: &mut dyn Serializer) {
                serializer.serialize_start_struct(stringify!($ty), self.len());
                for (key, value) in self {
                    serializer.serialize_field(&key, value);
                }
                serializer.serialize_end_struct();
            }
        }
    };
}
impl_serialize_for_map!(HashMap);
impl_serialize_for_map!(BTreeMap);
