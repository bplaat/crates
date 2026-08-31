/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::connection::InnerConnection;
use crate::{Bind, FromRow, Value};

/// A statement error.
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

/// A database column type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    /// Null.
    Null,
    /// Signed integer.
    Integer,
    /// Floating-point number.
    Float,
    /// Text.
    Text,
    /// Binary data.
    Blob,
}

pub(crate) trait PreparedStatement {
    fn reset(&mut self, connection: &InnerConnection);
    fn bind_value(&mut self, index: i32, value: Value) -> Result<(), StatementError>;
    fn bind_named_value(&mut self, name: &str, value: Value) -> Result<(), StatementError>;
    fn step(&mut self, connection: &InnerConnection) -> Result<Option<()>, StatementError>;
    fn column_count(&self) -> i32;
    fn column_name(&self, index: i32) -> String;
    fn column_type(&self, index: i32) -> ColumnType;
    fn column_declared_type(&self, index: i32) -> Option<String>;
    fn column_table_name(&self, index: i32) -> Option<String>;
    fn column_origin_name(&self, index: i32) -> Option<String>;
    fn column_value(&self, index: i32) -> Value;
    fn close(&mut self, connection: &InnerConnection);
}

enum StatementInner {
    #[cfg(not(any(feature = "mysql", feature = "sqlite")))]
    Disabled,
    #[cfg(feature = "mysql")]
    Mysql(crate::mysql::Prepared),
    #[cfg(feature = "sqlite")]
    Sqlite(crate::sqlite::Prepared),
}

impl StatementInner {
    fn backend(&self) -> &dyn PreparedStatement {
        match self {
            #[cfg(feature = "mysql")]
            Self::Mysql(s) => s,
            #[cfg(feature = "sqlite")]
            Self::Sqlite(s) => s,
            #[cfg(not(any(feature = "mysql", feature = "sqlite")))]
            Self::Disabled => unreachable!(),
        }
    }

    fn backend_mut(&mut self) -> &mut dyn PreparedStatement {
        match self {
            #[cfg(feature = "mysql")]
            Self::Mysql(s) => s,
            #[cfg(feature = "sqlite")]
            Self::Sqlite(s) => s,
            #[cfg(not(any(feature = "mysql", feature = "sqlite")))]
            Self::Disabled => unreachable!(),
        }
    }
}

/// A backend-neutral prepared statement without row type information.
pub struct RawStatement {
    inner: StatementInner,
    has_current_row: bool,
    connection: Arc<InnerConnection>,
}

impl RawStatement {
    #[cfg(feature = "mysql")]
    pub(crate) const fn new_mysql(
        statement: crate::mysql::Prepared,
        connection: Arc<InnerConnection>,
    ) -> Self {
        Self {
            inner: StatementInner::Mysql(statement),
            has_current_row: false,
            connection,
        }
    }

    #[cfg(feature = "sqlite")]
    pub(crate) const fn new_sqlite(
        statement: crate::sqlite::Prepared,
        connection: Arc<InnerConnection>,
    ) -> Self {
        Self {
            inner: StatementInner::Sqlite(statement),
            has_current_row: false,
            connection,
        }
    }

    fn assert_column_index(&self, index: i32) {
        assert!(
            (0..self.column_count()).contains(&index),
            "column index {index} is out of range"
        );
    }

    fn assert_current_row(&self) {
        assert!(
            self.has_current_row,
            "column values can only be read while the statement points to a row"
        );
    }

    /// Reset the statement.
    pub fn reset(&mut self) {
        self.has_current_row = false;
        self.inner.backend_mut().reset(&self.connection);
    }

    /// Bind values to the statement.
    pub fn bind(&mut self, params: impl Bind) -> Result<(), StatementError> {
        params.bind(self)
    }

    /// Bind a value by zero-based index.
    pub fn bind_value(&mut self, index: i32, value: Value) -> Result<(), StatementError> {
        self.inner.backend_mut().bind_value(index, value)
    }

    /// Bind a value by parameter name.
    pub fn bind_named_value(&mut self, name: &str, value: Value) -> Result<(), StatementError> {
        self.inner.backend_mut().bind_named_value(name, value)
    }

