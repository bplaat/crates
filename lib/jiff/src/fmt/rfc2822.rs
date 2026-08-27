/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::consts::{DAY_NAMES, DAYS_IN_WEEK, MONTH_NAMES, SECS_IN_DAY, SECS_IN_HOUR, SECS_IN_MIN};
use crate::utils::timestamp_to_ymd;
use crate::{Error, Timestamp};

/// A printer for RFC 2822 and RFC 9110 datetimes.
#[derive(Clone, Copy, Debug, Default)]
pub struct DateTimePrinter;

impl DateTimePrinter {
    /// Creates a new datetime printer.
    pub const fn new() -> Self {
        Self
    }

    /// Formats a timestamp as an RFC 9110 HTTP date.
    pub fn timestamp_to_rfc9110_string(&self, timestamp: &Timestamp) -> Result<String, Error> {
        let second = timestamp.total_nanoseconds().div_euclid(1_000_000_000) as i64;
        let (year, month, day) = timestamp_to_ymd(second);
        if year < 0 {
            return Err(Error);
        }
        let weekday = (second.div_euclid(SECS_IN_DAY) + 4).rem_euclid(DAYS_IN_WEEK);
        let day_second = second.rem_euclid(SECS_IN_DAY);
        Ok(format!(
            "{}, {day:02} {} {year:04} {:02}:{:02}:{:02} GMT",
            DAY_NAMES[weekday as usize],
            MONTH_NAMES[month as usize - 1],
            day_second / SECS_IN_HOUR,
            (day_second % SECS_IN_HOUR) / SECS_IN_MIN,
            day_second % SECS_IN_MIN
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_http_date() {
        let timestamp = Timestamp::from_second(1_582_977_600).unwrap();
        assert_eq!(
            DateTimePrinter::new()
                .timestamp_to_rfc9110_string(&timestamp)
                .unwrap(),
            "Sat, 29 Feb 2020 12:00:00 GMT"
        );
    }
}
