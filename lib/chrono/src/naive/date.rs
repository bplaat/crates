/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Add, Sub};
use std::str::FromStr;
use std::time::Duration;

use crate::utils::{
    DAYS_IN_MONTHS, DAYS_IN_MONTHS_LEAP, SECS_IN_DAY, is_leap_year, timestamp_to_ymd,
};
use crate::{NaiveDateTime, ParseError};

// MARK: NaiveDate
/// A Date
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NaiveDate(i64);

impl NaiveDate {
    pub(crate) fn from_timestamp(secs: i64) -> Self {
        Self(secs - secs.rem_euclid(SECS_IN_DAY))
    }

    /// Create a [NaiveDate] from year, month and day
    pub fn from_ymd_opt(year: u32, month: u32, day: u32) -> Option<Self> {
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

    /// Create a [NaiveDateTime] from date with hour, minute and second
    pub fn and_hms_opt(&self, hour: u32, minute: u32, second: u32) -> Option<NaiveDateTime> {
        let secs = (hour as i64) * 3600 + (minute as i64) * 60 + (second as i64);
        NaiveDateTime::from_timestamp(self.0 + secs, 0)
    }

    /// Get the unix timestamp of the [NaiveDate]
    pub fn timestamp(&self) -> i64 {
        self.0
    }
}

impl Add<Duration> for NaiveDate {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        Self::from_timestamp(self.0 + duration.as_secs() as i64)
    }
}

impl Sub<Duration> for NaiveDate {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self::Output {
        Self::from_timestamp(self.0 - duration.as_secs() as i64)
    }
}

impl FromStr for NaiveDate {
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
        Self::from_ymd_opt(year, month, day).ok_or(ParseError)
    }
}

impl Display for NaiveDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (year, month, day) = timestamp_to_ymd(self.0);
        write!(f, "{:04}-{:02}-{:02}", year, month, day)
    }
}

impl Debug for NaiveDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("NaiveDate(")?;
        Display::fmt(self, f)?;
        f.write_str(")")
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for NaiveDate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for NaiveDate {
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
    fn test_timestamp() {
        let date = NaiveDate::from_timestamp(1609459200);
        assert_eq!(date.timestamp(), 1609459200);
        let date = NaiveDate::from_timestamp(1609459300);
        assert_eq!(date.timestamp(), 1609459200);
        let date = NaiveDate::from_timestamp(-20);
        assert_eq!(date.timestamp(), -SECS_IN_DAY);
    }

    #[test]
    fn test_from_str() {
        let date: NaiveDate = "2021-01-01".parse().unwrap();
        assert_eq!(date.timestamp(), 1609459200);
        let date: NaiveDate = "2020-02-29".parse().unwrap();
        assert_eq!(date.timestamp(), 1582934400);
        let date: NaiveDate = "1969-12-20".parse().unwrap();
        assert_eq!(date.timestamp(), -1036800);
        let date: NaiveDate = "1968-02-29".parse().unwrap();
        assert_eq!(date.timestamp(), -58060800);

        assert!("2019-02-29".parse::<NaiveDate>().is_err());
        assert!("2019-02-29-23".parse::<NaiveDate>().is_err());
        assert!("2019-13-01".parse::<NaiveDate>().is_err());
        assert!("2019-02".parse::<NaiveDate>().is_err());
        assert!("2019--02-29".parse::<NaiveDate>().is_err());
    }

    #[test]
    fn test_display() {
        let date = NaiveDate::from_timestamp(1609459200);
        assert_eq!(date.to_string(), "2021-01-01");
        let date: NaiveDate = NaiveDate::from_timestamp(1582934400);
        assert_eq!(date.to_string(), "2020-02-29");
        let date: NaiveDate = NaiveDate::from_timestamp(-1036800);
        assert_eq!(date.to_string(), "1969-12-20");
        let date: NaiveDate = NaiveDate::from_timestamp(-58060800);
        assert_eq!(date.to_string(), "1968-02-29");
    }

    #[test]
    fn test_add_duration() {
        let date = NaiveDate::from_ymd_opt(1969, 12, 31).unwrap();
        let new_date = date + Duration::from_secs(1);
        assert_eq!(new_date.timestamp(), -SECS_IN_DAY);
        assert_eq!(new_date.to_string(), "1969-12-31");

        let new_date = date + Duration::from_secs(SECS_IN_DAY as u64);
        assert_eq!(new_date.timestamp(), 0);
        assert_eq!(new_date.to_string(), "1970-01-01");
    }

    #[test]
    fn test_sub_duration() {
        let date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let new_date = date - Duration::from_secs(1);
        assert_eq!(new_date.timestamp(), -SECS_IN_DAY);
        assert_eq!(new_date.to_string(), "1969-12-31");
    }
}
