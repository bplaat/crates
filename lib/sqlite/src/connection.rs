/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::ptr::{null, null_mut};

use anyhow::{bail, Result};
use libsqlite3_sys::*;
use serde::{Deserialize, Serialize};

use crate::statement::Statement;
use crate::value::ValueSerializer;

struct Raw(*mut sqlite3);
unsafe impl Send for Raw {}
unsafe impl Sync for Raw {}

pub struct Connection {
    db: Raw,
}

impl Connection {
    fn new(db: *mut sqlite3) -> Self {
        Self { db: Raw(db) }
    }

    pub fn open(path: &str) -> Result<Self> {
        // Open database
        let mut db = ptr::null_mut();
        let path = CString::new(path)?;
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
        Ok(Self::new(db))
    }

    pub fn execute(&self, query: impl AsRef<str>) -> Result<()> {
        // Execute query
        let query = CString::new(query.as_ref())?;
        let mut error: *mut c_char = ptr::null_mut();
        let result =
            unsafe { sqlite3_exec(self.db.0, query.as_ptr(), None, null_mut(), &mut error) };
        if result != SQLITE_OK {
            let error = unsafe { CStr::from_ptr(error) };
            bail!("Error: {}", error.to_str()?);
        }
        Ok(())
    }

    pub fn query<T>(&self, query: impl AsRef<str>, params: impl Serialize) -> Result<Statement<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Prepare statement
        let statement = self.prepare_statement::<T>(query.as_ref())?;

        // Serialize parameters to values
        let mut serializer = ValueSerializer::new();
        params.serialize(&mut serializer)?;
        let values = serializer.into_inner();

        // Bind values to statement
        statement.bind_values(&values)?;
        Ok(statement)
    }

    fn prepare_statement<T>(&self, query: &str) -> Result<Statement<T>> {
        let mut statement = ptr::null_mut();
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
        Ok(Statement::new(statement))
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { sqlite3_close(self.db.0) };
    }
}
