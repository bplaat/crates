/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Version {
    Http1_0,
    Http1_1,
}

impl FromStr for Version {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.0" => Ok(Version::Http1_0),
            _ => Ok(Version::Http1_1),
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Version::Http1_0 => "HTTP/1.0",
                Version::Http1_1 => "HTTP/1.1",
            }
        )
    }
}
