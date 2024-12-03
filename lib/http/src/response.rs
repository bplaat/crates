/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::BTreeMap;
use std::io::Write;
use std::net::TcpStream;
use std::str::{self};

use crate::Status;

pub struct Response {
    status: Status,
    headers: BTreeMap<String, String>,
    body: String,
}

impl Default for Response {
    fn default() -> Self {
        Self {
            status: Status::Ok,
            headers: BTreeMap::new(),
            body: String::new(),
        }
    }
}

impl Response {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.headers
            .insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    pub fn body(mut self, body: impl AsRef<str>) -> Self {
        self.body = body.as_ref().to_string();
        self
    }

    #[cfg(feature = "json")]
    pub fn json(mut self, value: impl serde::Serialize) -> Self {
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.body = serde_json::to_string(&value).expect("Can't serialize json");
        self
    }

    pub fn redirect(mut self, location: impl AsRef<str>) -> Self {
        self.status = Status::TemporaryRedirect;
        self.headers
            .insert("Location".to_string(), location.as_ref().to_string());
        self
    }

    pub(crate) fn write_to_stream(mut self, stream: &mut TcpStream) {
        // Finish headers
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        self.headers
            .insert("Connection".to_string(), "close".to_string());

        // Write response
        _ = stream.write(b"HTTP/1.1 ");
        _ = stream.write(match self.status {
            Status::Ok => b"200 OK\r\n",
            Status::TemporaryRedirect => b"307 Temporary Redirect\r\n",
            Status::BadRequest => b"400 Bad Request\r\n",
            Status::NotFound => b"404 Not Found\r\n",
            Status::MethodNotAllowed => b"405 Method Not Allowed\r\n",
            Status::InternalServerError => b"500 Internal Server Error\r\n",
        });
        for (name, value) in &self.headers {
            _ = stream.write(name.as_bytes());
            _ = stream.write(b": ");
            _ = stream.write(value.as_bytes());
            _ = stream.write(b"\r\n");
        }
        _ = stream.write(b"\r\n");
        _ = stream.write(self.body.as_bytes());
    }
}
