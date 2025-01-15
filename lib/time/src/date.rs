/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use crate::{is_leap_year, ParseError, DAYS_IN_MONTHS, DAYS_IN_MONTHS_LEAP_YEAR};

// MARK: Date
/// A Date
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Date(SystemTime);

impl Date {
    /// Create a Date with the current date
    pub fn now() -> Self {
        Self(SystemTime::now())
    }

    /// Create a Date from a timestamp
    pub fn from_timestamp(timestamp: u64) -> Self {
        Self(SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp))
    }

    /// Get the timestamp of the date and time
    pub fn timestamp(&self) -> u64 {
        self.0
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Should be after unix epoch")
            .as_secs()
    }
}

impl FromStr for Date {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');
        let year: u64 = parts
            .next()
            .ok_or(ParseError)?
            .parse()
            .map_err(|_| ParseError)?;
        let month: u64 = parts
            .next()
            .ok_or(ParseError)?
            .parse()
            .map_err(|_| ParseError)?;
        let day: u64 = parts
            .next()
            .ok_or(ParseError)?
            .parse()
            .map_err(|_| ParseError)?;

        let days_in_months = if is_leap_year(year) {
            DAYS_IN_MONTHS_LEAP_YEAR
        } else {
            DAYS_IN_MONTHS
        };
        if parts.next().is_some()
            || !(1..=12).contains(&month)
            || !(1..=days_in_months[(month - 1) as usize]).contains(&(day as u8))
        {
            return Err(ParseError);
        }

        let mut days_since_epoch = 0;
        for year in 1970..year {
            days_since_epoch += if is_leap_year(year) { 366 } else { 365 };
        }
        for moth in 0..(month - 1) {
            days_since_epoch += days_in_months[moth as usize] as u64;
        }
        days_since_epoch += day - 1;

        Ok(Self::from_timestamp(days_since_epoch * 86400))
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let timestamp = self.timestamp();
        let days_since_epoch = timestamp / 86400;

        let mut year = 1970;
        let mut day_in_year = days_since_epoch;
        while day_in_year >= if is_leap_year(year) { 366 } else { 365 } {
            day_in_year -= if is_leap_year(year) { 366 } else { 365 };
            year += 1;
        }

        let days_in_months = if is_leap_year(year) {
            DAYS_IN_MONTHS_LEAP_YEAR
        } else {
            DAYS_IN_MONTHS
        };
        let mut month = 0;
        let mut day_in_month = day_in_year;
        while day_in_month >= days_in_months[month] as u64 {
            day_in_month -= days_in_months[month] as u64;
            month += 1;
        }

        write!(f, "{:04}-{:02}-{:02}", year, month + 1, day_in_month + 1)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Date {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Date {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_now() {
        let date = Date::now();
        assert!(date.timestamp() > 0);
    }

    #[test]
    fn test_from_timestamp() {
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let date = Date::from_timestamp(timestamp);
        assert_eq!(date.to_string(), "2021-01-01");
    }

    #[test]
    fn test_timestamp() {
        let date = Date::from_timestamp(1609459200); // 2021-01-01 00:00:00 UTC
        assert_eq!(date.timestamp(), 1609459200);
    }

    #[test]
    fn test_from_str() {
        let date: Date = "2021-01-01".parse().unwrap();
        assert_eq!(date.to_string(), "2021-01-01");
    }

    #[test]
    fn test_invalid_from_str() {
        let date: Result<Date, _> = "invalid-date".parse();
        assert!(date.is_err());
    }

    #[test]
    fn test_leap_year() {
        let date: Date = "2020-02-29".parse().unwrap();
        assert_eq!(date.to_string(), "2020-02-29");
    }

    #[test]
    fn test_non_leap_year() {
        let date: Result<Date, _> = "2019-02-29".parse();
        assert!(date.is_err());
    }
}
