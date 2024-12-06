/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter};

// MARK: Status
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Status {
    Ok = 200,
    TemporaryRedirect = 307,
    BadRequest = 400,
    Unauthorized = 401,
    NotFound = 404,
    MethodNotAllowed = 405,
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
        write!(f, "Invalid status")
    }
}

impl Error for InvalidStatusError {}
