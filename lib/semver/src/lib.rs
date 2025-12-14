/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [semver](https://crates.io/crates/semver) crate

#![forbid(unsafe_code)]

use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};

// MARK: Version
/// A semantic version
#[derive(Debug, Clone)]
pub struct Version {
    /// Major number
    pub major: u16,
    /// Minor number
    pub minor: u16,
    /// Patch number
    pub patch: u16,
}

impl Version {
    /// Create a new version
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse a version string
    pub fn parse(version: &str) -> Result<Self, String> {
        let version = version.to_string();
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 1 && parts.len() != 2 && parts.len() != 3 {
            return Err("Invalid semver string".to_string());
        }

        let major = parts[0]
            .parse::<u16>()
            .map_err(|_| "Invalid major version".to_string())?;

        let minor = if parts.len() >= 2 {
            parts[1]
                .parse::<u16>()
                .map_err(|_| "Invalid minor version".to_string())?
        } else {
            0
        };

        let patch = if parts.len() == 3 {
            parts[2]
                .parse::<u16>()
                .map_err(|_| "Invalid patch version".to_string())?
        } else {
            0
        };

        Ok(Self::new(major, minor, patch))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}
impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                ord => ord,
            },
            ord => ord,
        }
    }
}

// MARK: MicrosoftVersion
/// A Microsoft version number
#[derive(Debug, Clone)]
pub struct MicrosoftVersion {
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
    pub fn new(major: u16, minor: u16, build: u16, revision: u16) -> Self {
        Self {
            major,
            minor,
            build,
            revision,
        }
    }

    /// Parse a version string
    pub fn parse(version: &str) -> Result<Self, String> {
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

#[cfg(test)]
mod test {
    use super::*;

    // MARK: Version tests
    #[test]
    fn version_parsing() {
        // valid parsing
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);

        // leading zeros are allowed by parse::<u32>()
        let v2 = Version::parse("01.02.03").unwrap();
        assert_eq!(v2, Version::new(1, 2, 3));

        // missing patch version defaults to 0
        let v3 = Version::parse("1.2").unwrap();
        assert_eq!(v3, Version::new(1, 2, 0));

        // missing minor and patch version defaults to 0
        let v4 = Version::parse("1").unwrap();
        assert_eq!(v4, Version::new(1, 0, 0));

        // invalid inputs
        assert!(Version::parse("1.2.").is_err());
        assert!(Version::parse("1.2.3.4").is_err());
        assert!(Version::parse("a.b.c").is_err());
        assert!(Version::parse("").is_err());
        assert!(Version::parse("1..3").is_err());
    }

    #[test]
    fn display_format() {
        let v = Version::new(1, 10, 5);
        assert_eq!(v.to_string(), "1.10.5");
    }

    #[test]
    fn equality_and_ordering() {
        let a = Version::parse("1.2.3").unwrap();
        let b = Version::parse("1.2.3").unwrap();
        let c = Version::parse("1.2.10").unwrap();
        let d = Version::parse("2.0.0").unwrap();
        let e = Version::parse("0.9.9").unwrap();

        assert_eq!(a, b);
        assert!(c > a);
        assert!(d > c);
        assert!(e < a);
    }

    #[test]
    fn sort_vector_of_versions() {
        let mut vec = vec![
            Version::parse("1.2.3").unwrap(),
            Version::parse("0.9.0").unwrap(),
            Version::parse("1.2.0").unwrap(),
            Version::parse("1.2.10").unwrap(),
        ];
        vec.sort();
        let ordered: Vec<String> = vec.into_iter().map(|v| v.to_string()).collect();
        assert_eq!(ordered, vec!["0.9.0", "1.2.0", "1.2.3", "1.2.10"]);
    }

    // MARK: MicrosoftVersion tests
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
