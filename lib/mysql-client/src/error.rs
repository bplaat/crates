/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::{fmt, io};

/// MySQL client error types.
#[derive(Debug)]
pub enum Error {
    /// IO error (connection, reading, writing).
    Io(io::Error),
    /// MySQL server error with error code and message.
    Server {
        /// Error code from MySQL server.
        code: u16,
        /// Error message from MySQL server.
        message: String,
    },
    /// Protocol error (malformed packets, unexpected data).
    Protocol(String),
    /// Authentication failure.
    Auth(String),
    /// Invalid parameter type or value.
    InvalidParameter(String),
    /// No rows returned when one was expected.
    NoRow,
    /// Connection error.
    Connection(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {e}"),
            Error::Server { code, message } => write!(f, "MySQL error [{code}]: {message}"),
            Error::Protocol(msg) => write!(f, "Protocol error: {msg}"),
            Error::Auth(msg) => write!(f, "Authentication error: {msg}"),
            Error::InvalidParameter(msg) => write!(f, "Invalid parameter: {msg}"),
            Error::NoRow => write!(f, "No row found"),
            Error::Connection(msg) => write!(f, "Connection error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

/// Result type alias for mysql-client operations.
pub type Result<T> = std::result::Result<T, Error>;
