/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

pub(crate) const EPOCH_YEAR: i64 = 1970;

pub(crate) const DAYS_IN_YEAR: i64 = 365;
pub(crate) const DAYS_IN_YEAR_LEAP: i64 = DAYS_IN_YEAR + 1;
pub(crate) const MONTHS_IN_YEAR: i64 = 12;
pub(crate) const DAYS_IN_WEEK: i64 = 7;

pub(crate) const SECS_IN_MIN: i64 = 60;
pub(crate) const SECS_IN_HOUR: i64 = 60 * SECS_IN_MIN;
pub(crate) const SECS_IN_DAY: i64 = 24 * SECS_IN_HOUR;

pub(crate) static DAYS_IN_MONTHS: [u8; MONTHS_IN_YEAR as usize] =
    [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
pub(crate) static DAYS_IN_MONTHS_LEAP: [u8; MONTHS_IN_YEAR as usize] =
    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

pub(crate) static MONTH_NAMES: [&str; MONTHS_IN_YEAR as usize] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
pub(crate) static DAY_NAMES: [&str; DAYS_IN_WEEK as usize] =
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
