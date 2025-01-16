/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Sub};
use std::str::FromStr;
use std::time::Duration;

use crate::{now, timestamp_to_ymd, Date, ParseError, DAY_NAMES, MONTH_NAMES, SECS_IN_DAY};

// MARK: DateTime
/// A DateTime in UTC timezone
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DateTime(i64);

impl DateTime {
    /// Create a DateTime with the current date and time
    pub fn now() -> Self {
        Self(now() as i64)
    }

    /// Create a DateTime from a timestamp
    pub fn from_timestamp(timestamp: i64) -> Self {
        Self(timestamp)
    }

    /// Create a DateTime from year, month, day, hour, minute and second
    pub fn from_ymdhms(
        year: u32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Option<Self> {
        Some(Self::from_timestamp(
            Date::from_ymd(year, month, day)?.timestamp()
                + (hour as i64) * 3600
                + (minute as i64) * 60
                + (second as i64),
        ))
    }

    /// Get the timestamp of the date and time
    pub fn timestamp(&self) -> i64 {
        self.0
    }

    /// Format to RFC 2822 string
    pub fn to_rfc2822(&self) -> String {
        let (year, month, day) = timestamp_to_ymd(self.0);
        let week_day = (self.0.div_euclid(SECS_IN_DAY) + 4).rem_euclid(7); // 1970-01-01 was a Thursday
        let day_sec = self.0.rem_euclid(SECS_IN_DAY);
        format!(
            "{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
            DAY_NAMES[week_day as usize],
            day,
            MONTH_NAMES[month as usize - 1],
            year,
            day_sec / 3600,
            (day_sec % 3600) / 60,
            day_sec % 60
        )
    }
}

impl Add<Duration> for DateTime {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        Self(self.0 + duration.as_secs() as i64)
    }
}

impl Sub<Duration> for DateTime {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self::Output {
        Self(self.0 - duration.as_secs() as i64)
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
        let hour: u32 = time_parts
            .next()
            .ok_or(ParseError)?
            .parse()
            .map_err(|_| ParseError)?;
        let minute: u32 = time_parts
            .next()
            .ok_or(ParseError)?
            .parse()
            .map_err(|_| ParseError)?;
        let second: u32 = time_parts
            .next()
            .ok_or(ParseError)?
            .parse()
            .map_err(|_| ParseError)?;
        if time_parts.next().is_some() || hour >= 24 || minute >= 60 || second >= 60 {
            return Err(ParseError);
        }

        Ok(Self::from_timestamp(
            Date::from_str(date_part)?.timestamp()
                + (hour as i64) * 3600
                + (minute as i64) * 60
                + (second as i64),
        ))
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (year, month, day) = timestamp_to_ymd(self.0);
        let day_sec = self.0.rem_euclid(SECS_IN_DAY);
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            year,
            month,
            day,
            day_sec / 3600,
            (day_sec % 3600) / 60,
            day_sec % 60
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
    fn test_timestamp() {
        let datetime = DateTime::from_timestamp(1609459345);
        assert_eq!(datetime.timestamp(), 1609459345);
    }

    #[test]
    fn test_to_rfc2822() {
        let datetime = DateTime::from_timestamp(1000000);
        assert_eq!(datetime.to_rfc2822(), "Mon, 12 Jan 1970 13:46:40 GMT");
        let datetime = DateTime::from_timestamp(1582977600);
        assert_eq!(datetime.to_rfc2822(), "Sat, 29 Feb 2020 12:00:00 GMT");
        let datetime = DateTime::from_timestamp(-1000000);
        assert_eq!(datetime.to_rfc2822(), "Sat, 20 Dec 1969 10:13:20 GMT");
    }

    #[test]
    fn test_from_str() {
        let datetime: DateTime = "2019-02-28T12:00:00Z".parse().unwrap();
        assert_eq!(datetime.timestamp(), 1551355200);
        let datetime: DateTime = "2020-02-29T12:00:00Z".parse().unwrap();
        assert_eq!(datetime.timestamp(), 1582977600);
        let datetime: DateTime = "1969-12-20T10:13:20Z".parse().unwrap();
        assert_eq!(datetime.timestamp(), -1000000);

        assert!("invalid-datetime".parse::<DateTime>().is_err());
        assert!("2020-02-29 12:00:00Z".parse::<DateTime>().is_err());
        assert!("2020-02-29T12:00:00T".parse::<DateTime>().is_err());
        assert!("2020-02-30T12:00:00Z".parse::<DateTime>().is_err());
        assert!("2020-02-29T25:00:00Z".parse::<DateTime>().is_err());
        assert!("2020-02-29T12:60:00Z".parse::<DateTime>().is_err());
        assert!("2020-02-29T12:00:60Z".parse::<DateTime>().is_err());
        assert!("2020-02-29T12:00:00".parse::<DateTime>().is_err());
        assert!("2021-02-29T12:00:00Z".parse::<DateTime>().is_err());
        assert!("2019-04-31T12:00:00Z".parse::<DateTime>().is_err());
    }

    #[test]
    fn test_display() {
        let datetime = DateTime::from_timestamp(1551355200);
        assert_eq!(datetime.to_string(), "2019-02-28T12:00:00Z");
        let datetime = DateTime::from_timestamp(1582977600);
        assert_eq!(datetime.to_string(), "2020-02-29T12:00:00Z");
        let datetime = DateTime::from_timestamp(-1000000);
        assert_eq!(datetime.to_string(), "1969-12-20T10:13:20Z");
    }

    #[test]
    fn test_add_duration() {
        let datetime = DateTime::from_timestamp(1609459200);
        let new_datetime = datetime + Duration::from_secs(1);
        assert_eq!(new_datetime.timestamp(), 1609459201);
    }

    #[test]
    fn test_sub_duration() {
        let datetime = DateTime::from_timestamp(1609459200);
        let new_datetime = datetime - Duration::from_secs(1);
        assert_eq!(new_datetime.timestamp(), 1609459199);
    }
}
