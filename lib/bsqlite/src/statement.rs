/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::ffi::{c_char, c_void, CStr, CString};
use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;

use libsqlite3_sys::*;

use crate::{Bind, FromRow, Value};

// MARK: Statement Error
/// A statement error
#[derive(Debug)]
pub struct StatementError {
    pub(crate) msg: String,
}

impl StatementError {
    #[doc(hidden)]
    pub fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

impl Display for StatementError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Statement error: {}", self.msg)
    }
}

impl Error for StatementError {}

// MARK: Column Type
/// Column type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    /// Null type
    Null,
    /// Integer type
    Integer,
    /// Float type
    Float,
    /// Text type
    Text,
    /// Blob type
    Blob,
}

// MARK: Raw Statement
/// Raw SQLite statement without type information
pub struct RawStatement(*mut sqlite3_stmt);

impl RawStatement {
    pub(crate) fn new(statement: *mut sqlite3_stmt) -> Self {
        Self(statement)
    }

    /// Reset the statement
    pub fn reset(&mut self) {
        // SAFETY: self.0 is a valid prepared statement handle.
        unsafe { sqlite3_reset(self.0) };
    }

    /// Bind values to the statement
    pub fn bind(&mut self, params: impl Bind) -> Result<(), StatementError> {
        params.bind(self)
    }

    /// Bind value to the statement
    pub fn bind_value(&mut self, index: i32, value: Value) -> Result<(), StatementError> {
        let index = index + 1;
        let result = match value {
            Value::Null => {
                // SAFETY: self.0 is a valid prepared statement handle and index is a valid
                // 1-based parameter index.
                unsafe { sqlite3_bind_null(self.0, index) }
            }
            Value::Integer(i) => {
                // SAFETY: self.0 is a valid prepared statement handle and index is a valid
                // 1-based parameter index.
                unsafe { sqlite3_bind_int64(self.0, index, i) }
            }
            Value::Float(f) => {
                // SAFETY: self.0 is a valid prepared statement handle and index is a valid
                // 1-based parameter index.
                unsafe { sqlite3_bind_double(self.0, index, f) }
            }
            Value::Text(s) => {
                let len = i32::try_from(s.len()).map_err(|_| StatementError {
                    msg: "text value too large to bind".to_string(),
                })?;
                // SAFETY: self.0 is a valid prepared statement, index is valid, s.as_ptr() points
                // to valid UTF-8 bytes for the given len, and SQLITE_TRANSIENT causes SQLite to
                // copy the data before returning.
                unsafe {
                    sqlite3_bind_text(
                        self.0,
                        index,
                        s.as_ptr() as *const c_char,
                        len,
                        SQLITE_TRANSIENT(),
                    )
                }
            }
            Value::Blob(b) => {
                let len = i32::try_from(b.len()).map_err(|_| StatementError {
                    msg: "blob value too large to bind".to_string(),
                })?;
                // SAFETY: self.0 is a valid prepared statement, index is valid, b.as_ptr() points
                // to valid bytes for the given len, and SQLITE_TRANSIENT causes SQLite to copy
                // the data before returning.
                unsafe {
                    sqlite3_bind_blob(
                        self.0,
                        index,
                        b.as_ptr() as *const c_void,
                        len,
                        SQLITE_TRANSIENT(),
                    )
                }
            }
        };
        if result != SQLITE_OK {
            // SAFETY: sqlite3_sql returns a NUL-terminated string that lives as long as the
            // statement; self.0 is a valid prepared statement handle.
            let query = unsafe { CStr::from_ptr(sqlite3_sql(self.0)) }.to_string_lossy();
            // SAFETY: sqlite3_db_handle always returns the db associated with self.0 (never null);
            // sqlite3_errmsg returns a valid NUL-terminated string until the next API call.
            let error = unsafe { CStr::from_ptr(sqlite3_errmsg(sqlite3_db_handle(self.0))) }
                .to_string_lossy();
            return Err(StatementError {
                msg: format!("Failed to bind value to statement '{query}': {error}"),
            });
        }
        Ok(())
    }

