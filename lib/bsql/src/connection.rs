/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;
#[cfg(feature = "mysql")]
use std::sync::Mutex;

use crate::{Bind, FromRow, Statement, StatementError};

pub(crate) enum InnerConnection {
    #[cfg(not(any(feature = "mysql", feature = "sqlite")))]
    Disabled,
    #[cfg(feature = "mysql")]
    Mysql(Mutex<crate::mysql::Client>),
    #[cfg(feature = "sqlite")]
    Sqlite(crate::sqlite::Connection),
}

impl InnerConnection {
    fn execute_script(&self, sql: &str) -> Result<(), StatementError> {
        match self {
            #[cfg(feature = "mysql")]
            Self::Mysql(c) => c.lock().map_err(|_| lock_error())?.execute_script(sql),
            #[cfg(feature = "sqlite")]
            Self::Sqlite(c) => c.execute_script(sql),
            #[cfg(not(any(feature = "mysql", feature = "sqlite")))]
            Self::Disabled => unreachable!(),
        }
    }

    fn prepare<T: FromRow>(self: &Arc<Self>, query: &str) -> Result<Statement<T>, StatementError> {
        match self.as_ref() {
            #[cfg(feature = "mysql")]
            Self::Mysql(c) => Ok(Statement::new_mysql(
                c.lock().map_err(|_| lock_error())?.prepare(query)?,
                Arc::clone(self),
            )),
            #[cfg(feature = "sqlite")]
            Self::Sqlite(c) => Ok(Statement::new_sqlite(c.prepare(query)?, Arc::clone(self))),
            #[cfg(not(any(feature = "mysql", feature = "sqlite")))]
            Self::Disabled => unreachable!(),
        }
    }

    fn begin(&self) -> Result<(), StatementError> {
        match self {
            #[cfg(feature = "mysql")]
            Self::Mysql(c) => c
                .lock()
                .map_err(|_| lock_error())?
                .execute_script("START TRANSACTION"),
            #[cfg(feature = "sqlite")]
            Self::Sqlite(c) => c.begin_transaction(),
            #[cfg(not(any(feature = "mysql", feature = "sqlite")))]
            Self::Disabled => unreachable!(),
        }
    }

    fn finish(&self, commit: bool) -> Result<(), StatementError> {
        match self {
            #[cfg(feature = "mysql")]
            Self::Mysql(c) => c
                .lock()
                .map_err(|_| lock_error())?
                .execute_script(if commit { "COMMIT" } else { "ROLLBACK" }),
            #[cfg(feature = "sqlite")]
            Self::Sqlite(c) => {
                if commit {
                    c.commit()
                } else {
                    c.rollback()
                }
            }
            #[cfg(not(any(feature = "mysql", feature = "sqlite")))]
            Self::Disabled => unreachable!(),
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    fn is_autocommit(&self) -> bool {
        match self {
            #[cfg(feature = "mysql")]
            Self::Mysql(_) => false,
            #[cfg(feature = "sqlite")]
            Self::Sqlite(c) => c.is_autocommit(),
            #[cfg(not(any(feature = "mysql", feature = "sqlite")))]
            Self::Disabled => unreachable!(),
        }
    }

    fn affected_rows(&self) -> u64 {
        match self {
            #[cfg(feature = "mysql")]
            Self::Mysql(c) => c.lock().map_or(0, |c| c.affected_rows),
            #[cfg(feature = "sqlite")]
            Self::Sqlite(c) => c.affected_rows(),
            #[cfg(not(any(feature = "mysql", feature = "sqlite")))]
            Self::Disabled => unreachable!(),
        }
    }

    fn last_insert_row_id(&self) -> u64 {
        match self {
            #[cfg(feature = "mysql")]
            Self::Mysql(c) => c.lock().map_or(0, |c| c.last_insert_id),
            #[cfg(feature = "sqlite")]
            Self::Sqlite(c) => c.last_insert_row_id(),
            #[cfg(not(any(feature = "mysql", feature = "sqlite")))]
            Self::Disabled => unreachable!(),
        }
    }

    #[cfg(feature = "mysql")]
    #[allow(clippy::missing_const_for_fn)]
    pub(crate) fn mysql(&self) -> Result<&Mutex<crate::mysql::Client>, StatementError> {
        match self {
            Self::Mysql(client) => Ok(client),
            #[cfg(feature = "sqlite")]
            Self::Sqlite(_) => Err(StatementError::new("operation requires a MySQL connection")),
        }
    }

    fn rollback_after_error<E>(&self, error: E) -> E
    where
        E: From<StatementError> + Display,
    {
        if self.is_autocommit() {
            return error;
        }
        match self.finish(false) {
            Ok(()) => error,
            Err(rollback_error) => E::from(StatementError::new(format!(
                "{error}; additionally failed to roll back transaction: {rollback_error}"
            ))),
        }
    }
}

#[cfg(feature = "mysql")]
fn lock_error() -> StatementError {
    StatementError::new("MySQL connection lock is poisoned")
}

/// A connection error.
#[derive(Debug)]
pub struct ConnectionError {
    msg: String,
}

impl ConnectionError {
    pub(crate) fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Connection error: {}", self.msg)
    }
}
impl Error for ConnectionError {}

struct RollbackOnDrop<'a> {
    connection: &'a InnerConnection,
    armed: bool,
}
impl Drop for RollbackOnDrop<'_> {
    fn drop(&mut self) {
        if self.armed && !self.connection.is_autocommit() {
            _ = self.connection.finish(false);
        }
    }
}

