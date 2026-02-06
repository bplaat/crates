/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![forbid(unsafe_code)]

//! A minimal MySQL client library with support for basic CRUD operations and prepared statements.
//!
//! # Features
//!
//! - TCP connections to MySQL 8.0+ servers
//! - Basic CRUD queries (SELECT, INSERT, UPDATE, DELETE)
//! - Prepared statements
//! - Simple and ergonomic API
//! - Zero external dependencies (uses only std library and workspace's sha1)
//!
//! # Example
//!
//! ```no_run
//! use mysql_client::{Connection, Value};
//!
//! # fn main() -> mysql_client::error::Result<()> {
//! let mut conn = Connection::connect("localhost", 3306, "user", "password", "database")?;
//!
//! // Execute a query
//! let rows = conn.query("SELECT id, name FROM users WHERE id = 1")?;
//! for row in rows {
//!     println!("ID: {:?}, Name: {:?}", row[0], row[1]);
//! }
//! # Ok(())
//! # }
//! ```

/// MySQL connection and authentication.
pub mod connection;
/// Error types for MySQL client operations.
pub mod error;
/// MySQL protocol packet handling.
pub mod protocol;
/// Type definitions for MySQL protocol.
pub mod types;
/// Value type and conversions.
pub mod value;

pub use connection::Connection;
pub use error::{Error, Result};
pub use types::{CapabilityFlags, ColumnDefinition, ColumnType, StatusFlags};
pub use value::{Row, Value};
