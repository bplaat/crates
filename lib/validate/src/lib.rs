/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple struct validation library

use std::collections::HashMap;
use std::error;
use std::fmt::{self, Display, Formatter};

// MARK: Error
/// Validate result
pub type Result = std::result::Result<(), Error>;

/// Validate error
#[derive(Debug)]
pub struct Error {
    message: String,
}
impl Error {
    /// Create validate error
    pub fn new(message: impl AsRef<str>) -> Self {
        Self {
            message: message.as_ref().to_string(),
        }
    }

    /// Get error message
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

// MARK: Report
/// Validation report
#[cfg(feature = "serde")]
#[derive(serde::Serialize)]
pub struct Report(pub HashMap<String, Vec<String>>);

/// Validation report
#[cfg(not(feature = "serde"))]
pub struct Report(pub HashMap<String, Vec<String>>);

// MARK: Validate
/// Validate trait
pub trait Validate {
    /// Validate context
    type Context;

    /// Validate self
    fn validate(&self) -> std::result::Result<(), Report>
    where
        Self::Context: Default,
    {
        let ctx = Self::Context::default();
        self.validate_with(&ctx)
    }

    /// Validate self with context
    fn validate_with(&self, context: &Self::Context) -> std::result::Result<(), Report>;
}

#[cfg(feature = "derive")]
pub use validate_derive::Validate;

#[cfg(feature = "email")]
lazy_static::lazy_static! {
    static ref EMAIL_REGEX: regex::Regex = regex::Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$").expect("Should compile");
}

#[cfg(feature = "email")]
/// Validate email
pub fn is_valid_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}

#[cfg(feature = "url")]
/// Validate url
pub fn is_valid_url(url: &str) -> bool {
    use std::str::FromStr;
    url::Url::from_str(url).is_ok()
}
