/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::ffi::{c_char, CStr, CString};
use std::fmt::{self, Display, Formatter};
use std::path::Path;
use std::ptr;
use std::sync::Arc;

use sqlite3_sys::*;

use crate::statement::{Bind, FromRow, Statement};

// MARK: Inner Connection
struct InnerConnection(*mut sqlite3);
unsafe impl Send for InnerConnection {}
unsafe impl Sync for InnerConnection {}

impl InnerConnection {
    fn open(path: &Path) -> Result<Self, ConnectionError> {
        // Open database
        let mut db = ptr::null_mut();
        let path = CString::new(
            path.to_str()
                .expect("Can't convert &Path to CString")
                .as_bytes(),
        )
        .expect("Can't convert &Path to CString");
        let result = unsafe {
            sqlite3_open_v2(
                path.as_ptr(),
                &mut db,
                SQLITE_OPEN_CREATE | SQLITE_OPEN_READWRITE | SQLITE_OPEN_FULLMUTEX,
                ptr::null(),
            )
        };
        if result != SQLITE_OK {
            let error = unsafe { CStr::from_ptr(sqlite3_errmsg(db)) }.to_string_lossy();
            return Err(ConnectionError {
                msg: format!("Failed to open database: {}", error),
            });
        }
        let db = InnerConnection(db);

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

    fn prepare<T>(&self, query: &str) -> Statement<T>
    where
        T: FromRow,
    {
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
            let error = unsafe { CStr::from_ptr(sqlite3_errmsg(self.0)) }.to_string_lossy();
            panic!("Failed to prepare statement: {}", error);
        }
        Statement::new(statement)
    }

    fn query<T>(&self, query: &str, params: impl Bind) -> Statement<T>
    where
        T: FromRow,
    {
        let mut statement = self.prepare::<T>(query);
        statement.bind(params);
        statement
    }

    fn execute(&self, query: &str, params: impl Bind) {
        self.query::<()>(query, params).next();
    }

    fn affected_rows(&self) -> i64 {
        unsafe { sqlite3_changes64(self.0) }
    }

    fn last_insert_row_id(&self) -> i64 {
        unsafe { sqlite3_last_insert_rowid(self.0) }
    }
}

impl Drop for InnerConnection {
    fn drop(&mut self) {
        unsafe { sqlite3_close(self.0) };
    }
}

// MARK: Connection Error
/// A connection error
#[derive(Debug)]
pub struct ConnectionError {
    msg: String,
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Connection error: {}", self.msg)
    }
}

impl Error for ConnectionError {}

// MARK: Connection
/// A SQLite connection
#[derive(Clone)]
pub struct Connection(Arc<InnerConnection>);

impl Connection {
    /// Open a connection to a SQLite database
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ConnectionError> {
        Ok(Connection(Arc::new(InnerConnection::open(path.as_ref())?)))
    }

    /// Prepare a statement
    pub fn prepare<T>(&self, query: impl AsRef<str>) -> Statement<T>
    where
        T: FromRow,
    {
        self.0.prepare(query.as_ref())
    }

    /// Run a query
    pub fn query<T>(&self, query: impl AsRef<str>, params: impl Bind) -> Statement<T>
    where
        T: FromRow,
    {
        self.0.query(query.as_ref(), params)
    }

    /// Execute a query
    pub fn execute(&self, query: impl AsRef<str>, params: impl Bind) {
        self.0.execute(query.as_ref(), params);
    }

    /// Get the number of affected rows
    pub fn affected_rows(&self) -> i64 {
        self.0.affected_rows()
    }

    /// Get the last inserted row id
    pub fn last_insert_row_id(&self) -> i64 {
        self.0.last_insert_row_id()
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    #[test]
    fn test_open_db_execute_queries() {
        let connection = super::Connection::open(":memory:").unwrap();
        connection.execute(
            "CREATE TABLE persons (id INTEGER PRIMARY KEY, name TEXT)",
            (),
        );
        connection.execute("INSERT INTO persons (name) VALUES (?)", "Alice".to_string());
        connection.execute("INSERT INTO persons (name) VALUES (?)", "Bob".to_string());
        let names = connection
            .query::<String>("SELECT name FROM persons", ())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["Alice".to_string(), "Bob".to_string()]);
    }
}
