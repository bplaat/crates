/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_char;
use std::fmt;
use std::marker::PhantomData;
use std::ptr::{null, null_mut};

use anyhow::{anyhow, bail, Context, Result};
use libsqlite3_sys::*;
use serde::de::{self, Deserialize, DeserializeSeed, Deserializer, SeqAccess, Visitor};
use serde::ser::{self, Serialize, Serializer};

// MARK: Connection
struct Raw(*mut sqlite3);
unsafe impl Send for Raw {}
unsafe impl Sync for Raw {}

pub struct Connection {
    db: Raw,
}

impl Connection {
    pub fn open(path: &str) -> Result<Self> {
        // Open database
        let mut db = std::ptr::null_mut();
        let path = std::ffi::CString::new(path)?;
        let result = unsafe {
            sqlite3_open_v2(
                path.as_ptr(),
                &mut db,
                SQLITE_OPEN_CREATE | SQLITE_OPEN_READWRITE | SQLITE_OPEN_FULLMUTEX,
                null(),
            )
        };
        if result != SQLITE_OK {
            bail!("Can't open database");
        }
        Ok(Self { db: Raw(db) })
    }

    pub fn execute(&self, query: &str) -> Result<()> {
        // Execute query
        let query = std::ffi::CString::new(query)?;
        let mut error: *mut c_char = std::ptr::null_mut();
        let result =
            unsafe { sqlite3_exec(self.db.0, query.as_ptr(), None, null_mut(), &mut error) };
        if result != SQLITE_OK {
            let error = unsafe { std::ffi::CStr::from_ptr(error) };
            bail!("Error: {}", error.to_str()?);
        }
        Ok(())
    }

    pub fn query<T>(&self, query: &str, params: impl Serialize) -> Result<Statement<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Prepare statement
        let mut statement = std::ptr::null_mut();
        let result = unsafe {
            sqlite3_prepare_v2(
                self.db.0,
                query.as_ptr() as *const c_char,
                query.as_bytes().len() as i32,
                &mut statement,
                null_mut(),
            )
        };
        if result != SQLITE_OK {
            bail!("Can't prepare statement: {}", result);
        }

        // Serialize parameters to values
        let mut serializer = ValueSerializer::new();
        params.serialize(&mut serializer)?;
        let values = serializer.into_inner();

        // Bind values to statement
        for (index, param) in values.iter().enumerate() {
            let result = match param {
                Value::Null => unsafe { sqlite3_bind_null(statement, index as i32 + 1) },
                Value::Integer(value) => unsafe {
                    sqlite3_bind_int64(statement, index as i32 + 1, *value)
                },
                Value::Real(value) => unsafe {
                    sqlite3_bind_double(statement, index as i32 + 1, *value)
                },
                Value::Text(value) => unsafe {
                    sqlite3_bind_text(
                        statement,
                        index as i32 + 1,
                        value.as_ptr() as *const c_char,
                        value.as_bytes().len() as i32,
                        SQLITE_TRANSIENT(),
                    )
                },
                Value::Blob(value) => unsafe {
                    sqlite3_bind_blob(
                        statement,
                        index as i32 + 1,
                        value.as_ptr() as *const std::ffi::c_void,
                        value.len() as i32,
                        SQLITE_TRANSIENT(),
                    )
                },
            };
            if result != SQLITE_OK {
                bail!("Can't bind value to statement");
            }
        }
        Ok(Statement::<T>(statement, PhantomData::<T>))
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { sqlite3_close(self.db.0) };
    }
}

// MARK: Statement
pub struct Statement<T>(*mut sqlite3_stmt, PhantomData<T>);

impl<T> Iterator for Statement<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = unsafe { sqlite3_step(self.0) };
        if result == SQLITE_ROW {
            // Read values from statement
            let column_count = unsafe { sqlite3_column_count(self.0) };
            let mut values: Vec<_> = Vec::with_capacity(column_count as usize);
            for i in 0..column_count {
                values.push(match unsafe { sqlite3_column_type(self.0, i) } {
                    SQLITE_NULL => Value::Null,
                    SQLITE_INTEGER => Value::Integer(unsafe { sqlite3_column_int64(self.0, i) }),
                    SQLITE_FLOAT => Value::Real(unsafe { sqlite3_column_double(self.0, i) }),
                    SQLITE_TEXT => {
                        let text = unsafe { sqlite3_column_text(self.0, i) };
                        let text = unsafe { std::ffi::CStr::from_ptr(text as *const c_char) }
                            .to_string_lossy()
                            .into_owned();
                        Value::Text(text)
                    }
                    SQLITE_BLOB => {
                        let blob = unsafe { sqlite3_column_blob(self.0, i) };
                        let size = unsafe { sqlite3_column_bytes(self.0, i) };
                        let blob =
                            unsafe { std::slice::from_raw_parts(blob as *const u8, size as usize) }
                                .to_vec();
                        Value::Blob(blob)
                    }
                    _ => unreachable!(),
                });
            }

            // Deserialize values to type
            let deserializer = ValuesDeserializer::new(values);
            Some(T::deserialize(deserializer).context("Can't deserialize row"))
        } else if result == SQLITE_DONE {
            None
        } else {
            Some(Err(anyhow!("Can't step statement")))
        }
    }
}

impl<T> Drop for Statement<T> {
    fn drop(&mut self) {
        unsafe { sqlite3_finalize(self.0) };
    }
}

// MARK: Value
#[derive(Debug)]
enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

// MARK: ValueError
#[derive(Debug)]
struct ValueError {
    message: String,
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ValueError: {}", self.message)
    }
}

impl std::error::Error for ValueError {}

impl serde::ser::Error for ValueError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        ValueError {
            message: msg.to_string(),
        }
    }
}

// MARK: ValueSerializer
struct ValueSerializer {
    output: Vec<Value>,
}

impl ValueSerializer {
    pub fn new() -> Self {
        Self { output: Vec::new() }
    }

    fn into_inner(self) -> Vec<Value> {
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
        self.output.push(Value::Text(v.to_string()));
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
struct ValuesDeserializer {
    values: Vec<Value>,
    index: usize,
}

impl ValuesDeserializer {
    pub fn new(values: Vec<Value>) -> Self {
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

struct ValueDeserializer<'a>(&'a Value);

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