/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A SQLite Rust library

pub use crate::connection::{Connection, ConnectionError};
pub use crate::statement::{Bind, FromRow, RawStatement, Statement};
pub use crate::value::{Value, ValueError};

mod connection;
mod statement;
mod value;

#[cfg(feature = "derive")]
pub use sqlite_derive::FromRow;
