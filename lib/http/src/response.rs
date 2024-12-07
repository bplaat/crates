/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::str::{self};

use crate::status::Status;

pub struct Response {
    pub status: Status,
    pub headers: BTreeMap<String, String>,
    pub body: String,
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

    pub(crate) fn read_from_stream(stream: &mut TcpStream) -> Result<Self, InvalidResponseError> {
        let mut reader = BufReader::new(stream);

        // Read first line
        let mut res = {
            let mut line = String::new();
            _ = reader.read_line(&mut line);
            let mut parts = line.splitn(3, ' ');
            let _http_version = parts.next().ok_or(InvalidResponseError)?;
            let status_code = parts
                .next()
                .ok_or(InvalidResponseError)?
                .parse::<i32>()
                .map_err(|_| InvalidResponseError)?;
            Response::default()
                .status(Status::try_from(status_code).map_err(|_| InvalidResponseError)?)
        };

        // Read headers
        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|_| InvalidResponseError)?;
            if line == "\r\n" {
                break;
            }
            let mut parts = line.split(":");
            res.headers.insert(
                parts.next().ok_or(InvalidResponseError)?.trim().to_string(),
                parts.next().ok_or(InvalidResponseError)?.trim().to_string(),
            );
        }

        // Read body
        if let Some(content_length) = res.headers.get("Content-Length") {
            let content_length = content_length.parse().map_err(|_| InvalidResponseError)?;
            if content_length > 0 {
                let mut buffer = vec![0_u8; content_length];
                _ = reader.read(&mut buffer);
                if let Ok(text) = str::from_utf8(&buffer) {
                    res.body.push_str(text);
                }
            }
        }
        Ok(res)
    }

    pub(crate) fn write_to_stream(mut self, stream: &mut TcpStream) {
        // Finish headers
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        self.headers
            .insert("Connection".to_string(), "close".to_string());

        // Write response
        _ = write!(stream, "HTTP/1.1 {}\r\n", self.status);
        for (name, value) in &self.headers {
            _ = write!(stream, "{}: {}\r\n", name, value);
        }
        _ = write!(stream, "\r\n{}", self.body);
    }
}

// MARK: InvalidResponseError
#[derive(Debug)]
pub struct InvalidResponseError;

impl Display for InvalidResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid response")
    }
}

impl Error for InvalidResponseError {}
