/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{c_char, CString};
use std::ptr;
use std::ptr::{null, null_mut};

use libsqlite3_sys::*;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::statement::Statement;

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
        let path = CString::new(path).expect("Can't convert &str to CString");
        let result = unsafe {
            sqlite3_open_v2(
                path.as_ptr(),
                &mut db,
                SQLITE_OPEN_CREATE | SQLITE_OPEN_READWRITE | SQLITE_OPEN_FULLMUTEX,
                null(),
            )
        };
        if result != SQLITE_OK {
            return Err(Error::new("Can't open database"));
        }
        let db = Self::new(db);

        // Use Write-Ahead Logging  mode
        db.execute("PRAGMA journal_mode=WAL")?;

        Ok(db)
    }

    pub fn execute(&self, query: impl AsRef<str>) -> Result<()> {
        // Execute query
        let query = CString::new(query.as_ref()).expect("Can't convert &str to CString");

        let result =
            unsafe { sqlite3_exec(self.db.0, query.as_ptr(), None, null_mut(), null_mut()) };
        if result != SQLITE_OK {
            return Err(Error::new("Can't execute query"));
        }
        Ok(())
    }

    pub fn prepare<T>(&self, query: impl AsRef<str>) -> Result<Statement<T>> {
        let query = query.as_ref();
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
            return Err(Error::new("Can't prepare statement"));
        }
        Ok(Statement::new(statement))
    }

    pub fn query<T>(&self, query: impl AsRef<str>, params: impl Serialize) -> Result<Statement<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let statement = self.prepare::<T>(query.as_ref())?;
        statement.bind(params)?;
        Ok(statement)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { sqlite3_close(self.db.0) };
    }
}
