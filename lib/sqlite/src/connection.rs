/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::ffi::{c_char, CString};
use std::fmt::{self, Display, Formatter};
use std::ptr;
use std::sync::Arc;

use crate::statement::{Bind, FromRow, Statement};
use crate::sys::*;

// MARK: Raw Connection
struct RawConnection(*mut sqlite3);
unsafe impl Send for RawConnection {}
unsafe impl Sync for RawConnection {}

impl RawConnection {
    fn open(path: &str) -> Result<Self, ConnectionError> {
        // Open database
        let mut db = ptr::null_mut();
        let path = CString::new(path).expect("Can't convert &str to CString");
        let result = unsafe {
            sqlite3_open_v2(
                path.as_ptr(),
                &mut db,
                SQLITE_OPEN_CREATE | SQLITE_OPEN_READWRITE | SQLITE_OPEN_FULLMUTEX,
                ptr::null(),
            )
        };
        if result != SQLITE_OK {
            return Err(ConnectionError);
        }
        let db: RawConnection = RawConnection(db);

        // Apply some SQLite performance settings (https://briandouglas.ie/sqlite-defaults/):
        // - Set the journal mode to Write-Ahead Logging for concurrency
        db.execute("PRAGMA journal_mode = WAL", ());
        // - Set synchronous mode to NORMAL for performance and data safety balance
        db.execute("PRAGMA synchronous = NORMAL", ());
        // - Set busy timeout to 5 seconds to avoid "database is locked" errors
        db.execute("PRAGMA busy_timeout = 5000", ());
        // - Set cache size to 20MB for faster data access
        db.execute("PRAGMA cache_size = 20000", ());
        // - Enable foreign key constraint enforcement
        db.execute("PRAGMA foreign_keys = ON", ());
        // - Enable auto vacuuming and set it to incremental mode for gradual space reclaiming
        db.execute("PRAGMA auto_vacuum = INCREMENTAL", ());
        // - Store temporary tables and data in memory for better performance
        db.execute("PRAGMA temp_store = MEMORY", ());
        // - Set the mmap_size to 2GB for faster read/write access using memory-mapped I/O
        db.execute("PRAGMA mmap_size = 2147483648", ());
        // - Set the page size to 8KB for balanced memory usage and performance
        db.execute("PRAGMA page_size = 8192", ());

        Ok(db)
    }

    fn prepare<T>(&self, query: impl AsRef<str>) -> Statement<T>
    where
        T: FromRow,
    {
        let query = query.as_ref();
        let mut statement = ptr::null_mut();
        let result = unsafe {
            sqlite3_prepare_v2(
                self.0,
                query.as_ptr() as *const c_char,
                query.as_bytes().len() as i32,
                &mut statement,
                ptr::null_mut(),
            )
        };
        if result != SQLITE_OK {
            panic!("Can't prepare statement");
        }
        Statement::new(statement)
    }

    fn query<T>(&self, query: impl AsRef<str>, params: impl Bind) -> Statement<T>
    where
        T: FromRow,
    {
        let mut statement = self.prepare::<T>(query.as_ref());
        statement.bind(params);
        statement
    }

    fn execute(&self, query: impl AsRef<str>, params: impl Bind) {
        self.query::<()>(query.as_ref(), params).next();
    }
}

impl Drop for RawConnection {
    fn drop(&mut self) {
        unsafe { sqlite3_close(self.0) };
    }
}

// MARK: Connection Error
/// A connection error
#[derive(Debug)]
pub struct ConnectionError;

impl Display for ConnectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Connection error")
    }
}

impl Error for ConnectionError {}

// MARK: Connection
#[derive(Clone)]
/// A SQLite connection
pub struct Connection(Arc<RawConnection>);

impl Connection {
    /// Open a connection to a SQLite database
    pub fn open(path: &str) -> Result<Self, ConnectionError> {
        Ok(Connection(Arc::new(RawConnection::open(path)?)))
    }

    /// Prepare a statement
    pub fn prepare<T>(&self, query: impl AsRef<str>) -> Statement<T>
    where
        T: FromRow,
    {
        self.0.prepare(query)
    }

    /// Run a query
    pub fn query<T>(&self, query: impl AsRef<str>, params: impl Bind) -> Statement<T>
    where
        T: FromRow,
    {
        self.0.query(query, params)
    }

    /// Execute a query
    pub fn execute(&self, query: impl AsRef<str>, params: impl Bind) {
        self.0.execute(query, params);
    }
}
