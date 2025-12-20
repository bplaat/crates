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

use libsqlite3_sys::*;

use crate::{Bind, FromRow, Statement};

// MARK: Inner Connection
/// The mode to open the database in
pub enum OpenMode {
    /// Read only
    ReadOnly,
    /// Read and write
    ReadWrite,
}

struct InnerConnection(*mut sqlite3);
unsafe impl Send for InnerConnection {}
unsafe impl Sync for InnerConnection {}

impl InnerConnection {
    fn open(path: &Path, mode: OpenMode) -> Result<Self, ConnectionError> {
        // Open database
        let mut db = ptr::null_mut();
        let path = CString::new(path.to_str().expect("Can't convert to CString").as_bytes())
            .expect("Can't convert to CString");
        let result = unsafe {
            sqlite3_open_v2(
                path.as_ptr(),
                &mut db,
                match mode {
                    OpenMode::ReadOnly => SQLITE_OPEN_READONLY | SQLITE_OPEN_FULLMUTEX,
                    OpenMode::ReadWrite => {
                        SQLITE_OPEN_CREATE | SQLITE_OPEN_READWRITE | SQLITE_OPEN_FULLMUTEX
                    }
                },
                ptr::null(),
            )
        };
        if result != SQLITE_OK {
            let error = unsafe { CStr::from_ptr(sqlite3_errmsg(db)) }.to_string_lossy();
            return Err(ConnectionError {
                msg: format!("Failed to open database: {error}"),
            });
        }
        Ok(InnerConnection(db))
    }

    fn prepare<T: FromRow>(&self, query: &str) -> Statement<T> {
        let mut statement = ptr::null_mut();
        let result = unsafe {
            sqlite3_prepare_v2(
                self.0,
                query.as_ptr() as *const c_char,
                query.len() as i32,
                &mut statement,
                ptr::null_mut(),
            )
        };
        if result != SQLITE_OK {
            let error = unsafe { CStr::from_ptr(sqlite3_errmsg(self.0)) }.to_string_lossy();
            panic!("bsqlite: Failed to prepare SQL statement!\n  Query: {query}\n  Error: {error}");
        }
        Statement::new(statement)
    }

    fn affected_rows(&self) -> i32 {
        unsafe { sqlite3_changes(self.0) }
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
    pub fn open(path: impl AsRef<Path>, mode: OpenMode) -> Result<Self, ConnectionError> {
        Ok(Connection(Arc::new(InnerConnection::open(
            path.as_ref(),
            mode,
        )?)))
    }

    /// Open a memory database
    pub fn open_memory() -> Result<Self, ConnectionError> {
        Self::open(":memory:", OpenMode::ReadWrite)
    }

    /// Set the journal mode to Write-Ahead Logging for better concurrency throughput
    pub fn enable_wal_logging(&self) {
        self.execute("PRAGMA journal_mode = WAL", ());
    }

    /// Apply various performance settings to the database
    pub fn apply_various_performance_settings(&self) {
        // Apply some SQLite performance settings (https://briandouglas.ie/sqlite-defaults/)
        // - Set synchronous mode to NORMAL for performance and data safety balance
        self.execute("PRAGMA synchronous = NORMAL", ());
        // - Set busy timeout to 5 seconds to avoid "database is locked" errors
        self.execute("PRAGMA busy_timeout = 5000", ());
        // - Set cache size to 20MB for faster data access
        self.execute("PRAGMA cache_size = 20000", ());
        // - Enable foreign key constraint enforcement
        self.execute("PRAGMA foreign_keys = ON", ());
        // - Enable auto vacuuming and set it to incremental mode for gradual space reclaiming
        self.execute("PRAGMA auto_vacuum = INCREMENTAL", ());
        // - Store temporary tables and data in memory for better performance
        self.execute("PRAGMA temp_store = MEMORY", ());
        // - Set the mmap_size to 2GB for faster read/write access using memory-mapped I/O
        self.execute("PRAGMA mmap_size = 2147483648", ());
        // - Set the page size to 8KB for balanced memory usage and performance
        self.execute("PRAGMA page_size = 8192", ());
    }

    /// Prepare a statement
    pub fn prepare<T: FromRow>(&self, query: impl AsRef<str>) -> Statement<T> {
        self.0.prepare(query.as_ref())
    }

    /// Run a query
    pub fn query<T: FromRow>(&self, query: impl AsRef<str>, params: impl Bind) -> Statement<T> {
        let mut statement = self.prepare::<T>(query.as_ref());
        statement.bind(params);
        statement
    }

    /// Run a query, read and expect the first row
    pub fn query_some<T: FromRow>(&self, query: impl AsRef<str>, params: impl Bind) -> T {
        self.query::<T>(query.as_ref(), params)
            .next()
            .expect("Should be some")
    }

    /// Execute a query
    pub fn execute(&self, query: impl AsRef<str>, params: impl Bind) {
        self.query::<()>(query.as_ref(), params).next();
    }

    /// Get the number of affected rows
    pub fn affected_rows(&self) -> i32 {
        self.0.affected_rows()
    }

    /// Get the last inserted row id
    pub fn last_insert_row_id(&self) -> i64 {
        self.0.last_insert_row_id()
    }
}

// MARK: Macros

/// Run a query with named arguments
#[macro_export]
macro_rules! query_args {
    ($t:tt, $db:expr, $query:expr, Args { $($key:ident : $value:expr),* $(,)? } $(,)?) => {{
        let mut stat = $db.prepare::<$t>($query);
        $(
            stat.bind_named_value(concat!(":", stringify!($key)), Into::<$crate::Value>::into($value));
        )*
        stat
    }};
}

/// Execute a query with named arguments
#[macro_export]
macro_rules! execute_args {
    ($db:expr, $query:expr, Args { $($key:ident : $value:expr),* $(,)? } $(,)?) => {{
        let mut stat = $db.prepare::<()>($query);
        $(
            stat.bind_named_value(concat!(":", stringify!($key)), Into::<$crate::Value>::into($value));
        )*
        stat.next();
    }};
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_open_db_execute_queries() {
        let db = Connection::open_memory().unwrap();
        db.execute(
            "CREATE TABLE persons (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, age INTEGER) STRICT",
            (),
        );
        db.execute(
            "INSERT INTO persons (name, age) VALUES (?, ?)",
            ("Alice".to_string(), 30),
        );
        execute_args!(
            db,
            "INSERT INTO persons (name, age) VALUES (:name, :age)",
            Args {
                name: "Bob".to_string(),
                age: 40,
            },
        );

        let total = db.query_some::<i64>("SELECT COUNT(id) FROM persons", ());
        assert_eq!(total, 2);
        let names = db
            .query::<(String, i64)>("SELECT name, age FROM persons", ())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![("Alice".to_string(), 30), ("Bob".to_string(), 40)]
        );
    }
}
