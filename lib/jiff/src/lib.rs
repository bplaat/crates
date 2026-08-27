/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [jiff](https://crates.io/crates/jiff) crate

use core::fmt::{Display, Formatter};

pub use timestamp::Timestamp;

/// Civil date and datetime types.
pub mod civil;
mod consts;
/// Date and time formatting support.
pub mod fmt;
mod timestamp;
/// Time zone and offset support.
pub mod tz;
mod utils;

/// A date and time error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("invalid date or time")
    }
}

impl core::error::Error for Error {}
