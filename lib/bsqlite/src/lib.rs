/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

pub use crate::bind::Bind;
pub use crate::connection::{Connection, ConnectionError, OpenMode};
pub use crate::from_row::FromRow;
pub use crate::statement::{ColumnType, RawStatement, Statement};
pub use crate::value::{Value, ValueError};

mod bind;
mod connection;
mod from_row;
mod statement;
mod value;

#[cfg(feature = "derive")]
pub use bsqlite_derive::{FromRow, FromValue};
