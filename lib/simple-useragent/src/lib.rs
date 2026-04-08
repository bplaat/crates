/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

use regex::Regex;

// MARK: Rules
mod rules_data {
    include!(concat!(env!("OUT_DIR"), "/rules_data.rs"));
}

struct Rules {
    user_agent: Vec<UserAgentRule>,
    os: Vec<OsRule>,
}

struct UserAgentRule {
    regex: Regex,
    family_replacement: Option<&'static str>,
    v1_replacement: Option<&'static str>,
    v2_replacement: Option<&'static str>,
    v3_replacement: Option<&'static str>,
}

struct OsRule {
    regex: Regex,
    os_replacement: Option<&'static str>,
    os_v1_replacement: Option<&'static str>,
    os_v2_replacement: Option<&'static str>,
    os_v3_replacement: Option<&'static str>,
}

impl Rules {
    fn parse() -> Self {
        Self {
            user_agent: rules_data::USER_AGENT_RULES
                .iter()
                .map(|rule| UserAgentRule {
                    regex: Regex::new(rule.regex).expect("Invalid regex"),
                    family_replacement: rule.family_replacement,
                    v1_replacement: rule.v1_replacement,
                    v2_replacement: rule.v2_replacement,
                    v3_replacement: rule.v3_replacement,
                })
                .collect(),
            os: rules_data::OS_RULES
                .iter()
                .map(|rule| OsRule {
                    regex: Regex::new(rule.regex).expect("Invalid regex"),
                    os_replacement: rule.os_replacement,
                    os_v1_replacement: rule.os_v1_replacement,
                    os_v2_replacement: rule.os_v2_replacement,
                    os_v3_replacement: rule.os_v3_replacement,
                })
                .collect(),
        }
    }
}

// MARK: UserAgent
/// User agent
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct UserAgent {
    /// Client
    pub client: Client,
    /// Operating System
    pub os: OS,
}

/// Client
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Client {
    /// Family
    pub family: String,
    /// Version
    pub version: Option<String>,
}

/// Operating System
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OS {
    /// Family
    pub family: String,
    /// Version
    pub version: Option<String>,
}

// MARK: UserAgentParser
/// User agent parser
pub struct UserAgentParser {
    rules: Rules,
}

impl Default for UserAgentParser {
    fn default() -> Self {
        Self {
            rules: Rules::parse(),
        }
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

    // https://github.com/ua-parser/uap-core/blob/master/docs/specification.md#user_agent_parsers
    fn parse_client(&self, user_agent: &str) -> Client {
        for rule in &self.rules.user_agent {
            if let Some(captures) = rule.regex.captures(user_agent) {
                let family = rule
                    .family_replacement
                    .map(|s| Self::map_replacement(s, &captures))
                    .unwrap_or_else(|| captures[1].to_string());
                let major = rule
                    .v1_replacement
                    .map(|s| Self::map_replacement(s, &captures))
                    .or_else(|| captures.get(2).map(|m| m.as_str().to_string()));
                let minor = rule
                    .v2_replacement
                    .map(|s| Self::map_replacement(s, &captures))
                    .or_else(|| captures.get(3).map(|m| m.as_str().to_string()));
                let patch = rule
                    .v3_replacement
                    .map(|s| Self::map_replacement(s, &captures))
                    .or_else(|| captures.get(4).map(|m| m.as_str().to_string()));
                return Client {
                    family,
                    version: Self::concat_version(major, minor, patch),
                };
            }
        }
        Client {
            family: "Other".to_string(),
            version: None,
        }
    }

    // https://github.com/ua-parser/uap-core/blob/master/docs/specification.md#user_agent_parsers
    fn parse_os(&self, user_agent: &str) -> OS {
        for rule in &self.rules.os {
            if let Some(captures) = rule.regex.captures(user_agent) {
                let family = rule
                    .os_replacement
                    .map(|s| Self::map_replacement(s, &captures))
                    .unwrap_or_else(|| captures[1].to_string());
                let major = rule
                    .os_v1_replacement
                    .map(|s| Self::map_replacement(s, &captures))
                    .or_else(|| captures.get(2).map(|m| m.as_str().to_string()));
                let minor = rule
                    .os_v2_replacement
                    .map(|s| Self::map_replacement(s, &captures))
                    .or_else(|| captures.get(3).map(|m| m.as_str().to_string()));
                let patch = rule
                    .os_v3_replacement
                    .map(|s| Self::map_replacement(s, &captures))
                    .or_else(|| captures.get(4).map(|m| m.as_str().to_string()));
                return OS {
                    family,
                    version: Self::concat_version(major, minor, patch),
                };
            }
        }
        OS {
            family: "Other".to_string(),
            version: None,
        }
    }

    fn map_replacement(replacement: &str, captures: &regex::Captures) -> String {
        let mut result = replacement.to_string();
        if result.contains("$1") {
            result = result.replace("$1", &captures[1]);
        }
        if result.contains("$2") {
            result = result.replace("$2", &captures[2]);
        }
        if result.contains("$3") {
            result = result.replace("$3", &captures[3]);
        }
        result
    }

    fn concat_version(
        major: Option<String>,
        minor: Option<String>,
        patch: Option<String>,
    ) -> Option<String> {
        let mut version = String::new();
        if let Some(major) = major {
            version.push_str(&major);
        }
        if let Some(minor) = minor {
            version.push('.');
            version.push_str(&minor);
        }
        if let Some(patch) = patch {
            version.push('.');
            version.push_str(&patch);
        }
        if version.is_empty() {
            None
        } else {
            Some(version)
        }
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parser() {
        let parser = UserAgentParser::new();

        let ua = parser.parse(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:133.0) Gecko/20100101 Firefox/133.0",
        );
        assert_eq!(ua.client.family, "Firefox");
        assert_eq!(ua.client.version.as_deref(), Some("133.0"));
        assert_eq!(ua.os.family, "Mac OS X");
        assert_eq!(ua.os.version.as_deref(), Some("10.15"));

        let ua = parser.parse(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
            );
        assert_eq!(ua.client.family, "Chrome");
        assert_eq!(ua.client.version.as_deref(), Some("91.0.4472"));
        assert_eq!(ua.os.family, "Windows");
        assert_eq!(ua.os.version.as_deref(), Some("10"));

        let ua = parser.parse(
                "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.1 Mobile/15E148 Safari/604.1",
            );
        assert_eq!(ua.client.family, "Mobile Safari");
        assert_eq!(ua.client.version.as_deref(), Some("14.0.1"));
        assert_eq!(ua.os.family, "iOS");
        assert_eq!(ua.os.version.as_deref(), Some("14.6"));

        let ua = parser.parse(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36 Edg/91.0.864.59",
            );
        assert_eq!(ua.client.family, "Edge");
        assert_eq!(ua.client.version.as_deref(), Some("91.0.864"));
        assert_eq!(ua.os.family, "Windows");
        assert_eq!(ua.os.version.as_deref(), Some("10"));

        let ua = parser.parse("UnknownUserAgent/1.0");
        assert_eq!(ua.client.family, "Other");
        assert_eq!(ua.client.version, None);
        assert_eq!(ua.os.family, "Other");
        assert_eq!(ua.os.version, None);
    }
}
