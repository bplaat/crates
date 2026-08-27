/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use core::fmt::{self, Debug, Display, Formatter};

use crate::civil::DateTime;
use crate::{Error, Timestamp};

/// A fixed UTC offset.
#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Offset;

impl Offset {
    /// The UTC offset.
    pub const UTC: Self = Self;

    /// Converts a timestamp to a UTC civil datetime.
    pub const fn to_datetime(self, timestamp: Timestamp) -> DateTime {
        timestamp.to_civil()
    }

    /// Converts a UTC civil datetime to a timestamp.
    pub const fn to_timestamp(self, datetime: DateTime) -> Result<Timestamp, Error> {
        Timestamp::from_total_nanoseconds(datetime.total_nanoseconds())
    }
}

impl Display for Offset {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("+00")
    }
}

impl Debug for Offset {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("00:00:00")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_offset_matches_upstream_formatting() {
        assert_eq!(Offset::UTC.to_string(), "+00");
        assert_eq!(format!("{:?}", Offset::UTC), "00:00:00");
    }
}
