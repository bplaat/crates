/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple HTTP/1.1 server/client library

pub use crate::enums::{Method, Status};
pub use crate::fetch::fetch;
pub use crate::request::Request;
pub use crate::response::Response;
pub use crate::serve::serve;

mod enums;
mod fetch;
mod request;
mod response;
mod serve;
