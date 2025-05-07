/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::time::Duration;

pub use crate::client::Client;
pub use crate::enums::{Method, Status};
pub use crate::header_map::HeaderMap;
pub use crate::request::Request;
pub use crate::response::Response;
pub use crate::serve::serve;

mod client;
mod enums;
mod header_map;
mod request;
mod response;
mod serve;

// MARK: Constants
pub(crate) const KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(5);
