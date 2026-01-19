/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [uuid](https://crates.io/crates/url) crate

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

// MARK: URL
/// Url
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Url {
    scheme: String,
    authority: Option<Authority>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Authority {
    userinfo: Option<String>,
    host: String,
    port: Option<u16>,
}

impl Url {
    /// Parse a new URL from a string
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        Self::from_str(s)
    }

    /// Get the URL scheme
    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    /// Get the URL authority
    pub fn userinfo(&self) -> Option<&str> {
        self.authority
            .as_ref()
            .and_then(|auth| auth.userinfo.as_deref())
    }

    /// Get the URL host
    pub fn host(&self) -> Option<&str> {
        self.authority.as_ref().map(|auth| auth.host.as_str())
    }

    /// Get the URL domain
    pub fn domain(&self) -> Option<&str> {
        self.authority.as_ref().map(|auth| auth.host.as_str())
    }

    /// Get the URL port
    pub fn port(&self) -> Option<u16> {
        self.authority.as_ref().and_then(|auth| auth.port)
    }

    /// Get the URL path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the URL query
    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    /// Get the URL fragment
    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }
}

impl FromStr for Url {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("://").collect();

        if parts.len() != 2 {
            return Err(ParseError("No scheme found".to_string()));
        }

        let scheme = parts[0].to_string();
        let mut path = parts[1];
        if scheme.is_empty() || path.is_empty() {
            return Err(ParseError("Scheme or path are empty".to_string()));
        }

        let mut authority = None;
        if let Some(idx) = path.find('/') {
            authority = Some(path[..idx].to_string());
            path = &path[idx..];
        }

        let mut query = None;
        if let Some(idx) = path.find('?') {
            query = Some(path[idx + 1..].to_string());
            path = &path[..idx];
        }

        let mut fragment = None;
        if let Some(idx) = path.find('#') {
            fragment = Some(path[idx + 1..].to_string());
            path = &path[..idx];
        }

        let authority = if let Some(authority) = authority {
            let mut authority = authority.as_str();
            let mut userinfo = None;
            if let Some(idx) = authority.find('@') {
                userinfo = Some(authority[..idx].to_string());
                authority = &authority[idx + 1..];
            }

            let mut host = authority;
            let mut port = None;
            if let Some(idx) = authority.find(':') {
                host = &authority[..idx];
                port = Some(
                    authority[idx + 1..]
                        .parse()
                        .map_err(|_| ParseError("Can't parse port".to_string()))?,
                );
                if let Some(port) = port
                    && port == 0
                {
                    return Err(ParseError("Port cannot be 0".to_string()));
                }
            }

            Some(Authority {
                userinfo,
                host: host.to_string(),
                port,
            })
        } else {
            None
        };

        Ok(Url {
            scheme,
            authority,
            path: path.to_string(),
            query,
            fragment,
        })
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}://", self.scheme)?;
        if let Some(authority) = &self.authority {
            if let Some(userinfo) = &authority.userinfo {
                write!(f, "{userinfo}@")?;
            }
            write!(f, "{}", authority.host)?;
            if let Some(port) = authority.port {
                write!(f, ":{port}")?;
            }
        }
        write!(f, "{}", self.path)?;
        if let Some(query) = &self.query {
            write!(f, "?{query}")?;
        }
        if let Some(fragment) = &self.fragment {
            write!(f, "#{fragment}")?;
        }
        Ok(())
    }
}

// MARK: ParseError
/// Url parser error
#[derive(Debug)]
pub struct ParseError(String);

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "URL parse error: {}", self.0)
    }
}

