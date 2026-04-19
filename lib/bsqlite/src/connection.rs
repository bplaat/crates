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

use crate::{Bind, FromRow, Statement, StatementError};

// MARK: Inner Connection
/// The mode to open the database in
pub enum OpenMode {
    /// Read only
    ReadOnly,
    /// Read and write
    ReadWrite,
}

struct InnerConnection(*mut sqlite3);
// SAFETY: InnerConnection exclusively owns its *mut sqlite3 handle and never aliases it, so
// transferring ownership to another thread is safe.
unsafe impl Send for InnerConnection {}
// SAFETY: SQLite opened with SQLITE_OPEN_FULLMUTEX serializes all API calls with an internal
// mutex, so shared access from multiple threads via &InnerConnection is safe.
unsafe impl Sync for InnerConnection {}

impl InnerConnection {
    fn open(path: &Path, mode: OpenMode) -> Result<Self, ConnectionError> {
        // Open database
        let mut db = ptr::null_mut();
        let path = CString::new(path.to_str().expect("Can't convert to CString").as_bytes())
            .expect("Can't convert to CString");
        // SAFETY: path is a valid NUL-terminated CString, db is initialized to null_mut, the
        // flags are valid SQLite open mode constants, and the vfs argument is null (use default).
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
            let error = if db.is_null() {
                "unknown error (db handle is null)".into()
            } else {
                // SAFETY: db is non-null (checked above), and sqlite3_errmsg returns a valid
                // NUL-terminated string that remains valid until the next SQLite API call on db.
                unsafe { CStr::from_ptr(sqlite3_errmsg(db)) }.to_string_lossy()
            };
            return Err(ConnectionError {
                msg: format!("Failed to open database: {error}"),
            });
        }
        Ok(InnerConnection(db))
    }

    fn prepare<T: FromRow>(&self, query: &str) -> Result<Statement<T>, StatementError> {
        let mut statement = ptr::null_mut();
        // SAFETY: self.0 is a valid open db handle, query bytes are valid UTF-8 from a &str with
        // the correct byte length, statement is initialized to null_mut, and the tail pointer is
        // null (not needed here).
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
            // SAFETY: self.0 is a valid open db handle, and sqlite3_errmsg returns a valid
            // NUL-terminated string that remains valid until the next SQLite API call.
            let error = unsafe { CStr::from_ptr(sqlite3_errmsg(self.0)) }.to_string_lossy();
            return Err(StatementError {
                msg: format!("Failed to prepare statement '{query}': {error}"),
            });
        }
        Ok(Statement::new(statement))
    }

    fn affected_rows(&self) -> i32 {
        // SAFETY: self.0 is a valid open db handle.
        unsafe { sqlite3_changes(self.0) }
    }

    fn last_insert_row_id(&self) -> i64 {
        // SAFETY: self.0 is a valid open db handle.
        unsafe { sqlite3_last_insert_rowid(self.0) }
    }
}

