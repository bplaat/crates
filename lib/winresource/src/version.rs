/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};

/// A Microsoft version number
#[derive(Debug, Clone)]
pub(crate) struct MicrosoftVersion {
    /// Major versnumberion
    pub major: u16,
    /// Minor number
    pub minor: u16,
    /// Build number
    pub build: u16,
    /// Revision number
    pub revision: u16,
}

impl MicrosoftVersion {
    /// Create a new Microsoft version
    pub(crate) fn new(major: u16, minor: u16, build: u16, revision: u16) -> Self {
        Self {
            major,
            minor,
            build,
            revision,
        }
    }

    /// Parse a version string
    pub(crate) fn parse(version: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 4 {
            return Err("Invalid semver string".to_string());
        }

        let major = parts[0]
            .parse::<u16>()
            .map_err(|_| "Invalid major version".to_string())?;
        let minor = parts[1]
            .parse::<u16>()
            .map_err(|_| "Invalid minor version".to_string())?;
        let build = parts[2]
            .parse::<u16>()
            .map_err(|_| "Invalid build version".to_string())?;
        let revision = parts[3]
            .parse::<u16>()
            .map_err(|_| "Invalid revision version".to_string())?;
        Ok(Self::new(major, minor, build, revision))
    }
}

impl Display for MicrosoftVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.build, self.revision
        )
    }
}

impl PartialEq for MicrosoftVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.build == other.build
            && self.revision == other.revision
    }
}
impl Eq for MicrosoftVersion {}

impl PartialOrd for MicrosoftVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for MicrosoftVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => match self.build.cmp(&other.build) {
                    Ordering::Equal => self.revision.cmp(&other.revision),
                    ord => ord,
                },
                ord => ord,
            },
            ord => ord,
        }
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn microsoft_version_parsing() {
        // valid parsing
        let v = MicrosoftVersion::parse("1.2.3.4").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.build, 3);
        assert_eq!(v.revision, 4);

        // leading zeros are allowed
        let v2 = MicrosoftVersion::parse("01.02.003.0004").unwrap();
        assert_eq!(v2, MicrosoftVersion::new(1, 2, 3, 4));

        // invalid inputs
        assert!(MicrosoftVersion::parse("1.2.3").is_err());
        assert!(MicrosoftVersion::parse("1.2.3.4.5").is_err());
        assert!(MicrosoftVersion::parse("a.b.c.d").is_err());
        assert!(MicrosoftVersion::parse("").is_err());
        assert!(MicrosoftVersion::parse("1..3.4").is_err());
    }

    #[test]
    fn microsoft_display_format() {
        let v = MicrosoftVersion::new(10, 0, 2, 5);
        assert_eq!(v.to_string(), "10.0.2.5");
    }

    #[test]
    fn microsoft_equality_and_ordering() {
        let a = MicrosoftVersion::parse("1.2.3.4").unwrap();
        let b = MicrosoftVersion::parse("1.2.3.4").unwrap();
        let c = MicrosoftVersion::parse("1.2.4.0").unwrap();
        let d = MicrosoftVersion::parse("2.0.0.0").unwrap();
        let e = MicrosoftVersion::parse("1.1.255.255").unwrap();

        assert_eq!(a, b);
        assert!(c > a);
        assert!(d > c);
        assert!(e < a);
    }

    #[test]
    fn sort_vector_of_microsoft_versions() {
        let mut vec = vec![
            MicrosoftVersion::parse("1.2.3.4").unwrap(),
            MicrosoftVersion::parse("0.9.0.1").unwrap(),
            MicrosoftVersion::parse("1.2.10.0").unwrap(),
            MicrosoftVersion::parse("1.0.0.0").unwrap(),
        ];
        vec.sort();
        let ordered: Vec<String> = vec.into_iter().map(|v| v.to_string()).collect();
        assert_eq!(ordered, vec!["0.9.0.1", "1.0.0.0", "1.2.3.4", "1.2.10.0"]);
    }
}