    /// Bind named value to the statement
    pub fn bind_named_value(&mut self, name: &str, value: Value) -> Result<(), StatementError> {
        let c_name = CString::new(name).expect("Can't convert to CString");
        // SAFETY: self.0 is a valid prepared statement handle and c_name is a valid
        // NUL-terminated CString.
        let index = unsafe { sqlite3_bind_parameter_index(self.0, c_name.as_ptr()) };
        if index == 0 {
            return Err(StatementError {
                msg: format!("Parameter '{name}' not found in statement"),
            });
        }
        self.bind_value(index - 1, value)
    }

    /// Step the statement
    pub fn step(&mut self) -> Result<Option<()>, StatementError> {
        // SAFETY: self.0 is a valid prepared statement handle.
        let result = unsafe { sqlite3_step(self.0) };
        if result == SQLITE_ROW {
            Ok(Some(()))
        } else if result == SQLITE_DONE {
            Ok(None)
        } else {
            // SAFETY: sqlite3_sql returns a NUL-terminated string that lives as long as the
            // statement; self.0 is a valid prepared statement handle.
            let query = unsafe { CStr::from_ptr(sqlite3_sql(self.0)) }.to_string_lossy();
            // SAFETY: sqlite3_db_handle always returns the db associated with self.0 (never null);
            // sqlite3_errmsg returns a valid NUL-terminated string until the next API call.
            let error = unsafe { CStr::from_ptr(sqlite3_errmsg(sqlite3_db_handle(self.0))) }
                .to_string_lossy();
            Err(StatementError {
                msg: format!("Failed to step statement '{query}': {error}"),
            })
        }
    }

    /// Get the number of columns in the statement
    pub fn column_count(&self) -> i32 {
        // SAFETY: self.0 is a valid prepared statement handle.
        unsafe { sqlite3_column_count(self.0) }
    }

    /// Get the name of a column
    pub fn column_name(&self, index: i32) -> String {
        // SAFETY: self.0 is a valid prepared statement handle and index is within the column range.
        let name = unsafe { sqlite3_column_name(self.0, index) };
        // SAFETY: sqlite3_column_name returns a valid NUL-terminated string owned by SQLite that
        // remains valid for the lifetime of the statement.
        unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .to_string()
    }

    /// Get the type of a column
    pub fn column_type(&self, index: i32) -> ColumnType {
        // SAFETY: self.0 is a valid prepared statement handle and index is within the column range.
        match unsafe { sqlite3_column_type(self.0, index) } {
            SQLITE_NULL => ColumnType::Null,
            SQLITE_INTEGER => ColumnType::Integer,
            SQLITE_FLOAT => ColumnType::Float,
            SQLITE_TEXT => ColumnType::Text,
            SQLITE_BLOB => ColumnType::Blob,
            r#type => unreachable!("Unknown column type: {}", r#type),
        }
    }

    /// Get the declared type of a column
    pub fn column_declared_type(&self, index: i32) -> Option<String> {
        // SAFETY: self.0 is a valid prepared statement handle and index is within the column range.
        let decl_type = unsafe { sqlite3_column_decltype(self.0, index) };
        if !decl_type.is_null() {
            Some(
                // SAFETY: decl_type is non-null (checked above) and points to a valid
                // NUL-terminated string owned by SQLite, valid for the statement lifetime.
                unsafe { CStr::from_ptr(decl_type) }
                    .to_string_lossy()
                    .to_string(),
            )
        } else {
            None
        }
    }

    /// Get the table name of a column
    pub fn column_table_name(&self, index: i32) -> Option<String> {
        // SAFETY: self.0 is a valid prepared statement handle and index is within the column range.
        let table_name = unsafe { sqlite3_column_table_name(self.0, index) };
        if !table_name.is_null() {
            Some(
                // SAFETY: table_name is non-null (checked above) and points to a valid
                // NUL-terminated string owned by SQLite, valid for the statement lifetime.
                unsafe { CStr::from_ptr(table_name) }
                    .to_string_lossy()
                    .to_string(),
            )
        } else {
            None
        }
    }

    /// Get the origin name of a column
    pub fn column_origin_name(&self, index: i32) -> Option<String> {
        // SAFETY: self.0 is a valid prepared statement handle and index is within the column range.
        let origin_name = unsafe { sqlite3_column_origin_name(self.0, index) };
        if !origin_name.is_null() {
            Some(
                // SAFETY: origin_name is non-null (checked above) and points to a valid
                // NUL-terminated string owned by SQLite, valid for the statement lifetime.
                unsafe { CStr::from_ptr(origin_name) }
                    .to_string_lossy()
                    .to_string(),
            )
        } else {
            None
        }
    }