    /// Advance the statement.
    pub fn step(&mut self) -> Result<Option<()>, StatementError> {
        self.has_current_row = false;
        let row = self.inner.backend_mut().step(&self.connection)?;
        self.has_current_row = row.is_some();
        Ok(row)
    }

    /// Return the column count.
    pub fn column_count(&self) -> i32 {
        self.inner.backend().column_count()
    }

    /// Return a column name.
    pub fn column_name(&self, index: i32) -> String {
        self.assert_column_index(index);
        self.inner.backend().column_name(index)
    }

    /// Return the current column value type.
    pub fn column_type(&self, index: i32) -> ColumnType {
        self.assert_current_row();
        self.assert_column_index(index);
        self.inner.backend().column_type(index)
    }

    /// Return the declared column type.
    pub fn column_declared_type(&self, index: i32) -> Option<String> {
        self.assert_column_index(index);
        self.inner.backend().column_declared_type(index)
    }

    /// Return the source table name.
    pub fn column_table_name(&self, index: i32) -> Option<String> {
        self.assert_column_index(index);
        self.inner.backend().column_table_name(index)
    }

    /// Return the source column name.
    pub fn column_origin_name(&self, index: i32) -> Option<String> {
        self.assert_column_index(index);
        self.inner.backend().column_origin_name(index)
    }

    /// Return a column value from the current row.
    pub fn column_value(&self, index: i32) -> Value {
        self.assert_current_row();
        self.assert_column_index(index);
        self.inner.backend().column_value(index)
    }
}

impl Drop for RawStatement {
    fn drop(&mut self) {
        self.inner.backend_mut().close(&self.connection);
    }
}

/// A typed prepared statement.
pub struct Statement<T: FromRow>(RawStatement, PhantomData<T>);

impl<T: FromRow> Statement<T> {
    #[cfg(feature = "mysql")]
    pub(crate) const fn new_mysql(
        statement: crate::mysql::Prepared,
        connection: Arc<InnerConnection>,
    ) -> Self {
        Self(RawStatement::new_mysql(statement, connection), PhantomData)
    }

    #[cfg(feature = "sqlite")]
    pub(crate) const fn new_sqlite(
        statement: crate::sqlite::Prepared,
        connection: Arc<InnerConnection>,
    ) -> Self {
        Self(RawStatement::new_sqlite(statement, connection), PhantomData)
    }

    /// Reset the statement.
    pub fn reset(&mut self) {
        self.0.reset();
    }
    /// Bind all parameters.
    pub fn bind(&mut self, params: impl Bind) -> Result<(), StatementError> {
        self.0.bind(params)
    }
    /// Bind a parameter by zero-based index.
    pub fn bind_value(
        &mut self,
        index: i32,
        value: impl Into<Value>,
    ) -> Result<(), StatementError> {
        self.0.bind_value(index, value.into())
    }
    /// Bind a parameter by name.
    pub fn bind_named_value(
        &mut self,
        name: &str,
        value: impl Into<Value>,
    ) -> Result<(), StatementError> {
        self.0.bind_named_value(name, value.into())
    }
    /// Advance to the next row.
    pub fn step(&mut self) -> Result<Option<()>, StatementError> {
        self.0.step()
    }
    /// Return the column count.
    pub fn column_count(&self) -> i32 {
        self.0.column_count()
    }
    /// Return a column name.
    pub fn column_name(&self, index: i32) -> String {
        self.0.column_name(index)
    }
    /// Return the current value type.
    pub fn column_type(&self, index: i32) -> ColumnType {
        self.0.column_type(index)
    }
    /// Return a declared column type.
    pub fn column_declared_type(&self, index: i32) -> Option<String> {
        self.0.column_declared_type(index)
    }
    /// Return the source table name.
    pub fn column_table_name(&self, index: i32) -> Option<String> {
        self.0.column_table_name(index)
    }
    /// Return the source column name.
    pub fn column_origin_name(&self, index: i32) -> Option<String> {
        self.0.column_origin_name(index)
    }
    /// Return a value from the current row.
    pub fn column_value(&self, index: i32) -> Value {
        self.0.column_value(index)
    }
}

impl<T: FromRow> Iterator for Statement<T> {
    type Item = Result<T, StatementError>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.step() {
            Ok(Some(())) => Some(
                T::from_row(&mut self.0).map_err(|error| StatementError::new(error.to_string())),
            ),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        }
    }
}
