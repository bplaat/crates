/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Display, Formatter};
use std::io;

/// Error returned by MaxMind DB operations.
#[derive(Debug)]
pub enum MaxMindDbError {
    /// The database file is invalid or corrupt.
    InvalidDatabase(String),
    /// An I/O error occurred while reading the database.
    Io(io::Error),
}

impl Display for MaxMindDbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDatabase(msg) => write!(f, "invalid database: {msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
        }
    }
}

impl std::error::Error for MaxMindDbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for MaxMindDbError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
