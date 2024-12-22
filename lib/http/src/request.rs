/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::str::{self, FromStr};

use url::Url;

use crate::method::Method;

/// HTTP request
pub struct Request {
    /// URL
    pub url: Url,
    /// Method
    pub method: Method,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Body
    pub body: String,
    /// Client address
    pub client_addr: Option<SocketAddr>,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            url: Url::from_str("http://localhost").expect("Should parse"),
            method: Method::Get,
            headers: HashMap::new(),
            body: String::new(),
            client_addr: None,
        }
    }
}

impl Request {
    /// Create new request
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new request with URL
    pub fn with_url(url: impl AsRef<str>) -> Self {
        Self {
            url: url.as_ref().parse().expect("Invalid url"),
            ..Self::default()
        }
    }

    /// Set URL
    pub fn url(mut self, url: Url) -> Self {
        self.url = url;
        self
    }

    /// Set method
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Set header
    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.headers
            .insert(name.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    /// Set body
    pub fn body(mut self, body: impl AsRef<str>) -> Self {
        self.body = body.as_ref().to_string();
        self
    }

    pub(crate) fn read_from_stream(stream: &mut TcpStream) -> Result<Request, InvalidRequestError> {
        let local_addr = stream
            .local_addr()
            .expect("Can't get tcp stream local addr");
        let client_addr = stream.peer_addr().ok();
        let mut reader = BufReader::new(stream);

        // Read first line
        let (method, path) = {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|_| InvalidRequestError)?;
            let mut parts = line.split(" ");
            let method = parts.next().ok_or(InvalidRequestError)?.trim().to_string();
            let path = parts.next().ok_or(InvalidRequestError)?.trim().to_string();
            (method.parse().map_err(|_| InvalidRequestError)?, path)
        };

        // Read headers
        let mut headers = HashMap::new();
        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|_| InvalidRequestError)?;
            if line == "\r\n" {
                break;
            }
            let mut parts = line.split(":");
            headers.insert(
                parts.next().ok_or(InvalidRequestError)?.trim().to_string(),
                parts.next().ok_or(InvalidRequestError)?.trim().to_string(),
            );
        }

        // Read body
        let mut body = String::new();
        if let Some(content_length) = headers.get("Content-Length") {
            let content_length = content_length.parse().map_err(|_| InvalidRequestError)?;
            if content_length > 0 {
                let mut buffer = vec![0_u8; content_length];
                reader.read(&mut buffer).map_err(|_| InvalidRequestError)?;
                if let Ok(text) = str::from_utf8(&buffer) {
                    body = text.to_string();
                }
            }
        }

        // Parse URL
        let url = Url::from_str(&format!(
            "http://{}{}",
            headers.get("Host").unwrap_or(&local_addr.to_string()),
            path
        ))
        .map_err(|_| InvalidRequestError)?;

        Ok(Request {
            url,
            method,
            headers,
            body,
            client_addr,
        })
    }

    pub(crate) fn write_to_stream(mut self, stream: &mut TcpStream) {
        // Finish headers
        let authority = self.url.authority.as_ref().expect("Invalid url");
        self.headers.insert(
            "Host".to_string(),
            if let Some(port) = authority.port {
                format!("{}:{}", authority.host, port)
            } else {
                authority.host.clone()
            },
        );
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        self.headers
            .insert("Connection".to_string(), "close".to_string());

        // Write request
        let path = if let Some(query) = self.url.query {
            format!("{}?{}", self.url.path, query)
        } else {
            self.url.path
        };
        _ = write!(stream, "{} {} HTTP/1.1\r\n", self.method, path);
        for (name, value) in &self.headers {
            _ = write!(stream, "{}: {}\r\n", name, value);
        }
        _ = write!(stream, "\r\n{}", self.body);
    }
}

// MARK: InvalidRequestError
#[derive(Debug)]
pub(crate) struct InvalidRequestError;

impl Display for InvalidRequestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid request")
    }
}

impl Error for InvalidRequestError {}
