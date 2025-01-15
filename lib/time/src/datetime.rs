/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use crate::{
    is_leap_year, Date, ParseError, DAYS_IN_MONTHS, DAYS_IN_MONTHS_LEAP_YEAR, DAY_NAMES,
    MONTH_NAMES,
};

// MARK: DateTime
/// A DateTime in UTC timezone
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DateTime(SystemTime);

impl DateTime {
    /// Create a DateTime with the current date and time
    pub fn now() -> Self {
        Self(SystemTime::now())
    }

    /// Create a DateTime from a timestamp
    pub fn from_timestamp(timestamp: u64) -> Self {
        Self(SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp))
    }

    /// Create a DateTime from year, month, day, hour, minute and second
    pub fn from_ymdhms(
        year: u64,
        month: u64,
        day: u64,
        hour: u64,
        minute: u64,
        second: u64,
    ) -> Option<Self> {
        Some(Self::from_timestamp(
            Date::from_ymd(year, month, day)?.timestamp() + hour * 3600 + minute * 60 + second,
        ))
    }

    /// Get the timestamp of the date and time
    pub fn timestamp(&self) -> u64 {
        self.0
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Should be after unix epoch")
            .as_secs()
    }

    /// Format to RFC 2822 string
    pub fn to_rfc2822(&self) -> String {
        let timestamp = self.timestamp();
        let days_since_epoch = timestamp / 86400;
        let day_in_week = (days_since_epoch + 4) % 7; // 1970-01-01 was a Thursday

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

        format!(
            "{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
            DAY_NAMES[day_in_week as usize],
            day_in_month + 1,
            MONTH_NAMES[month],
            year,
            (timestamp % 86400) / 3600,
            (timestamp % 3600) / 60,
            timestamp % 60
        )
    }
}

impl FromStr for DateTime {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut datetime_parts = s.split('T');
        let date_part = datetime_parts.next().ok_or(ParseError)?;
        let time_part = datetime_parts.next().ok_or(ParseError)?;
        if datetime_parts.next().is_some() {
            return Err(ParseError);
        }

        let mut time_parts = time_part.strip_suffix('Z').ok_or(ParseError)?.split(':');
        let hour: u64 = time_parts
            .next()
            .ok_or(ParseError)?
            .parse()
            .map_err(|_| ParseError)?;
        let minute: u64 = time_parts
            .next()
            .ok_or(ParseError)?
            .parse()
            .map_err(|_| ParseError)?;
        let second: u64 = time_parts
            .next()
            .ok_or(ParseError)?
            .parse()
            .map_err(|_| ParseError)?;
        if time_parts.next().is_some() || hour >= 24 || minute >= 60 || second >= 60 {
            return Err(ParseError);
        }

        Ok(Self::from_timestamp(
            Date::from_str(date_part)?.timestamp() + hour * 3600 + minute * 60 + second,
        ))
    }
}

impl Display for DateTime {
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

        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            year,
            month + 1,
            day_in_month + 1,
            (timestamp % 86400) / 3600,
            (timestamp % 3600) / 60,
            timestamp % 60
        )
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DateTime {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DateTime {
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
        let datetime = DateTime::now();
        assert!(datetime.timestamp() > 0);
    }

    #[test]
    fn test_from_timestamp() {
        let timestamp = 1_000_000;
        let datetime = DateTime::from_timestamp(timestamp);
        assert_eq!(datetime.timestamp(), timestamp);
    }

    #[test]
    fn test_timestamp() {
        let datetime = DateTime::from_timestamp(1_000_000);
        assert_eq!(datetime.timestamp(), 1_000_000);
    }

    #[test]
    fn test_to_rfc2822() {
        let datetime = DateTime::from_timestamp(1_000_000);
        assert_eq!(datetime.to_rfc2822(), "Mon, 12 Jan 1970 13:46:40 GMT");
    }

    #[test]
    fn test_to_rfc2822_leap_year() {
        let datetime = DateTime::from_timestamp(1582977600);
        assert_eq!(datetime.to_rfc2822(), "Sat, 29 Feb 2020 12:00:00 GMT");
    }

    #[test]
    fn test_from_str() {
        let datetime: DateTime = "2019-02-28T12:00:00Z".parse().unwrap();
        assert_eq!(datetime.timestamp(), 1551355200);
    }

    #[test]
    fn test_from_str_leap_year() {
        let datetime: DateTime = "2020-02-29T12:00:00Z".parse().unwrap();
        assert_eq!(datetime.timestamp(), 1582977600);
    }

    #[test]
    fn test_display() {
        let datetime = DateTime::from_timestamp(1551355200);
        assert_eq!(datetime.to_string(), "2019-02-28T12:00:00Z");
    }

    #[test]
    fn test_display_leap_year() {
        let datetime = DateTime::from_timestamp(1582977600);
        assert_eq!(datetime.to_string(), "2020-02-29T12:00:00Z");
    }

    #[test]
    fn test_invalid_parse() {
        let invalid_datetime_str = "invalid-datetime";
        assert!(invalid_datetime_str.parse::<DateTime>().is_err());

        let invalid_datetime_str = "2020-02-29 12:00:00Z"; // Missing 'T'
        assert!(invalid_datetime_str.parse::<DateTime>().is_err());

        let invalid_datetime_str = "2020-02-29T12:00:00T"; // Extra 'T'
        assert!(invalid_datetime_str.parse::<DateTime>().is_err());

        let invalid_datetime_str = "2020-02-30T12:00:00Z"; // Invalid date
        assert!(invalid_datetime_str.parse::<DateTime>().is_err());

        let invalid_datetime_str = "2020-02-29T25:00:00Z"; // Invalid hour
        assert!(invalid_datetime_str.parse::<DateTime>().is_err());

        let invalid_datetime_str = "2020-02-29T12:60:00Z"; // Invalid minute
        assert!(invalid_datetime_str.parse::<DateTime>().is_err());

        let invalid_datetime_str = "2020-02-29T12:00:60Z"; // Invalid second
        assert!(invalid_datetime_str.parse::<DateTime>().is_err());

        let invalid_datetime_str = "2020-02-29T12:00:00"; // Missing 'Z'
        assert!(invalid_datetime_str.parse::<DateTime>().is_err());

        let invalid_datetime_str = "2021-02-29T12:00:00Z"; // Non-leap year
        assert!(invalid_datetime_str.parse::<DateTime>().is_err());

        let invalid_datetime_str = "2019-04-31T12:00:00Z"; // Invalid day in April
        assert!(invalid_datetime_str.parse::<DateTime>().is_err());
    }
}
