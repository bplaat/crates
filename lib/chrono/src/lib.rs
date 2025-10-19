/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [chrono](https://crates.io/crates/chrono) crate

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub use datetime::DateTime;
pub use naive::date::NaiveDate;
pub use naive::datetime::NaiveDateTime;
pub use timezone::{TimeZone, Utc};

mod consts;
mod datetime;
mod naive;
mod timezone;
mod utils;

// MARK: ParseError
/// Parser error
#[derive(Debug)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "URL parse error")
    }
}

impl Error for ParseError {}