impl Error for ParseError {}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_correct() {
        let urls = [
            "http://example.com",
            "http://example.com/",
            "http://example.com/?",
            "http://example.com/#",
            "http://example.com/?#",
            "http://example.com/path",
            "http://example.com/path/",
            "http://example.com/path/?",
            "http://example.com/path/#",
            "http://example.com/path/?#",
            "http://example.com/path?query",
            "http://example.com/path/?query",
            "http://example.com/path#fragment",
            "http://example.com/path/#fragment",
            "http://example.com/path?query#fragment",
            "http://example.com/path/?query#fragment",
            "http://user:pass@example.com",
            "http://user:pass@example.com/",
            "http://user:pass@example.com/path",
            "http://user:pass@example.com/path?query",
            "http://user:pass@example.com/path#fragment",
            "http://user:pass@example.com/path?query#fragment",
            "http://example.com:8080",
            "http://example.com:8080/",
            "http://example.com:8080/path",
            "http://example.com:8080/path?query",
            "http://example.com:8080/path#fragment",
            "http://example.com:8080/path?query#fragment",
            "https://user:pass@example.com",
            "https://user:pass@example.com/",
            "https://user:pass@example.com/path",
            "https://user:pass@example.com/path?query",
            "https://user:pass@example.com/path#fragment",
            "https://user:pass@example.com/path?query#fragment",
            "https://example.com:8080",
            "https://example.com:8080/",
            "https://example.com:8080/path",
            "https://example.com:8080/path?query",
            "https://example.com:8080/path#fragment",
            "httsp://example.com:8080/path?query#fragment",
            "ws://example.com/",
            "wss://example.com/",
            "ws://example.com:8080/",
            "wss://example.com:8080/",
            "ws://example.com/path",
            "wss://example.com/path",
            "ws://example.com/path?query",
            "wss://example.com/path?query",
            "ws://example.com/path#fragment",
            "wss://example.com/path#fragment",
        ];
        for url in &urls {
            assert!(Url::from_str(url).is_ok());
        }
    }

    #[test]
    fn test_parse_invalid() {
        let invalid_urls = [
            "http://",
            "://example.com",
            "http://example.com:abc/",
            "http://example.com:0/",
            "http://example.com:999999/",
            "http://example.com:65536/",
            "http://example.com:-1/",
            "http://example.com:1a2b3c/",
            "http://example.com:/",
            "http://example.com:/path",
            "http://example.com:/path/",
            "http://example.com:/path?query",
            "http://example.com:/path#fragment",
            "http://example.com:/path?query#fragment",
            "https://",
            "https://example.com:abc/",
            "https://example.com:0/",
            "https://example.com:999999/",
            "https://example.com:65536/",
            "https://example.com:-1/",
            "https://example.com:1a2b3c/",
            "https://example.com:/",
            "https://example.com:/path",
            "https://example.com:/path/",
            "https://example.com:/path?query",
            "https://example.com:/path#fragment",
            "https://example.com:/path?query#fragment",
        ];
        for url in &invalid_urls {
            assert!(Url::from_str(url).is_err());
        }
    }

    #[test]
    fn test_display() {
        let urls = [
            ("http://example.com", "http://example.com"),
            ("http://example.com/", "http://example.com/"),
            ("http://example.com/path", "http://example.com/path"),
            (
                "http://example.com/path?query",
                "http://example.com/path?query",
            ),
            (
                "http://example.com/path#fragment",
                "http://example.com/path#fragment",
            ),
            (
                "http://example.com/path?query#fragment",
                "http://example.com/path?query#fragment",
            ),
            (
                "http://user:pass@example.com",
                "http://user:pass@example.com",
            ),
            (
                "http://user:pass@example.com/path",
                "http://user:pass@example.com/path",
            ),
            (
                "http://user:pass@example.com/path?query",
                "http://user:pass@example.com/path?query",
            ),
            (
                "http://user:pass@example.com/path#fragment",
                "http://user:pass@example.com/path#fragment",
            ),
            (
                "http://user:pass@example.com/path?query#fragment",
                "http://user:pass@example.com/path?query#fragment",
            ),
            ("http://example.com:8080", "http://example.com:8080"),
            (
                "http://example.com:8080/path",
                "http://example.com:8080/path",
            ),
            (
                "http://example.com:8080/path?query",
                "http://example.com:8080/path?query",
            ),
            (
                "http://example.com:8080/path#fragment",
                "http://example.com:8080/path#fragment",
            ),
            (
                "http://example.com:8080/path?query#fragment",
                "http://example.com:8080/path?query#fragment",
            ),
            ("https://example.com:8080", "https://example.com:8080"),
            (
                "https://example.com:8080/path",
                "https://example.com:8080/path",
            ),
            (
                "https://example.com:8080/path?query",
                "https://example.com:8080/path?query",
            ),
            (
                "https://example.com:8080/path#fragment",
                "https://example.com:8080/path#fragment",
            ),
            (
                "https://example.com:8080/path?query#fragment",
                "https://example.com:8080/path?query#fragment",
            ),
            ("ws://example.com/", "ws://example.com/"),
            ("wss://example.com/", "wss://example.com/"),
            ("ws://example.com:8080/", "ws://example.com:8080/"),
            ("wss://example.com:8080/", "wss://example.com:8080/"),
            ("ws://example.com/path", "ws://example.com/path"),
            ("wss://example.com/path", "wss://example.com/path"),
            ("ws://example.com/path?query", "ws://example.com/path?query"),
            (
                "wss://example.com/path?query",
                "wss://example.com/path?query",
            ),
            (
                "ws://example.com/path#fragment",
                "ws://example.com/path#fragment",
            ),
            (
                "wss://example.com/path#fragment",
                "wss://example.com/path#fragment",
            ),
        ];
        for (input, expected) in &urls {
            let url = Url::from_str(input).unwrap();
            assert_eq!(url.to_string(), *expected);
        }
    }
}