impl Drop for InnerConnection {
    fn drop(&mut self) {
        // SAFETY: self.0 is the exclusively owned db handle; Drop guarantees no other references
        // exist, and sqlite3_close frees the handle exactly once.
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
    pub fn enable_wal_logging(&self) -> Result<(), StatementError> {
        self.execute("PRAGMA journal_mode = WAL", ())
    }

    /// Apply various performance settings to the database
    pub fn apply_various_performance_settings(&self) -> Result<(), StatementError> {
        // Apply some SQLite performance settings (https://briandouglas.ie/sqlite-defaults/)
        // - Set synchronous mode to NORMAL for performance and data safety balance
        self.execute("PRAGMA synchronous = NORMAL", ())?;
        // - Set busy timeout to 5 seconds to avoid "database is locked" errors
        self.execute("PRAGMA busy_timeout = 5000", ())?;
        // - Set cache size to 20MB for faster data access
        self.execute("PRAGMA cache_size = 20000", ())?;
        // - Enable foreign key constraint enforcement
        self.execute("PRAGMA foreign_keys = ON", ())?;
        // - Enable auto vacuuming and set it to incremental mode for gradual space reclaiming
        self.execute("PRAGMA auto_vacuum = INCREMENTAL", ())?;
        // - Store temporary tables and data in memory for better performance
        self.execute("PRAGMA temp_store = MEMORY", ())?;
        // - Set the mmap_size to 2GB for faster read/write access using memory-mapped I/O
        self.execute("PRAGMA mmap_size = 2147483648", ())?;
        // - Set the page size to 8KB for balanced memory usage and performance
        self.execute("PRAGMA page_size = 8192", ())?;
        Ok(())
    }

    /// Prepare a statement
    pub fn prepare<T: FromRow>(
        &self,
        query: impl AsRef<str>,
    ) -> Result<Statement<T>, StatementError> {
        self.0.prepare(query.as_ref())
    }

    /// Run a query
    pub fn query<T: FromRow>(
        &self,
        query: impl AsRef<str>,
        params: impl Bind,
    ) -> Result<Statement<T>, StatementError> {
        let mut statement = self.prepare::<T>(query.as_ref())?;
        statement.bind(params)?;
        Ok(statement)
    }

    /// Run a query, read and expect the first row
    pub fn query_some<T: FromRow>(
        &self,
        query: impl AsRef<str>,
        params: impl Bind,
    ) -> Result<T, StatementError> {
        self.query::<T>(query.as_ref(), params)?
            .next()
            .transpose()?
            .ok_or_else(|| StatementError {
                msg: "expected at least one row from query".to_string(),
            })
    }

    /// Execute a query
    pub fn execute(&self, query: impl AsRef<str>, params: impl Bind) -> Result<(), StatementError> {
        self.query::<()>(query.as_ref(), params)?
            .next()
            .transpose()?;
        Ok(())
    }

    /// Get the number of affected rows
    pub fn affected_rows(&self) -> i32 {
        self.0.affected_rows()
    }

    /// Create FTS5 virtual table and sync triggers for a table
    pub fn create_fts_tables(&self, table: &str, columns: &[&str]) -> Result<(), StatementError> {
        let cols = columns.join(", ");
        let new_cols = columns
            .iter()
            .map(|c| format!("new.{c}"))
            .collect::<Vec<_>>()
            .join(", ");
        let set_cols = columns
            .iter()
            .map(|c| format!("{c} = new.{c}"))
            .collect::<Vec<_>>()
            .join(", ");

        self.execute(
            format!(
                "CREATE VIRTUAL TABLE IF NOT EXISTS {table}_fts USING fts5({cols}, id UNINDEXED)"
            ),
            (),
        )?;
        self.execute(
            format!(
                "CREATE TRIGGER IF NOT EXISTS {table}_ai AFTER INSERT ON {table} BEGIN
                    INSERT INTO {table}_fts({cols}, id) VALUES ({new_cols}, new.id);
                END"
            ),
            (),
        )?;
        self.execute(
            format!(
                "CREATE TRIGGER IF NOT EXISTS {table}_au AFTER UPDATE ON {table} BEGIN
                    UPDATE {table}_fts SET {set_cols} WHERE id = old.id;
                END"
            ),
            (),
        )?;
        self.execute(
            format!(
                "CREATE TRIGGER IF NOT EXISTS {table}_ad BEFORE DELETE ON {table} BEGIN
                    DELETE FROM {table}_fts WHERE id = old.id;
                END"
            ),
            (),
        )?;
        Ok(())
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
        (|| -> std::result::Result<_, $crate::StatementError> {
            let mut stat = $db.prepare::<$t>($query)?;
            $(
                stat.bind_named_value(concat!(":", stringify!($key)), Into::<$crate::Value>::into($value))?;
            )*
            Ok(stat)
        })()
    }};
}

