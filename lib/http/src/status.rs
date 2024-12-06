/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Display, Formatter};

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