    /// Get the value of a column
    pub fn column_value(&self, index: i32) -> Value {
        // SAFETY: self.0 is a valid prepared statement handle and index is within the column range.
        match unsafe { sqlite3_column_type(self.0, index) } {
            SQLITE_NULL => Value::Null,
            SQLITE_INTEGER => {
                // SAFETY: self.0 is valid, index is in bounds, and the column type is INTEGER.
                Value::Integer(unsafe { sqlite3_column_int64(self.0, index) })
            }
            SQLITE_FLOAT => {
                // SAFETY: self.0 is valid, index is in bounds, and the column type is FLOAT.
                Value::Float(unsafe { sqlite3_column_double(self.0, index) })
            }
            SQLITE_TEXT => {
                // SAFETY: self.0 is valid, index is in bounds, and the column type is TEXT.
                let text = unsafe { sqlite3_column_text(self.0, index) };
                // SAFETY: sqlite3_column_text returns a valid NUL-terminated UTF-8 string (SQLite
                // guarantees UTF-8 encoding) that is valid until the column value is reread.
                let text = unsafe { CStr::from_ptr(text as *const c_char) }
                    .to_string_lossy()
                    .to_string();
                Value::Text(text)
            }
            SQLITE_BLOB => {
                // SAFETY: self.0 is valid, index is in bounds, and the column type is BLOB.
                let blob = unsafe { sqlite3_column_blob(self.0, index) };
                if blob.is_null() {
                    return Value::Blob(Vec::new());
                }
                // SAFETY: Called on the same column immediately after sqlite3_column_blob to
                // retrieve the correct byte length for that blob.
                let len = unsafe { sqlite3_column_bytes(self.0, index) };
                // SAFETY: blob is non-null (checked above), len matches the blob size returned
                // by sqlite3_column_bytes, and the memory is valid for the duration of this call.
                let slice = unsafe { std::slice::from_raw_parts(blob as *const u8, len as usize) };
                Value::Blob(slice.to_vec())
            }
            r#type => unreachable!("Unknown column type: {}", r#type),
        }
    }
}

impl Drop for RawStatement {
    fn drop(&mut self) {
        // SAFETY: self.0 is the exclusively owned statement handle; Drop guarantees no other
        // references exist, and sqlite3_finalize frees the handle exactly once.
        unsafe { sqlite3_finalize(self.0) };
    }
}

// MARK: Statement
/// A SQLite statement with type information
pub struct Statement<T>(RawStatement, PhantomData<T>);

impl<T> Statement<T> {
    pub(crate) fn new(statement: *mut sqlite3_stmt) -> Self {
        Self(RawStatement::new(statement), PhantomData)
    }

    /// Reset the statement
    pub fn reset(&mut self) {
        self.0.reset();
    }

    /// Bind values to the statement
    pub fn bind(&mut self, params: impl Bind) -> Result<(), StatementError> {
        self.0.bind(params)
    }

    /// Bind value to the statement
    pub fn bind_value(
        &mut self,
        index: i32,
        value: impl Into<Value>,
    ) -> Result<(), StatementError> {
        self.0.bind_value(index, value.into())
    }

    /// Bind named value to the statement
    pub fn bind_named_value(
        &mut self,
        name: &str,
        value: impl Into<Value>,
    ) -> Result<(), StatementError> {
        self.0.bind_named_value(name, value.into())
    }

    /// Step the statement
    pub fn step(&mut self) -> Result<Option<()>, StatementError> {
        self.0.step()
    }

    /// Get the number of columns in the statement
    pub fn column_count(&self) -> i32 {
        self.0.column_count()
    }

    /// Get the name of a column
    pub fn column_name(&self, index: i32) -> String {
        self.0.column_name(index)
    }

    /// Get the type of a column
    pub fn column_type(&self, index: i32) -> ColumnType {
        self.0.column_type(index)
    }

    /// Get the declared type of a column
    pub fn column_declared_type(&self, index: i32) -> Option<String> {
        self.0.column_declared_type(index)
    }

