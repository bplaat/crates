/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::str::{self};

use crate::serve::KEEP_ALIVE_TIMEOUT;
use crate::status::Status;
use crate::version::Version;
use crate::Request;

/// HTTP response
#[derive(Default)]
pub struct Response {
    /// Status
    pub status: Status,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Body
    pub body: Vec<u8>,
}

impl Response {
    /// Create new response
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new response with status
    pub fn with_status(status: Status) -> Self {
        Self {
            status,
            ..Default::default()
        }
    }

    /// Set status
    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    /// Create new response with header
    pub fn with_header(name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        Self::default().header(name, value)
    }

    /// Set header
    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.headers
            .insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Create new response with body
    pub fn with_body(body: impl Into<Vec<u8>>) -> Self {
        Self {
            body: body.into(),
            ..Default::default()
        }
    }

    /// Set body
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Create new response with json body
    #[cfg(feature = "json")]
    pub fn with_json(value: impl serde::Serialize) -> Self {
        Self::default().json(value)
    }

    /// Set json body
    #[cfg(feature = "json")]
    pub fn json(mut self, value: impl serde::Serialize) -> Self {
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.body = serde_json::to_string(&value)
            .expect("Can't serialize json")
            .into();
        self
    }

    /// Create new response with redirect header
    pub fn with_redirect(location: impl AsRef<str>) -> Self {
        Self::default().redirect(location)
    }

    /// Set redirect header
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
            reader
                .read_line(&mut line)
                .map_err(|_| InvalidResponseError)?;
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
            let split = line.find(":").ok_or(InvalidResponseError)?;
            res.headers.insert(
                line[0..split].trim().to_string(),
                line[split + 1..].trim().to_string(),
            );
        }

        // Read body
        if let Some(content_length) = res.headers.get("Content-Length") {
            let content_length = content_length.parse().map_err(|_| InvalidResponseError)?;
            if content_length > 0 {
                res.body = Vec::with_capacity(content_length);
                reader
                    .read(&mut res.body)
                    .map_err(|_| InvalidResponseError)?;
            }
        }
        Ok(res)
    }

    pub(crate) fn write_to_stream(mut self, stream: &mut TcpStream, req: &Request) {
        // Finish headers
        #[cfg(feature = "date")]
        self.headers.insert(
            "Date".to_string(),
            chrono::Utc::now().to_rfc2822().replace("+0000", "GMT"),
        );
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        if req.version == Version::Http1_1 {
            if req.headers.get("Connection").map(|v| v.as_str()) != Some("close") {
                self.headers
                    .insert("Connection".to_string(), "keep-alive".to_string());
                self.headers.insert(
                    "Keep-Alive".to_string(),
                    format!("timeout={}", KEEP_ALIVE_TIMEOUT.as_secs()),
                );
            } else {
                self.headers
                    .insert("Connection".to_string(), "close".to_string());
            }
        }

        // Write response
        _ = write!(stream, "{} {}\r\n", req.version, self.status);
        for (name, value) in &self.headers {
            _ = write!(stream, "{}: {}\r\n", name, value);
        }
        _ = write!(stream, "\r\n");
        _ = stream.write_all(&self.body);
    }
}

// MARK: InvalidResponseError
#[derive(Debug)]
pub(crate) struct InvalidResponseError;

impl Display for InvalidResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid response")
    }
}

impl Error for InvalidResponseError {}
