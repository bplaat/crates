/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal URL parser library

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

// MARK: URL
/// Url
#[derive(Clone)]
pub struct Url {
    /// Scheme
    pub scheme: String,
    /// Authority
    pub authority: Option<Authority>,
    /// Path
    pub path: String,
    /// Query
    pub query: Option<String>,
    /// Fragment
    pub fragment: Option<String>,
}

/// Url authority
#[derive(Clone)]
pub struct Authority {
    /// User info
    pub userinfo: Option<String>,
    /// Host
    pub host: String,
    /// Port
    pub port: Option<u16>,
}

impl FromStr for Url {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("://").collect();
        if parts.len() != 2 {
            return Err(ParseError);
        }

        let scheme = parts[0].to_string();
        let mut path = parts[1];
        if scheme.is_empty() || path.is_empty() {
            return Err(ParseError);
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
                port = Some(authority[idx + 1..].parse().map_err(|_| ParseError)?);
                if let Some(port) = port {
                    if port == 0 {
                        return Err(ParseError);
                    }
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

// MARK: ParseError
/// Url parser error
#[derive(Debug)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "URL parse error")
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
        ];
        for url in &invalid_urls {
            assert!(Url::from_str(url).is_err());
        }
    }
}
