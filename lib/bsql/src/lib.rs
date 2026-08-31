/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![allow(unsafe_code)]

pub use crate::bind::Bind;
pub use crate::connection::{Connection, ConnectionError};
pub use crate::from_row::FromRow;
#[cfg(feature = "sqlite")]
pub use crate::sqlite::{preprocess_fts_query, Migration, MigrationError};
pub use crate::statement::{ColumnType, RawStatement, Statement, StatementError};
pub use crate::value::{Value, ValueError};

mod bind;
mod connection;
mod from_row;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "sqlite")]
mod sqlite;
mod statement;
mod value;

#[cfg(all(feature = "derive", feature = "sqlite"))]
pub use bsql_derive::run_migrations;
#[cfg(feature = "derive")]
pub use bsql_derive::{FromRow, FromValue};
