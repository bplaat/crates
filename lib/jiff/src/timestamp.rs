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

const NANOS_PER_SECOND: i128 = 1_000_000_000;
const MIN_SECOND: i64 = -377_705_023_201;
const MAX_SECOND: i64 = 253_402_207_200;
const MAX_NANOSECOND: i32 = 999_999_999;
const MIN_NANOSECOND: i32 = -999_999_999;
const MIN_TOTAL_NANOSECONDS: i128 = MIN_SECOND as i128 * NANOS_PER_SECOND;
const MAX_TOTAL_NANOSECONDS: i128 = MAX_SECOND as i128 * NANOS_PER_SECOND + MAX_NANOSECOND as i128;

/// An instant in time represented as nanoseconds since the Unix epoch.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(i128);

impl Timestamp {
    /// The minimum representable timestamp.
    pub const MIN: Self = Self(MIN_TOTAL_NANOSECONDS);

    /// The maximum representable timestamp.
    pub const MAX: Self = Self(MAX_TOTAL_NANOSECONDS);

    /// The Unix epoch, `1970-01-01T00:00:00Z`.
    pub const UNIX_EPOCH: Self = Self(0);

    /// Creates a timestamp from seconds and fractional nanoseconds since the Unix epoch.
    pub const fn new(second: i64, nanosecond: i32) -> Result<Self, Error> {
        if second < MIN_SECOND
            || second > MAX_SECOND
            || nanosecond < MIN_NANOSECOND
            || nanosecond > MAX_NANOSECOND
        {
            return Err(Error);
        }
        Self::from_total_nanoseconds(second as i128 * NANOS_PER_SECOND + nanosecond as i128)
    }

    /// Creates a timestamp from seconds since the Unix epoch.
    pub const fn from_second(second: i64) -> Result<Self, Error> {
        Self::new(second, 0)
    }

    /// Returns the current system time as a timestamp.
    pub fn now() -> Self {
        let now = std::time::SystemTime::now();
        let total_nanoseconds = match now.duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos() as i128,
            Err(error) => -(error.duration().as_nanos() as i128),
        };
        Self::from_total_nanoseconds(total_nanoseconds)
            .expect("system time is in the supported range")
    }

    /// Returns the number of whole seconds since the Unix epoch, truncated toward zero.
    pub const fn as_second(self) -> i64 {
        (self.0 / NANOS_PER_SECOND) as i64
    }

    /// Returns the fractional nanosecond component.
    pub const fn subsec_nanosecond(self) -> i32 {
        (self.0 % NANOS_PER_SECOND) as i32
    }

    pub(crate) const fn total_nanoseconds(self) -> i128 {
        self.0
    }

    pub(crate) const fn from_total_nanoseconds(total: i128) -> Result<Self, Error> {
        if total < MIN_TOTAL_NANOSECONDS || total > MAX_TOTAL_NANOSECONDS {
            return Err(Error);
        }
        Ok(Self(total))
    }

    pub(crate) const fn to_civil(self) -> DateTime {
        DateTime::from_total_nanoseconds_unchecked(self.0)
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::UNIX_EPOCH
    }
}

impl Add<Duration> for Timestamp {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        let total = self
            .0
            .checked_add(duration.as_nanos() as i128)
            .expect("adding duration to timestamp overflowed");
        Self::from_total_nanoseconds(total).expect("adding duration to timestamp overflowed")
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self::Output {
        let total = self
            .0
            .checked_sub(duration.as_nanos() as i128)
            .expect("subtracting duration from timestamp overflowed");
        Self::from_total_nanoseconds(total).expect("subtracting duration from timestamp overflowed")
    }
}

impl FromStr for Timestamp {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let datetime = value.strip_suffix('Z').ok_or(Error)?.parse::<DateTime>()?;
        Self::from_total_nanoseconds(datetime.total_nanoseconds())
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_civil().fmt_with_suffix(f, "Z")
    }
}

impl Debug for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Timestamp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_parsing_formatting_and_arithmetic() {
        let timestamp: Timestamp = "2020-02-29T12:00:00.123456789Z".parse().unwrap();
        assert_eq!(timestamp.as_second(), 1_582_977_600);
        assert_eq!(timestamp.subsec_nanosecond(), 123_456_789);
        assert_eq!(timestamp.to_string(), "2020-02-29T12:00:00.123456789Z");
        assert_eq!(
            (timestamp + Duration::from_nanos(1)).to_string(),
            "2020-02-29T12:00:00.12345679Z"
        );
    }

    #[test]
    fn timestamp_normalizes_signed_nanoseconds() {
        let timestamp = Timestamp::new(-5, 123_456_789).unwrap();
        assert_eq!(timestamp.as_second(), -4);
        assert_eq!(timestamp.subsec_nanosecond(), -876_543_211);
        assert_eq!(timestamp.to_string(), "1969-12-31T23:59:55.123456789Z");
    }

    #[test]
    fn timestamp_matches_upstream_bounds() {
        assert_eq!(Timestamp::MIN.to_string(), "-009999-01-02T01:59:59Z");
        assert_eq!(Timestamp::MAX.to_string(), "9999-12-30T22:00:00.999999999Z");
        assert!(Timestamp::from_second(MIN_SECOND - 1).is_err());
        assert!(Timestamp::from_second(MAX_SECOND + 1).is_err());
    }

    #[test]
    fn timestamp_rejects_invalid_values() {
        assert!(Timestamp::new(0, 1_000_000_000).is_err());
        assert!("2020-02-30T12:00:00Z".parse::<Timestamp>().is_err());
        assert!("2020-02-29T12:00:00".parse::<Timestamp>().is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn timestamp_serde_roundtrip_preserves_nanoseconds() {
        let timestamp = Timestamp::new(1_582_977_600, 123_456_789).unwrap();
        let json = serde_json::to_string(&timestamp).unwrap();
        assert_eq!(json, "\"2020-02-29T12:00:00.123456789Z\"");
        assert_eq!(serde_json::from_str::<Timestamp>(&json).unwrap(), timestamp);
    }
}
