/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal URL parser library

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// Url
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
        let rest = parts[1];

        let mut authority = None;
        let mut path = rest;
        let mut query = None;
        let mut fragment = None;

        if let Some(idx) = rest.find('/') {
            authority = Some(rest[..idx].to_string());
            path = &rest[idx..];
        }

        if let Some(idx) = path.find('?') {
            query = Some(path[idx + 1..].to_string());
            path = &path[..idx];
        }

        if let Some(idx) = path.find('#') {
            fragment = Some(path[idx + 1..].to_string());
            path = &path[..idx];
        }

        let authority = authority.map(|auth| {
            let parts: Vec<&str> = auth.split('@').collect();
            let (userinfo, hostport) = if parts.len() == 2 {
                (Some(parts[0].to_string()), parts[1])
            } else {
                (None, parts[0])
            };

            let parts: Vec<&str> = hostport.split(':').collect();
            let (host, port) = if parts.len() == 2 {
                (parts[0].to_string(), Some(parts[1].parse().unwrap()))
            } else {
                (parts[0].to_string(), None)
            };

            Authority {
                userinfo,
                host,
                port,
            }
        });

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
