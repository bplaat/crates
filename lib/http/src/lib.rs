/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple HTTP/1.1 server/client library

pub use crate::fetch::fetch;
pub use crate::method::Method;
pub use crate::request::Request;
pub use crate::response::Response;
pub use crate::serve::serve;
pub use crate::status::Status;

mod fetch;
mod method;
mod request;
mod response;
mod serve;
mod status;
mod version;
