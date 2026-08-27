/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use core::fmt::{self, Debug, Display, Formatter};
use core::ops::{Add, Sub};
use core::str::FromStr;
use core::time::Duration;

use crate::Error;
use crate::civil::DateTime;
use crate::consts::{EPOCH_YEAR, MONTHS_IN_YEAR, SECS_IN_DAY};
use crate::utils::{days_in_year, days_in_year_month, timestamp_to_ymd};

/// A date in the Gregorian calendar.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date(i64);

impl Date {
    /// The minimum representable date.
    pub const MIN: Self = Self(-377_705_116_800);

    /// The maximum representable date.
    pub const MAX: Self = Self(253_402_214_400);

    /// The zero date, `0000-01-01`.
    pub const ZERO: Self = Self(-62_167_219_200);

    pub(crate) const fn from_second(second: i64) -> Self {
        Self(second - second.rem_euclid(SECS_IN_DAY))
    }

    /// Creates a date from its year, month and day components.
    pub fn new(year: i16, month: i8, day: i8) -> Result<Self, Error> {
        if !(-9999..=9999).contains(&year)
            || !(1..=MONTHS_IN_YEAR as i8).contains(&month)
            || !(1..=days_in_year_month(year, month) as i8).contains(&day)
        {
            return Err(Error);
        }

        let mut days_from_epoch = 0;
        if year >= EPOCH_YEAR as i16 {
            for year in (EPOCH_YEAR as i16)..year {
                days_from_epoch += days_in_year(year);
            }
        } else {
            for year in (year..(EPOCH_YEAR as i16)).rev() {
                days_from_epoch -= days_in_year(year);
            }
        }
        for month in 1..month {
            days_from_epoch += days_in_year_month(year, month);
        }
        days_from_epoch += i64::from(day) - 1;

        Ok(Self::from_second(days_from_epoch * SECS_IN_DAY))
    }

    /// Creates a datetime on this date.
    pub const fn at(self, hour: i8, minute: i8, second: i8, nanosecond: i32) -> DateTime {
        assert!(hour >= 0 && hour < 24, "hour is valid");
        assert!(minute >= 0 && minute < 60, "minute is valid");
        assert!(second >= 0 && second < 60, "second is valid");
        assert!(
            nanosecond >= 0 && nanosecond < 1_000_000_000,
            "nanosecond is valid"
        );
        DateTime::from_date_unchecked(self, hour, minute, second, nanosecond)
    }

    /// Returns the year component.
    pub fn year(self) -> i16 {
        timestamp_to_ymd(self.0).0
    }

    /// Returns the month component.
    pub fn month(self) -> i8 {
        timestamp_to_ymd(self.0).1
    }

    /// Returns the day component.
    pub fn day(self) -> i8 {
        timestamp_to_ymd(self.0).2
    }

    pub(crate) const fn as_second(self) -> i64 {
        self.0
    }
}

impl Default for Date {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add<Duration> for Date {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        let days = (duration.as_secs() / SECS_IN_DAY as u64) as i64;
        let seconds = days
            .checked_mul(SECS_IN_DAY)
            .expect("adding duration to date overflowed");
        let second = self
            .0
            .checked_add(seconds)
            .expect("adding duration to date overflowed");
        assert!(second <= Self::MAX.0, "adding duration to date overflowed");
        Self::from_second(second)
    }
}

impl Sub<Duration> for Date {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self::Output {
        let days = (duration.as_secs() / SECS_IN_DAY as u64) as i64;
        let seconds = days
            .checked_mul(SECS_IN_DAY)
            .expect("subtracting duration from date overflowed");
        let second = self
            .0
            .checked_sub(seconds)
            .expect("subtracting duration from date overflowed");
        assert!(
            second >= Self::MIN.0,
            "subtracting duration from date overflowed"
        );
        Self::from_second(second)
    }
}

impl FromStr for Date {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.rsplitn(3, '-');
        let day = parts.next().ok_or(Error)?.parse().map_err(|_| Error)?;
        let month = parts.next().ok_or(Error)?.parse().map_err(|_| Error)?;
        let year = parts.next().ok_or(Error)?.parse().map_err(|_| Error)?;
        if parts.next().is_some() {
            return Err(Error);
        }
        Self::new(year, month, day)
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (year, month, day) = timestamp_to_ymd(self.0);
        if year < 0 {
            write!(f, "-{:06}-{month:02}-{day:02}", -i32::from(year))
        } else {
            write!(f, "{year:04}-{month:02}-{day:02}")
        }
    }
}

impl Debug for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
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
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_parsing_formatting_and_arithmetic() {
        let date: Date = "2020-02-29".parse().unwrap();
        assert_eq!(date.to_string(), "2020-02-29");
        assert_eq!(date.year(), 2020);
        assert_eq!(date.month(), 2);
        assert_eq!(date.day(), 29);
        assert_eq!(Date::new(-9999, 1, 1).unwrap().to_string(), "-009999-01-01");
        assert_eq!(
            (date + Duration::from_secs(SECS_IN_DAY as u64)).to_string(),
            "2020-03-01"
        );
        assert_eq!((date - Duration::from_secs(1)).to_string(), "2020-02-29");
    }

    #[test]
    fn date_rejects_invalid_values() {
        assert!(Date::new(2019, 2, 29).is_err());
        assert!("2019-13-01".parse::<Date>().is_err());
        assert!("2019-02".parse::<Date>().is_err());
    }
}
