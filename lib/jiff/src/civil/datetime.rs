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
use crate::civil::Date;
use crate::consts::{SECS_IN_DAY, SECS_IN_HOUR, SECS_IN_MIN};
use crate::utils::{parse_fraction, timestamp_to_ymd};

const NANOS_PER_SECOND: i128 = 1_000_000_000;
const MIN_TOTAL_NANOSECONDS: i128 = -377_705_116_800_i128 * NANOS_PER_SECOND;
const MAX_TOTAL_NANOSECONDS: i128 = 253_402_300_799_i128 * NANOS_PER_SECOND + 999_999_999;
const ZERO_TOTAL_NANOSECONDS: i128 = -62_167_219_200_i128 * NANOS_PER_SECOND;

/// A civil datetime without a time zone.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime(i128);

impl DateTime {
    /// The minimum representable civil datetime.
    pub const MIN: Self = Self(MIN_TOTAL_NANOSECONDS);

    /// The maximum representable civil datetime.
    pub const MAX: Self = Self(MAX_TOTAL_NANOSECONDS);

    /// The zero civil datetime, `0000-01-01T00:00:00`.
    pub const ZERO: Self = Self(ZERO_TOTAL_NANOSECONDS);

    /// Creates a civil datetime from its components.
    pub fn new(
        year: i16,
        month: i8,
        day: i8,
        hour: i8,
        minute: i8,
        second: i8,
        nanosecond: i32,
    ) -> Result<Self, Error> {
        Self::from_date(
            Date::new(year, month, day)?,
            hour,
            minute,
            second,
            nanosecond,
        )
    }

    pub(crate) fn from_date(
        date: Date,
        hour: i8,
        minute: i8,
        second: i8,
        nanosecond: i32,
    ) -> Result<Self, Error> {
        if !(0..24).contains(&hour)
            || !(0..60).contains(&minute)
            || !(0..60).contains(&second)
            || !(0..1_000_000_000).contains(&nanosecond)
        {
            return Err(Error);
        }
        Ok(Self::from_date_unchecked(
            date, hour, minute, second, nanosecond,
        ))
    }

    pub(crate) const fn from_date_unchecked(
        date: Date,
        hour: i8,
        minute: i8,
        second: i8,
        nanosecond: i32,
    ) -> Self {
        let whole_seconds = date.as_second()
            + hour as i64 * SECS_IN_HOUR
            + minute as i64 * SECS_IN_MIN
            + second as i64;
        Self(whole_seconds as i128 * NANOS_PER_SECOND + nanosecond as i128)
    }

    pub(crate) const fn from_total_nanoseconds_unchecked(total: i128) -> Self {
        Self(total)
    }

    fn from_total_nanoseconds(total: i128) -> Result<Self, Error> {
        if !(MIN_TOTAL_NANOSECONDS..=MAX_TOTAL_NANOSECONDS).contains(&total) {
            return Err(Error);
        }
        Ok(Self(total))
    }

    /// Returns the date component.
    pub const fn date(self) -> Date {
        Date::from_second(self.whole_second())
    }

    /// Returns the hour component.
    pub const fn hour(self) -> i8 {
        (self.whole_second().rem_euclid(SECS_IN_DAY) / SECS_IN_HOUR) as i8
    }

    /// Returns the minute component.
    pub const fn minute(self) -> i8 {
        ((self.whole_second().rem_euclid(SECS_IN_DAY) % SECS_IN_HOUR) / SECS_IN_MIN) as i8
    }

    /// Returns the second component.
    pub const fn second(self) -> i8 {
        (self.whole_second().rem_euclid(SECS_IN_MIN)) as i8
    }

    /// Returns the fractional nanosecond component.
    pub const fn subsec_nanosecond(self) -> i32 {
        self.0.rem_euclid(NANOS_PER_SECOND) as i32
    }

    pub(crate) const fn total_nanoseconds(self) -> i128 {
        self.0
    }

    const fn whole_second(self) -> i64 {
        self.0.div_euclid(NANOS_PER_SECOND) as i64
    }

    pub(crate) fn fmt_with_suffix(&self, f: &mut Formatter<'_>, suffix: &str) -> fmt::Result {
        let whole_second = self.whole_second();
        let (year, month, day) = timestamp_to_ymd(whole_second);
        if year < 0 {
            write!(f, "-{:06}", -i32::from(year))?;
        } else {
            write!(f, "{year:04}")?;
        }
        write!(
            f,
            "-{month:02}-{day:02}T{:02}:{:02}:{:02}",
            self.hour(),
            self.minute(),
            self.second()
        )?;
        let mut fraction = self.subsec_nanosecond();
        if fraction != 0 {
            let mut width = 9;
            while fraction % 10 == 0 {
                fraction /= 10;
                width -= 1;
            }
            write!(f, ".{fraction:0width$}")?;
        }
        f.write_str(suffix)
    }
}

impl Default for DateTime {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add<Duration> for DateTime {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        let total = self
            .0
            .checked_add(duration.as_nanos() as i128)
            .expect("adding duration to datetime overflowed");
        Self::from_total_nanoseconds(total).expect("adding duration to datetime overflowed")
    }
}

impl Sub<Duration> for DateTime {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self::Output {
        let total = self
            .0
            .checked_sub(duration.as_nanos() as i128)
            .expect("subtracting duration from datetime overflowed");
        Self::from_total_nanoseconds(total).expect("subtracting duration from datetime overflowed")
    }
}

impl FromStr for DateTime {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (date, time) = value.split_once(['T', ' ']).ok_or(Error)?;
        let mut parts = time.split(':');
        let hour = parts.next().ok_or(Error)?.parse().map_err(|_| Error)?;
        let minute = parts.next().ok_or(Error)?.parse().map_err(|_| Error)?;
        let second_and_fraction = parts.next().ok_or(Error)?;
        if parts.next().is_some() {
            return Err(Error);
        }
        let (second, nanosecond) = match second_and_fraction.split_once('.') {
            Some((second, fraction)) => (
                second.parse().map_err(|_| Error)?,
                parse_fraction(fraction).ok_or(Error)?,
            ),
            None => (second_and_fraction.parse().map_err(|_| Error)?, 0),
        };
        Self::from_date(date.parse()?, hour, minute, second, nanosecond)
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_with_suffix(f, "")
    }
}

impl Debug for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
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
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datetime_parsing_formatting_and_arithmetic() {
        let datetime: DateTime = "1969-12-20 10:13:20.123".parse().unwrap();
        assert_eq!(datetime.to_string(), "1969-12-20T10:13:20.123");
        assert_eq!(datetime.hour(), 10);
        assert_eq!(datetime.minute(), 13);
        assert_eq!(datetime.second(), 20);
        assert_eq!(datetime.subsec_nanosecond(), 123_000_000);
        assert_eq!(
            (datetime + Duration::from_nanos(1)).to_string(),
            "1969-12-20T10:13:20.123000001"
        );
    }

    #[test]
    fn datetime_rejects_invalid_values() {
        assert!(DateTime::new(2020, 2, 30, 12, 0, 0, 0).is_err());
        assert!(DateTime::new(2020, 2, 29, 25, 0, 0, 0).is_err());
        assert!(DateTime::new(2020, 2, 29, 12, 0, 0, 1_000_000_000).is_err());
        assert!("2020-02-29T12:00".parse::<DateTime>().is_err());
    }
}