    /// Get the table name of a column
    pub fn column_table_name(&self, index: i32) -> Option<String> {
        self.0.column_table_name(index)
    }

    /// Get the origin name of a column
    pub fn column_origin_name(&self, index: i32) -> Option<String> {
        self.0.column_origin_name(index)
    }

    /// Get the value of a column
    pub fn column_value(&self, index: i32) -> Value {
        self.0.column_value(index)
    }
}

impl<T: FromRow> Iterator for Statement<T> {
    type Item = Result<T, StatementError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.step() {
            Ok(Some(())) => {
                Some(T::from_row(&mut self.0).map_err(|e| StatementError { msg: e.to_string() }))
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ColumnType, Connection, StatementError, Value};

    #[test]
    fn test_statement_metadata_and_column_accessors() -> Result<(), StatementError> {
        let db = Connection::open_memory().unwrap();
        db.execute(
            "CREATE TABLE items (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                score REAL NOT NULL,
                payload BLOB NOT NULL,
                note TEXT
            ) STRICT",
            (),
        )?;
        db.execute(
            "INSERT INTO items (id, name, score, payload, note) VALUES (?, ?, ?, ?, ?)",
            (
                1_i64,
                "widget".to_string(),
                3.5_f64,
                vec![1_u8, 2_u8, 3_u8],
                Option::<String>::None,
            ),
        )?;

        let mut statement =
            db.prepare::<()>("SELECT id AS item_id, name, score, payload, note FROM items")?;

        assert_eq!(statement.column_count(), 5);
        assert_eq!(statement.column_name(0), "item_id");
        assert_eq!(statement.column_name(1), "name");
        assert_eq!(
            statement.column_declared_type(0).as_deref(),
            Some("INTEGER")
        );
        assert_eq!(statement.column_declared_type(1).as_deref(), Some("TEXT"));
        assert_eq!(statement.column_declared_type(2).as_deref(), Some("REAL"));
        assert_eq!(statement.column_declared_type(3).as_deref(), Some("BLOB"));
        assert_eq!(statement.column_declared_type(4).as_deref(), Some("TEXT"));
        assert_eq!(statement.column_table_name(0).as_deref(), Some("items"));
        assert_eq!(statement.column_origin_name(0).as_deref(), Some("id"));

        assert_eq!(statement.step()?, Some(()));

        assert_eq!(statement.column_type(0), ColumnType::Integer);
        assert_eq!(statement.column_type(1), ColumnType::Text);
        assert_eq!(statement.column_type(2), ColumnType::Float);
        assert_eq!(statement.column_type(3), ColumnType::Blob);
        assert_eq!(statement.column_type(4), ColumnType::Null);

        match statement.column_value(0) {
            Value::Integer(value) => assert_eq!(value, 1),
            _ => panic!("expected integer value"),
        }
        match statement.column_value(1) {
            Value::Text(value) => assert_eq!(value, "widget"),
            _ => panic!("expected text value"),
        }
        match statement.column_value(2) {
            Value::Float(value) => assert_eq!(value, 3.5),
            _ => panic!("expected float value"),
        }
        match statement.column_value(3) {
            Value::Blob(value) => assert_eq!(value, vec![1, 2, 3]),
            _ => panic!("expected blob value"),
        }
        assert!(matches!(statement.column_value(4), Value::Null));
        assert_eq!(statement.step()?, None);

        Ok(())
    }

    #[test]
    fn test_statement_reset_allows_rebinding() -> Result<(), StatementError> {
        let db = Connection::open_memory().unwrap();
        let mut statement = db.prepare::<i64>("SELECT ?")?;

        statement.bind(10_i64)?;
        assert_eq!(statement.next().transpose()?, Some(10));

        statement.reset();
        statement.bind(20_i64)?;
        assert_eq!(statement.next().transpose()?, Some(20));

        Ok(())
    }

    #[test]
    fn test_bind_named_value_reports_missing_parameter() {
        let db = Connection::open_memory().unwrap();
        let mut statement = db.prepare::<()>("SELECT 1").unwrap();

        let error = statement.bind_named_value(":missing", 1_i64).unwrap_err();
        assert_eq!(
            error.to_string(),
            "Statement error: Parameter ':missing' not found in statement"
        );
    }
}
