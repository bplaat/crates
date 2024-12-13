/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::net::TcpStream;

use crate::request::Request;
use crate::response::Response;

// MARK: Fetch
/// Fetch request with http client
pub fn fetch(req: Request) -> Result<Response, FetchError> {
    let authority = req.url.authority.as_ref().unwrap();
    let mut stream = TcpStream::connect(format!(
        "{}:{}",
        authority.host,
        authority.port.unwrap_or(80)
    ))
    .map_err(|_| FetchError)?;
    req.write_to_stream(&mut stream);
    Response::read_from_stream(&mut stream).map_err(|_| FetchError)
}

// MARK: FetchError
#[derive(Debug)]
pub struct FetchError;

impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Fetch error")
    }
}

impl Error for FetchError {}
