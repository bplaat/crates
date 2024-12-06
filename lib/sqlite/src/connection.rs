/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{c_char, CString};
use std::ptr;
use std::ptr::{null, null_mut};

use crate::error::{Error, Result};
use crate::statement::{Bind, FromRow, Statement};
use crate::sys::*;

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

        // Apply some SQLite performance settings (https://briandouglas.ie/sqlite-defaults/):
        // - Set the journal mode to Write-Ahead Logging for concurrency
        db.execute("PRAGMA journal_mode = WAL")?;
        // - Set synchronous mode to NORMAL for performance and data safety balance
        db.execute("PRAGMA synchronous = NORMAL")?;
        // - Set busy timeout to 5 seconds to avoid "database is locked" errors
        db.execute("PRAGMA busy_timeout = 5000")?;
        // - Set cache size to 20MB for faster data access
        db.execute("PRAGMA cache_size = 20000")?;
        // - Enable foreign key constraint enforcement
        db.execute("PRAGMA foreign_keys = ON")?;
        // - Enable auto vacuuming and set it to incremental mode for gradual space reclaiming
        db.execute("PRAGMA auto_vacuum = INCREMENTAL")?;
        // - Store temporary tables and data in memory for better performance
        db.execute("PRAGMA temp_store = MEMORY")?;
        // - Set the mmap_size to 2GB for faster read/write access using memory-mapped I/O
        db.execute("PRAGMA mmap_size = 2147483648")?;
        // - Set the page size to 8KB for balanced memory usage and performance
        db.execute("PRAGMA page_size = 8192")?;

        Ok(db)
    }

    pub fn execute(&self, query: impl AsRef<str>) -> Result<()> {
        // Execute query
        let query = CString::new(query.as_ref()).expect("Can't convert &str to CString");
        let result = unsafe {
            sqlite3_exec(
                self.db.0,
                query.as_ptr(),
                null_mut(),
                null_mut(),
                null_mut(),
            )
        };
        if result != SQLITE_OK {
            return Err(Error::new("Can't execute query"));
        }
        Ok(())
    }

    pub fn prepare<T>(&self, query: impl AsRef<str>) -> Result<Statement<T>>
    where
        T: FromRow,
    {
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

    pub fn query<T>(&self, query: impl AsRef<str>, params: impl Bind) -> Result<Statement<T>>
    where
        T: FromRow,
    {
        let mut statement = self.prepare::<T>(query.as_ref())?;
        statement.bind(params)?;
        Ok(statement)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { sqlite3_close(self.db.0) };
    }
}
