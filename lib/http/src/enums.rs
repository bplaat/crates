/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

// MARK: Version
#[derive(Clone, Debug, PartialEq, Eq)]
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

// MARK: Method
/// HTTP method
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Method {
    /// GET
    Get,
    /// HEAD
    Head,
    /// POST
    Post,
    /// PUT
    Put,
    /// DELETE
    Delete,
    /// CONNECT
    Connect,
    /// OPTIONS
    Options,
    /// TRACE
    Trace,
    /// PATCH
    Patch,
}

impl FromStr for Method {
    type Err = InvalidMethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::Get),
            "HEAD" => Ok(Method::Head),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "CONNECT" => Ok(Method::Connect),
            "OPTIONS" => Ok(Method::Options),
            "TRACE" => Ok(Method::Trace),
            "PATCH" => Ok(Method::Patch),
            _ => Err(InvalidMethodError),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Method::Get => "GET",
                Method::Head => "HEAD",
                Method::Post => "POST",
                Method::Put => "PUT",
                Method::Delete => "DELETE",
                Method::Connect => "CONNECT",
                Method::Options => "OPTIONS",
                Method::Trace => "TRACE",
                Method::Patch => "PATCH",
            }
        )
    }
}

// MARK: InvalidMethodError
#[derive(Debug)]
pub struct InvalidMethodError;

impl Display for InvalidMethodError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid HTTP method")
    }
}

impl Error for InvalidMethodError {}

// MARK: Status
/// Http status
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Status {
    /// 200 OK
    #[default]
    Ok = 200,
    /// 307 Temporary Redirect
    TemporaryRedirect = 307,
    /// 400 Bad Request
    BadRequest = 400,
    /// 401 Unauthorized
    Unauthorized = 401,
    /// 404 Not Found
    NotFound = 404,
    /// 405 Method Not Allowed
    MethodNotAllowed = 405,
    /// 500 Internal Server Error
    InternalServerError = 500,
}

impl TryFrom<i32> for Status {
    type Error = InvalidStatusError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            200 => Ok(Status::Ok),
            307 => Ok(Status::TemporaryRedirect),
            400 => Ok(Status::BadRequest),
            401 => Ok(Status::Unauthorized),
            404 => Ok(Status::NotFound),
            405 => Ok(Status::MethodNotAllowed),
            500 => Ok(Status::InternalServerError),
            _ => Err(InvalidStatusError),
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Status::Ok => "200 OK",
                Status::TemporaryRedirect => "307 Temporary Redirect",
                Status::BadRequest => "400 Bad Request",
                Status::Unauthorized => "401 Unauthorized",
                Status::NotFound => "404 Not Found",
                Status::MethodNotAllowed => "405 Method Not Allowed",
                Status::InternalServerError => "500 Internal Server Error",
            }
        )
    }
}

// MARK: InvalidStatusError
#[derive(Debug)]
pub struct InvalidStatusError;

impl Display for InvalidStatusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid HTTP status")
    }
}

impl Error for InvalidStatusError {}
