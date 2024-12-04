/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error;
use std::fmt::{self, Display, Formatter};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SqliteError: {}", self.message)
    }
}

impl error::Error for Error {}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::new(&msg.to_string())
    }
}