/// Execute a query with named arguments
#[macro_export]
macro_rules! execute_args {
    ($db:expr, $query:expr, Args { $($key:ident : $value:expr),* $(,)? } $(,)?) => {{
        (|| -> std::result::Result<_, $crate::StatementError> {
            let mut stat = $db.prepare::<()>($query)?;
            $(
                stat.bind_named_value(concat!(":", stringify!($key)), Into::<$crate::Value>::into($value))?;
            )*
            stat.next().transpose()?;
            Ok(())
        })()
    }};
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_open_db_execute_queries() -> Result<(), StatementError> {
        let db = Connection::open_memory().unwrap();
        db.execute(
            "CREATE TABLE persons (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, age INTEGER) STRICT",
            (),
        )?;
        db.execute(
            "INSERT INTO persons (name, age) VALUES (?, ?)",
            ("Alice".to_string(), 30),
        )?;
        execute_args!(
            db,
            "INSERT INTO persons (name, age) VALUES (:name, :age)",
            Args {
                name: "Bob".to_string(),
                age: 40,
            },
        )?;

        let total = db.query_some::<i64>("SELECT COUNT(id) FROM persons", ())?;
        assert_eq!(total, 2);
        let names = db
            .query::<(String, i64)>("SELECT name, age FROM persons", ())?
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            names,
            vec![("Alice".to_string(), 30), ("Bob".to_string(), 40)]
        );
        Ok(())
    }

    #[test]
    fn test_all_types_crud() -> Result<(), StatementError> {
        let db = Connection::open_memory().unwrap();
        db.execute(
            "CREATE TABLE data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                int_val INTEGER NOT NULL,
                float_val REAL NOT NULL,
                text_val TEXT NOT NULL,
                blob_val BLOB NOT NULL,
                opt_int INTEGER
            )",
            (),
        )?;

        // INSERT all value types including NULL optional
        db.execute(
            "INSERT INTO data (int_val, float_val, text_val, blob_val, opt_int) VALUES (?, ?, ?, ?, ?)",
            (42i64, 3.11f64, "hello".to_string(), vec![0xCAu8, 0xFE], Option::<i64>::None),
        )?;
        assert_eq!(db.last_insert_row_id(), 1);

        // SELECT and verify all types round-trip
        let row = db.query_some::<(i64, f64, String, Vec<u8>, Option<i64>)>(
            "SELECT int_val, float_val, text_val, blob_val, opt_int FROM data WHERE id = 1",
            (),
        )?;
        assert_eq!(row.0, 42);
        assert_eq!(row.1, 3.11);
        assert_eq!(row.2, "hello");
        assert_eq!(row.3, vec![0xCAu8, 0xFE]);
        assert_eq!(row.4, None);

        // UPDATE and verify affected_rows
        db.execute("UPDATE data SET int_val = 99 WHERE id = 1", ())?;
        assert_eq!(db.affected_rows(), 1);
        let updated = db.query_some::<i64>("SELECT int_val FROM data WHERE id = 1", ())?;
        assert_eq!(updated, 99);

        // INSERT second row with Some optional
        db.execute(
            "INSERT INTO data (int_val, float_val, text_val, blob_val, opt_int) VALUES (?, ?, ?, ?, ?)",
            (0i64, 0.0f64, String::new(), vec![], Some(7i64)),
        )?;
        let opt = db.query_some::<Option<i64>>("SELECT opt_int FROM data WHERE id = 2", ())?;
        assert_eq!(opt, Some(7));

        // DELETE and verify count drops
        db.execute("DELETE FROM data WHERE id = 1", ())?;
        assert_eq!(db.affected_rows(), 1);
        let count = db.query_some::<i64>("SELECT COUNT(*) FROM data", ())?;
        assert_eq!(count, 1);

        Ok(())
    }

    #[test]
    fn test_query_some_empty_error() {
        let db = Connection::open_memory().unwrap();
        db.execute("CREATE TABLE empty (id INTEGER PRIMARY KEY)", ())
            .unwrap();
        let result = db.query_some::<i64>("SELECT id FROM empty", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_rows_iteration() -> Result<(), StatementError> {
        let db = Connection::open_memory().unwrap();
        db.execute("CREATE TABLE nums (n INTEGER NOT NULL)", ())?;
        for i in 1i64..=5 {
            db.execute("INSERT INTO nums (n) VALUES (?)", i)?;
        }
        let rows: Vec<i64> = db
            .query::<i64>("SELECT n FROM nums ORDER BY n", ())?
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(rows, vec![1, 2, 3, 4, 5]);
        Ok(())
    }

    #[test]
    fn test_named_args_macro() -> Result<(), StatementError> {
        let db = Connection::open_memory().unwrap();
        db.execute(
            "CREATE TABLE kv (key TEXT NOT NULL, val INTEGER NOT NULL)",
            (),
        )?;
        execute_args!(
            db,
            "INSERT INTO kv (key, val) VALUES (:key, :val)",
            Args {
                key: "answer".to_string(),
                val: 42i64,
            },
        )?;
        let rows: Vec<i64> = query_args!(
            i64,
            db,
            "SELECT val FROM kv WHERE key = :key",
            Args {
                key: "answer".to_string(),
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(rows, vec![42]);
        Ok(())
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_uuid_roundtrip() -> Result<(), StatementError> {
        use uuid::Uuid;
        let db = Connection::open_memory().unwrap();
        db.execute("CREATE TABLE items (id BLOB NOT NULL)", ())?;
        let uuid = Uuid::from_bytes([
            0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4,
            0x30, 0xc8,
        ]);
        db.execute("INSERT INTO items (id) VALUES (?)", uuid)?;
        let fetched: Uuid = db.query_some("SELECT id FROM items", ())?;
        assert_eq!(fetched, uuid);
        Ok(())
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono_roundtrip() -> Result<(), StatementError> {
        use chrono::{DateTime, NaiveDate, Utc};
        let db = Connection::open_memory().unwrap();
        db.execute(
            "CREATE TABLE events (day INTEGER NOT NULL, ts INTEGER NOT NULL)",
            (),
        )?;
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let ts = DateTime::<Utc>::from_timestamp_secs(1_700_000_000).unwrap();
        db.execute("INSERT INTO events (day, ts) VALUES (?, ?)", (date, ts))?;
        let (fetched_date, fetched_ts): (NaiveDate, DateTime<Utc>) =
            db.query_some("SELECT day, ts FROM events", ())?;
        assert_eq!(fetched_date, date);
        assert_eq!(fetched_ts, ts);
        Ok(())
    }
}