/// A database connection.
#[derive(Clone)]
pub struct Connection {
    inner: Arc<InnerConnection>,
}

impl Connection {
    pub(crate) fn from_inner(inner: InnerConnection) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    #[cfg(feature = "sqlite")]
    pub(crate) fn require_sqlite(&self) -> Result<&crate::sqlite::Connection, StatementError> {
        match self.inner.as_ref() {
            InnerConnection::Sqlite(c) => Ok(c),
            #[cfg(feature = "mysql")]
            InnerConnection::Mysql(_) => Err(StatementError::new(
                "operation requires a SQLite connection",
            )),
        }
    }

    #[cfg(feature = "sqlite")]
    pub(crate) fn is_sqlite(&self) -> bool {
        #[cfg(feature = "sqlite")]
        if matches!(self.inner.as_ref(), InnerConnection::Sqlite(_)) {
            return true;
        }
        false
    }

    /// Prepare a statement.
    pub fn prepare<T: FromRow>(
        &self,
        query: impl AsRef<str>,
    ) -> Result<Statement<T>, StatementError> {
        self.inner.prepare(query.as_ref())
    }

    /// Prepare and bind a query.
    pub fn query<T: FromRow>(
        &self,
        query: impl AsRef<str>,
        params: impl Bind,
    ) -> Result<Statement<T>, StatementError> {
        let mut statement = self.prepare::<T>(query)?;
        statement.bind(params)?;
        Ok(statement)
    }

    /// Read the first returned row.
    pub fn query_some<T: FromRow>(
        &self,
        query: impl AsRef<str>,
        params: impl Bind,
    ) -> Result<T, StatementError> {
        self.query::<T>(query, params)?
            .next()
            .transpose()?
            .ok_or_else(|| StatementError::new("expected at least one row from query"))
    }

    /// Execute a SQL script.
    pub fn execute_script(&self, sql: &str) -> Result<(), StatementError> {
        self.inner.execute_script(sql)
    }

    /// Execute a prepared query.
    pub fn execute(&self, query: impl AsRef<str>, params: impl Bind) -> Result<(), StatementError> {
        self.query::<()>(query, params)?.next().transpose()?;
        Ok(())
    }

    /// Return the number of affected rows.
    pub fn affected_rows(&self) -> u64 {
        self.inner.affected_rows()
    }

    /// Return the last inserted row identifier.
    pub fn last_insert_row_id(&self) -> u64 {
        self.inner.last_insert_row_id()
    }

    /// Run a closure inside a transaction.
    pub fn transaction<T, E>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<StatementError> + Display,
    {
        self.inner.begin().map_err(E::from)?;
        let mut rollback = RollbackOnDrop {
            connection: &self.inner,
            armed: true,
        };
        match operation(self) {
            Ok(value) => match self.inner.finish(true) {
                Ok(()) => {
                    rollback.armed = false;
                    Ok(value)
                }
                Err(error) => {
                    rollback.armed = false;
                    Err(self.inner.rollback_after_error(E::from(error)))
                }
            },
            Err(error) => {
                rollback.armed = false;
                Err(self.inner.rollback_after_error(error))
            }
        }
    }
}

/// Run a query with named arguments.
#[macro_export]
macro_rules! query_args {
    ($t:tt, $db:expr, $query:expr, Args { $($key:ident : $value:expr),* $(,)? } $(,)?) => {{
        (|| -> std::result::Result<_, $crate::StatementError> {
            let mut statement = $db.prepare::<$t>($query)?;
            $(statement.bind_named_value(concat!(":", stringify!($key)), Into::<$crate::Value>::into($value))?;)*
            Ok(statement)
        })()
    }};
}

/// Execute a query with named arguments.
#[macro_export]
macro_rules! execute_args {
    ($db:expr, $query:expr, Args { $($key:ident : $value:expr),* $(,)? } $(,)?) => {{
        (|| -> std::result::Result<_, $crate::StatementError> {
            let mut statement = $db.prepare::<()>($query)?;
            $(statement.bind_named_value(concat!(":", stringify!($key)), Into::<$crate::Value>::into($value))?;)*
            statement.next().transpose()?;
            Ok(())
        })()
    }};
}
