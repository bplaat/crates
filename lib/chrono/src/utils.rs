/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::consts::{
    DAYS_IN_MONTHS, DAYS_IN_MONTHS_LEAP, DAYS_IN_YEAR, DAYS_IN_YEAR_LEAP, EPOCH_YEAR, SECS_IN_DAY,
};

pub(crate) fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

pub(crate) fn days_in_year(year: u32) -> i64 {
    if is_leap_year(year) {
        DAYS_IN_YEAR_LEAP
    } else {
        DAYS_IN_YEAR
    }
}

pub(crate) fn days_in_year_month(year: u32, month: u32) -> i64 {
    let days_in_months = if is_leap_year(year) {
        DAYS_IN_MONTHS_LEAP
    } else {
        DAYS_IN_MONTHS
    };
    days_in_months[(month - 1) as usize] as i64
}

pub(crate) fn timestamp_to_ymd(timestamp: i64) -> (u32, u32, u32) {
    let days_epoch_diff = timestamp.div_euclid(SECS_IN_DAY);
    let mut year = EPOCH_YEAR as u32;
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
