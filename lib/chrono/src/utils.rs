/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::consts::{
    DAYS_IN_MONTHS, DAYS_IN_MONTHS_LEAP, DAYS_IN_YEAR, DAYS_IN_YEAR_LEAP, EPOCH_YEAR, SECS_IN_DAY,
};

pub(crate) const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub(crate) const fn days_in_year(year: i32) -> i64 {
    if is_leap_year(year) {
        DAYS_IN_YEAR_LEAP
    } else {
        DAYS_IN_YEAR
    }
}

pub(crate) const fn days_in_year_month(year: i32, month: u32) -> i64 {
    let days_in_months = if is_leap_year(year) {
        DAYS_IN_MONTHS_LEAP
    } else {
        DAYS_IN_MONTHS
    };
    days_in_months[(month - 1) as usize] as i64
}

pub(crate) const fn timestamp_to_ymd(timestamp: i64) -> (i32, u32, u32) {
    let days_epoch_diff = timestamp.div_euclid(SECS_IN_DAY);
    let mut year = EPOCH_YEAR as i32;
    let mut year_day = days_epoch_diff;
    while year_day < 0 || year_day >= days_in_year(year) {
        if year_day < 0 {
            year -= 1;
            year_day += days_in_year(year);
        } else {
            year_day -= days_in_year(year);
            year += 1;
        }
    }

    let mut month = 1;
    let mut day = year_day;
    while day >= days_in_year_month(year, month) {
        day -= days_in_year_month(year, month);
        month += 1;
    }

    (year, month, day as u32 + 1)
}
