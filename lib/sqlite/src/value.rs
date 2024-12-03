/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::{error, fmt};

use anyhow::Result;
use serde::de::{self, DeserializeSeed, Deserializer, SeqAccess, Visitor};
use serde::ser::{self, Serialize, Serializer};

#[derive(Debug)]
pub(crate) enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

// MARK: ValueError
#[derive(Debug)]
pub(crate) struct ValueError {
    message: String,
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ValueError: {}", self.message)
    }
}

impl error::Error for ValueError {}

impl serde::ser::Error for ValueError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        ValueError {
            message: msg.to_string(),
        }
    }
}

// MARK: ValueSerializer
pub(crate) struct ValueSerializer {
    output: Vec<Value>,
}

impl ValueSerializer {
    pub(crate) fn new() -> Self {
        Self { output: Vec::new() }
    }

    pub(crate) fn into_inner(self) -> Vec<Value> {
        self.output
    }
}

impl Serializer for &mut ValueSerializer {
    type Ok = ();
    type Error = ValueError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Integer(v as i64));
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Integer(v as i64));
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Integer(v as i64));
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Integer(v as i64));
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Integer(v));
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Integer(v as i64));
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Integer(v as i64));
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Integer(v as i64));
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Integer(v as i64));
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Real(v as f64));
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Real(v));
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Text(v.to_string()));
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        // Hack: Save UUID as blob
        #[cfg(feature = "uuid")]
        {
            if let Ok(uuid) = Uuid::parse_str(v) {
                self.output.push(Value::Blob(uuid.as_bytes().to_vec()));
            } else {
                self.output.push(Value::Text(v.to_string()));
            }
        }
        #[cfg(not(feature = "uuid"))]
        {
            self.output.push(Value::Text(v.to_string()));
        }
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Blob(v.to_vec()));
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Null);
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.output.push(Value::Text(variant.to_string()));
        Ok(())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.output.push(Value::Text(variant.to_string()));
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.output.push(Value::Text(variant.to_string()));
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.output.push(Value::Text(variant.to_string()));
        Ok(self)
    }
}

impl ser::SerializeSeq for &mut ValueSerializer {
    type Ok = ();
    type Error = ValueError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeTuple for &mut ValueSerializer {
    type Ok = ();
    type Error = ValueError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeTupleStruct for &mut ValueSerializer {
    type Ok = ();
    type Error = ValueError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeTupleVariant for &mut ValueSerializer {
    type Ok = ();
    type Error = ValueError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeMap for &mut ValueSerializer {
    type Ok = ();
    type Error = ValueError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeStruct for &mut ValueSerializer {
    type Ok = ();
    type Error = ValueError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut ValueSerializer {
    type Ok = ();
    type Error = ValueError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// MARK: ValuesDeserializer
pub(crate) struct ValuesDeserializer {
    values: Vec<Value>,
    index: usize,
}

impl ValuesDeserializer {
    pub(crate) fn new(values: Vec<Value>) -> Self {
        Self { values, index: 0 }
    }
}

impl<'de> Deserializer<'de> for ValuesDeserializer {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for ValuesDeserializer {
    type Error = de::value::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.index >= self.values.len() {
            return Ok(None);
        }
        let value = &self.values[self.index];
        self.index += 1;
        seed.deserialize(ValueDeserializer(value)).map(Some)
    }
}

pub(crate) struct ValueDeserializer<'a>(&'a Value);

impl<'a> ValueDeserializer<'a> {
    pub(crate) fn new(value: &'a Value) -> Self {
        ValueDeserializer(value)
    }
}

impl<'de> Deserializer<'de> for ValueDeserializer<'_> {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Null => visitor.visit_none(),
            Value::Integer(i) => visitor.visit_i64(*i),
            Value::Real(f) => visitor.visit_f64(*f),
            Value::Text(s) => visitor.visit_str(s),
            Value::Blob(b) => visitor.visit_bytes(b),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
