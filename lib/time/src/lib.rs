/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple UTC DateTime library similar to `chrono`

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::time::SystemTime;

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

// MARK: Constants
pub(crate) const SECS_IN_DAY: i64 = 86400;
pub(crate) static DAYS_IN_MONTHS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
pub(crate) static DAYS_IN_MONTHS_LEAP: [u8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

pub(crate) static MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
pub(crate) static DAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

// MARK: Utils
pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub(crate) fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub(crate) fn timestamp_to_ymd(timestamp: i64) -> (u32, u32, u32) {
    let days_epoch_diff = timestamp.div_euclid(SECS_IN_DAY);
    let mut year = 1970;
    let mut year_day = days_epoch_diff;
    while year_day < 0 || year_day >= if is_leap_year(year) { 366 } else { 365 } {
        if year_day < 0 {
            year -= 1;
            year_day += if is_leap_year(year) { 366 } else { 365 };
        } else {
            year_day -= if is_leap_year(year) { 366 } else { 365 };
            year += 1;
        }
    }

    let days_in_months = if is_leap_year(year) {
        DAYS_IN_MONTHS_LEAP
    } else {
        DAYS_IN_MONTHS
    };
    let mut month = 0;
    let mut day = year_day;
    while day >= days_in_months[month] as i64 {
        day -= days_in_months[month] as i64;
        month += 1;
    }

    (year, month as u32 + 1, day as u32 + 1)
}
