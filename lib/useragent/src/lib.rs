/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A basic user agent parser library
//!
//! Regexes are copied from https://github.com/ua-parser/uap-core

use regex::Regex;
use serde::{Deserialize, Deserializer};

const REGEXES: &[u8] = include_bytes!("../regexes.yaml");

// MARK: UserAgent
/// User agent
#[derive(Debug)]
pub struct UserAgent {
    /// Client
    pub client: Client,
    /// Operating System
    pub os: OS,
}

/// Client
#[derive(Debug)]
pub struct Client {
    /// Family
    pub family: String,
    /// Major version
    pub major: Option<String>,
    /// Minor version
    pub minor: Option<String>,
    /// Patch version
    pub patch: Option<String>,
}

/// Operating System
#[derive(Debug)]
pub struct OS {
    /// Family
    pub family: String,
    /// Major version
    pub major: Option<String>,
    /// Minor version
    pub minor: Option<String>,
    /// Patch version
    pub patch: Option<String>,
    /// Patch minor version
    pub patch_minor: Option<String>,
}

// MARK: UserAgentParser
/// User agent parser
#[derive(Deserialize)]
pub struct UserAgentParser {
    user_agent_parsers: Vec<Parser>,
    os_parsers: Vec<Parser>,
}

#[derive(Deserialize)]
struct Parser {
    #[serde(deserialize_with = "string_as_regex")]
    regex: Regex,
    family_replacement: Option<String>,
    v1_replacement: Option<String>,
    v2_replacement: Option<String>,
    v3_replacement: Option<String>,
    os_replacement: Option<String>,
    os_v1_replacement: Option<String>,
    os_v2_replacement: Option<String>,
    os_v3_replacement: Option<String>,
    os_v4_replacement: Option<String>,
}

fn string_as_regex<'de, D>(deserializer: D) -> Result<Regex, D::Error>
where
    D: Deserializer<'de>,
{
    let str = String::deserialize(deserializer)?;
    Ok(Regex::new(&str).expect("Invalid regex"))
}

impl Default for UserAgentParser {
    fn default() -> Self {
        serde_yaml::from_slice(REGEXES).expect("Invalid regexes")
    }
}

impl UserAgentParser {
    /// Create new user agent parser
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse user agent
    pub fn parse(&self, user_agent: &str) -> UserAgent {
        UserAgent {
            client: self.parse_client(user_agent),
            os: self.parse_os(user_agent),
        }
    }

    fn parse_client(&self, user_agent: &str) -> Client {
        for parser in &self.user_agent_parsers {
            if let Some(captures) = parser.regex.captures(user_agent) {
                let family = parser
                    .family_replacement
                    .clone()
                    .unwrap_or_else(|| captures[1].to_string());
                let major = parser
                    .v1_replacement
                    .clone()
                    .or_else(|| captures.get(2).map(|m| m.as_str().to_string()));
                let minor = parser
                    .v2_replacement
                    .clone()
                    .or_else(|| captures.get(3).map(|m| m.as_str().to_string()));
                let patch = parser
                    .v3_replacement
                    .clone()
                    .or_else(|| captures.get(4).map(|m| m.as_str().to_string()));
                return Client {
                    family,
                    major,
                    minor,
                    patch,
                };
            }
        }
        Client {
            family: "Other".to_string(),
            major: None,
            minor: None,
            patch: None,
        }
    }

    fn parse_os(&self, user_agent: &str) -> OS {
        for parser in &self.os_parsers {
            if let Some(captures) = parser.regex.captures(user_agent) {
                let family = parser
                    .os_replacement
                    .clone()
                    .unwrap_or_else(|| captures[1].to_string());
                let major = parser
                    .os_v1_replacement
                    .clone()
                    .or_else(|| captures.get(2).map(|m| m.as_str().to_string()));
                let minor = parser
                    .os_v2_replacement
                    .clone()
                    .or_else(|| captures.get(3).map(|m| m.as_str().to_string()));
                let patch = parser
                    .os_v3_replacement
                    .clone()
                    .or_else(|| captures.get(4).map(|m| m.as_str().to_string()));
                let patch_minor = parser
                    .os_v4_replacement
                    .clone()
                    .or_else(|| captures.get(5).map(|m| m.as_str().to_string()));
                return OS {
                    family,
                    major,
                    minor,
                    patch,
                    patch_minor,
                };
            }
        }
        OS {
            family: "Other".to_string(),
            major: None,
            minor: None,
            patch: None,
            patch_minor: None,
        }
    }
}

// MARK: Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let parser = UserAgentParser::new();

        let ua = parser.parse(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:133.0) Gecko/20100101 Firefox/133.0",
        );
        assert_eq!(ua.client.family, "Firefox");
        assert_eq!(ua.client.major, Some("133".to_string()));
        assert_eq!(ua.os.family, "Mac OS X");
        assert_eq!(ua.os.major, Some("10".to_string()));
        assert_eq!(ua.os.minor, Some("15".to_string()));

        let ua = parser.parse(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
            );
        assert_eq!(ua.client.family, "Chrome");
        assert_eq!(ua.client.major, Some("91".to_string()));
        assert_eq!(ua.client.minor, Some("0".to_string()));
        assert_eq!(ua.client.patch, Some("4472".to_string()));
        assert_eq!(ua.os.family, "Windows");
        assert_eq!(ua.os.major, Some("10".to_string()));
        assert_eq!(ua.os.minor, None);

        let ua = parser.parse(
                "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.1 Mobile/15E148 Safari/604.1",
            );
        assert_eq!(ua.client.family, "Mobile Safari");
        assert_eq!(ua.client.major, Some("14".to_string()));
        assert_eq!(ua.client.minor, Some("0".to_string()));
        assert_eq!(ua.client.patch, Some("1".to_string()));
        assert_eq!(ua.os.family, "iOS");
        assert_eq!(ua.os.major, Some("14".to_string()));
        assert_eq!(ua.os.minor, Some("6".to_string()));

        let ua = parser.parse(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36 Edg/91.0.864.59",
            );
        assert_eq!(ua.client.family, "Edge");
        assert_eq!(ua.client.major, Some("91".to_string()));
        assert_eq!(ua.client.minor, Some("0".to_string()));
        assert_eq!(ua.client.patch, Some("864".to_string()));
        assert_eq!(ua.os.family, "Windows");
        assert_eq!(ua.os.major, Some("10".to_string()));
        assert_eq!(ua.os.minor, None);

        let ua = parser.parse("UnknownUserAgent/1.0");
        assert_eq!(ua.client.family, "Other");
        assert_eq!(ua.client.major, None);
        assert_eq!(ua.client.minor, None);
        assert_eq!(ua.client.patch, None);
        assert_eq!(ua.os.family, "Other");
        assert_eq!(ua.os.major, None);
        assert_eq!(ua.os.minor, None);
        assert_eq!(ua.os.patch, None);
        assert_eq!(ua.os.patch_minor, None);
    }
}
