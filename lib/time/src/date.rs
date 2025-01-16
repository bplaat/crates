/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Sub};
use std::str::FromStr;
use std::time::Duration;

use crate::{
    is_leap_year, now, timestamp_to_ymd, ParseError, DAYS_IN_MONTHS, DAYS_IN_MONTHS_LEAP,
    SECS_IN_DAY,
};

// MARK: Date
/// A Date
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Date(i64);

impl Date {
    /// Create a Date with the current date
    pub fn now() -> Self {
        Self::from_timestamp(now() as i64)
    }

    /// Create a Date from a unix timestamp, rounds to nearest day
    pub fn from_timestamp(timestamp: i64) -> Self {
        Self(timestamp - timestamp.rem_euclid(SECS_IN_DAY))
    }

    /// Create a Date from year, month and day
    pub fn from_ymd(year: u32, month: u32, day: u32) -> Option<Self> {
        let days_in_months = if is_leap_year(year) {
            DAYS_IN_MONTHS_LEAP
        } else {
            DAYS_IN_MONTHS
        };
        if !(1..=12).contains(&month)
            || !(1..=days_in_months[(month - 1) as usize]).contains(&(day as u8))
        {
            return None;
        }

        let mut days_epoch_diff = 0;
        if year >= 1970 {
            for year in 1970..year {
                days_epoch_diff += if is_leap_year(year) { 366 } else { 365 };
            }
        } else {
            for year in (year..1970).rev() {
                days_epoch_diff -= if is_leap_year(year) { 366 } else { 365 };
            }
        }
        for month in 0..(month - 1) {
            days_epoch_diff += days_in_months[month as usize] as i64;
        }
        days_epoch_diff += day as i64 - 1;

        Some(Self::from_timestamp(days_epoch_diff * SECS_IN_DAY))
    }

    /// Get the unix timestamp of the date
    pub fn timestamp(&self) -> i64 {
        self.0
    }
}

impl Add<Duration> for Date {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        Self::from_timestamp(self.0 + duration.as_secs() as i64)
    }
}

impl Sub<Duration> for Date {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self::Output {
        Self::from_timestamp(self.0 - duration.as_secs() as i64)
    }
}

impl FromStr for Date {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');
        let year: u32 = parts
            .next()
            .ok_or(ParseError)?
            .parse()
            .map_err(|_| ParseError)?;
        let month: u32 = parts
            .next()
            .ok_or(ParseError)?
            .parse()
            .map_err(|_| ParseError)?;
        let day: u32 = parts
            .next()
            .ok_or(ParseError)?
            .parse()
            .map_err(|_| ParseError)?;
        if parts.next().is_some() {
            return Err(ParseError);
        }
        Self::from_ymd(year, month, day).ok_or(ParseError)
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (year, month, day) = timestamp_to_ymd(self.0);
        write!(f, "{:04}-{:02}-{:02}", year, month, day)
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
    fn test_timestamp() {
        let date = Date::from_timestamp(1609459200);
        assert_eq!(date.timestamp(), 1609459200);
        let date = Date::from_timestamp(1609459300);
        assert_eq!(date.timestamp(), 1609459200);
        let date = Date::from_timestamp(-20);
        assert_eq!(date.timestamp(), -SECS_IN_DAY);
    }

    #[test]
    fn test_from_str() {
        let date: Date = "2021-01-01".parse().unwrap();
        assert_eq!(date.timestamp(), 1609459200);
        let date: Date = "2020-02-29".parse().unwrap();
        assert_eq!(date.timestamp(), 1582934400);
        let date: Date = "1969-12-20".parse().unwrap();
        assert_eq!(date.timestamp(), -1036800);
        let date: Date = "1968-02-29".parse().unwrap();
        assert_eq!(date.timestamp(), -58060800);

        assert!("2019-02-29".parse::<Date>().is_err());
        assert!("2019-02-29-23".parse::<Date>().is_err());
        assert!("2019-13-01".parse::<Date>().is_err());
        assert!("2019-02".parse::<Date>().is_err());
        assert!("2019--02-29".parse::<Date>().is_err());
    }

    #[test]
    fn test_display() {
        let date = Date::from_timestamp(1609459200);
        assert_eq!(date.to_string(), "2021-01-01");
        let date: Date = Date::from_timestamp(1582934400);
        assert_eq!(date.to_string(), "2020-02-29");
        let date: Date = Date::from_timestamp(-1036800);
        assert_eq!(date.to_string(), "1969-12-20");
        let date: Date = Date::from_timestamp(-58060800);
        assert_eq!(date.to_string(), "1968-02-29");
    }

    #[test]
    fn test_add_duration() {
        let date = Date::from_ymd(1969, 12, 31).unwrap();
        let new_date = date + Duration::from_secs(1);
        assert_eq!(new_date.timestamp(), -SECS_IN_DAY);
        assert_eq!(new_date.to_string(), "1969-12-31");

        let new_date = date + Duration::from_secs(SECS_IN_DAY as u64);
        assert_eq!(new_date.timestamp(), 0);
        assert_eq!(new_date.to_string(), "1970-01-01");
    }

    #[test]
    fn test_sub_duration() {
        let date = Date::from_ymd(1970, 1, 1).unwrap();
        let new_date = date - Duration::from_secs(1);
        assert_eq!(new_date.timestamp(), -SECS_IN_DAY);
        assert_eq!(new_date.to_string(), "1969-12-31");
    }
}
