/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple UTC DateTime extension of `std::time` similar to `chrono`

use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub use date::Date;
pub use datetime::DateTime;

mod date;
mod datetime;

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

// MARK: Utils
pub(crate) const DAYS_IN_MONTHS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
pub(crate) const DAYS_IN_MONTHS_LEAP_YEAR: [u8; 12] =
    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

pub(crate) const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
pub(crate) const DAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

pub(crate) fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
