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

pub fn fetch(req: Request) -> Result<Response, FetchError> {
    let mut stream =
        TcpStream::connect(format!("{}:{}", req.host, req.port)).map_err(|_| FetchError)?;
    req.write_to_stream(&mut stream);
    Response::read_from_stream(&mut stream).map_err(|_| FetchError)
}

#[derive(Debug)]
pub struct FetchError;

impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Fetch error")
    }
}

impl Error for FetchError {}
