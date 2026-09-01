/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![allow(unsafe_code)]

#[cfg(not(any(feature = "mysql", feature = "sqlite")))]
compile_error!("bsql requires the `mysql` or `sqlite` feature");

#[cfg(any(feature = "mysql", feature = "sqlite"))]
pub use crate::bind::Bind;
#[cfg(any(feature = "mysql", feature = "sqlite"))]
pub use crate::connection::{Connection, ConnectionError, ExecutionResult, PoolOptions};
#[cfg(any(feature = "mysql", feature = "sqlite"))]
pub use crate::from_row::FromRow;
#[cfg(feature = "mysql")]
pub use crate::mysql::MysqlTransport;
#[cfg(feature = "sqlite")]
pub use crate::sqlite::{preprocess_fts_query, Migration, MigrationError, SqliteMode};
#[cfg(any(feature = "mysql", feature = "sqlite"))]
pub use crate::statement::{ColumnType, RawStatement, Statement, StatementError};
#[cfg(any(feature = "mysql", feature = "sqlite"))]
pub use crate::value::{Value, ValueError};

#[cfg(any(feature = "mysql", feature = "sqlite"))]
mod bind;
#[cfg(any(feature = "mysql", feature = "sqlite"))]
mod connection;
#[cfg(any(feature = "mysql", feature = "sqlite"))]
mod from_row;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(any(feature = "mysql", feature = "sqlite"))]
mod statement;
#[cfg(any(feature = "mysql", feature = "sqlite"))]
mod value;

#[cfg(all(feature = "derive", feature = "sqlite"))]
pub use bsql_derive::run_migrations;
#[cfg(all(feature = "derive", any(feature = "mysql", feature = "sqlite")))]
pub use bsql_derive::{FromRow, FromValue};
