/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple and small HTTP/1.1 server/client library

#![forbid(unsafe_code)]

pub use crate::enums::{Method, Status};
pub use crate::request::{HeaderMap, Request};
pub use crate::response::Response;
pub use crate::serve::serve;

mod enums;
mod request;
mod response;
mod serve;
