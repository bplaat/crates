/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{c_char, c_void, CStr};
use std::marker::PhantomData;
use std::slice;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::sys::*;
use crate::value::{Value, ValueDeserializer, ValueSerializer, ValuesDeserializer};

pub struct Statement<T> {
    statement: *mut sqlite3_stmt,
    _maker: PhantomData<T>,
}

impl<T> Statement<T> {
    pub(crate) fn new(statement: *mut sqlite3_stmt) -> Self {
        Self {
            statement,
            _maker: PhantomData,
        }
    }

    pub fn bind(&self, params: impl Serialize) -> Result<()> {
        // Reset statement
        unsafe { sqlite3_reset(self.statement) };

        // Serialize params
        let mut serializer = ValueSerializer::new();
        params
            .serialize(&mut serializer)
            .map_err(|_| Error::new("Can't serialize params"))?;

        // Bind values
        let mut index = 1;
        for value in serializer.into_inner() {
            let result = match value {
                Value::Null => unsafe { sqlite3_bind_null(self.statement, index) },
                Value::Integer(i) => unsafe { sqlite3_bind_int64(self.statement, index, i) },
                Value::Real(f) => unsafe { sqlite3_bind_double(self.statement, index, f) },
                Value::Text(s) => unsafe {
                    sqlite3_bind_text(
                        self.statement,
                        index,
                        s.as_ptr() as *const c_char,
                        s.as_bytes().len() as i32,
                        SQLITE_TRANSIENT,
                    )
                },
                Value::Blob(b) => unsafe {
                    sqlite3_bind_blob(
                        self.statement,
                        index,
                        b.as_ptr() as *const c_void,
                        b.len() as i32,
                        SQLITE_TRANSIENT,
                    )
                },
            };
            if result != SQLITE_OK {
                return Err(Error::new("Can't bind value"));
            }
            index += 1;
        }
        Ok(())
    }

    fn read_values(&self) -> Vec<Value> {
        let column_count = unsafe { sqlite3_column_count(self.statement) };
        let mut values: Vec<_> = Vec::with_capacity(column_count as usize);
        for index in 0..column_count {
            values.push(
                #[allow(non_snake_case)]
                match unsafe { sqlite3_column_type(self.statement, index) } {
                    SQLITE_NULL => Value::Null,
                    SQLITE_INTEGER => {
                        Value::Integer(unsafe { sqlite3_column_int64(self.statement, index) })
                    }
                    SQLITE_FLOAT => {
                        Value::Real(unsafe { sqlite3_column_double(self.statement, index) })
                    }
                    SQLITE_TEXT => {
                        let text = unsafe { sqlite3_column_text(self.statement, index) };
                        let text: String = unsafe { CStr::from_ptr(text as *const c_char) }
                            .to_string_lossy()
                            .into_owned();
                        Value::Text(text)
                    }
                    SQLITE_BLOB => {
                        let blob = unsafe { sqlite3_column_blob(self.statement, index) };
                        let size = unsafe { sqlite3_column_bytes(self.statement, index) };
                        let blob =
                            unsafe { slice::from_raw_parts(blob as *const u8, size as usize) }
                                .to_vec();
                        Value::Blob(blob)
                    }
                    _ => unreachable!(),
                },
            );
        }
        values
    }
}

impl<T> Iterator for Statement<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = unsafe { sqlite3_step(self.statement) };
        if result == SQLITE_ROW {
            let values = self.read_values();

            // Deserialize values to type
            if values.len() == 1 {
                let deserializer = ValueDeserializer::new(values.first().expect("Should be some"));
                Some(T::deserialize(deserializer).map_err(|_| Error::new("Can't deserialize row")))
            } else {
                let deserializer = ValuesDeserializer::new(values);
                Some(T::deserialize(deserializer).map_err(|_| Error::new("Can't deserialize row")))
            }
        } else if result == SQLITE_DONE {
            None
        } else {
            Some(Err(Error::new("Can't step statement")))
        }
    }
}

impl<T> Drop for Statement<T> {
    fn drop(&mut self) {
        unsafe { sqlite3_finalize(self.statement) };
    }
}