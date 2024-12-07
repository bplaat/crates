/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::BTreeMap;
use std::error;
use std::fmt::{self, Display, Formatter};

// MARK: Error
pub type Result = std::result::Result<(), Error>;

#[derive(Debug)]
pub struct Error {
    message: String,
}
impl Error {
    pub fn new(message: impl AsRef<str>) -> Self {
        Self {
            message: message.as_ref().to_string(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Validate error: {}", self.message)
    }
}
impl error::Error for Error {}

// MARK: Errors
#[cfg(feature = "serde")]
#[derive(serde::Serialize)]
pub struct Errors(pub BTreeMap<String, Vec<String>>);

#[cfg(not(feature = "serde"))]
pub struct Errors(pub BTreeMap<String, Vec<String>>);

// MARK: Validate
pub trait Validate {
    type Context;

    fn validate(&self) -> std::result::Result<(), Errors>
    where
        Self::Context: Default,
    {
        let ctx = Self::Context::default();
        self.validate_with(&ctx)
    }

    fn validate_with(&self, context: &Self::Context) -> std::result::Result<(), Errors>;
}

#[cfg(feature = "derive")]
pub use validate_derive::Validate;

#[cfg(feature = "email")]
pub fn is_valid_email(email: &str) -> bool {
    let re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(email)
}
