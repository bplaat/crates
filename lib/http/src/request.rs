/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr};
use std::str::{self, FromStr};

use url::Url;

use crate::enums::{Method, Version};

/// HTTP request
#[derive(Clone)]
pub struct Request {
    /// HTTP version
    pub(crate) version: Version,
    /// URL
    pub url: Url,
    /// Method
    pub method: Method,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Parameters
    pub params: HashMap<String, String>,
    /// Body
    pub body: Option<Vec<u8>>,
    /// Client address
    pub client_addr: SocketAddr,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            version: Version::Http1_1,
            url: Url::from_str("http://localhost").expect("Should parse"),
            method: Method::Get,
            headers: HashMap::new(),
            params: HashMap::new(),
            body: None,
            client_addr: (Ipv4Addr::LOCALHOST, 0).into(),
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
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set body
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub(crate) fn read_from_stream(
        stream: &mut dyn Read,
        client_addr: SocketAddr,
    ) -> Result<Request, InvalidRequestError> {
        let mut reader = BufReader::new(stream);

        // Read first line
        let (method, path, version) = {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|_| InvalidRequestError)?;
            let mut parts = line.split(" ");
            (
                parts
                    .next()
                    .ok_or(InvalidRequestError)?
                    .trim()
                    .parse()
                    .map_err(|_| InvalidRequestError)?,
                parts.next().ok_or(InvalidRequestError)?.trim().to_string(),
                parts
                    .next()
                    .ok_or(InvalidRequestError)?
                    .trim()
                    .to_string()
                    .parse()
                    .map_err(|_| InvalidRequestError)?,
            )
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
            let split = line.find(":").ok_or(InvalidRequestError)?;
            headers.insert(
                line[0..split].trim().to_string(),
                line[split + 1..].trim().to_string(),
            );
        }

        // Read body
        let mut body = None;
        if let Some(content_length) = headers.get("Content-Length") {
            let content_length = content_length.parse().map_err(|_| InvalidRequestError)?;
            if content_length > 0 {
                let mut buffer = vec![0; content_length];
                reader.read(&mut buffer).map_err(|_| InvalidRequestError)?;
                body = Some(buffer);
            }
        }

        // Parse URL
        let url = Url::from_str(&if version == Version::Http1_1 {
            format!(
                "http://{}{}",
                headers.get("Host").ok_or(InvalidRequestError)?,
                path
            )
        } else {
            format!("http://localhost{}", path)
        })
        .map_err(|_| InvalidRequestError)?;

        Ok(Request {
            version,
            url,
            method,
            headers,
            params: HashMap::new(),
            body,
            client_addr,
        })
    }

    pub(crate) fn write_to_stream(mut self, stream: &mut dyn Write) {
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
        self.headers.insert(
            "Content-Length".to_string(),
            if let Some(body) = &self.body {
                body.len()
            } else {
                0
            }
            .to_string(),
        );
        if self.version == Version::Http1_1 {
            self.headers
                .insert("Connection".to_string(), "close".to_string());
        }

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
        _ = write!(stream, "\r\n");
        if let Some(body) = &self.body {
            _ = stream.write_all(body);
        }
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

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_from_stream() {
        let raw_request = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let mut stream = &raw_request[..];
        let request =
            Request::read_from_stream(&mut stream, (Ipv4Addr::LOCALHOST, 12345).into()).unwrap();
        assert_eq!(request.method, Method::Get);
        assert_eq!(request.url.to_string(), "http://localhost/");
        assert_eq!(request.version, Version::Http1_1);
        assert_eq!(request.headers.get("Host").unwrap(), "localhost");
    }

    #[test]
    fn test_read_from_stream_with_body() {
        let raw_request =
            b"POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 13\r\n\r\nHello, world!";
        let mut stream = &raw_request[..];
        let request =
            Request::read_from_stream(&mut stream, (Ipv4Addr::LOCALHOST, 12345).into()).unwrap();
        assert_eq!(request.method, Method::Post);
        assert_eq!(request.url.to_string(), "http://localhost/");
        assert_eq!(request.version, Version::Http1_1);
        assert_eq!(request.headers.get("Host").unwrap(), "localhost");
        assert_eq!(request.body.unwrap(), b"Hello, world!");
    }

    #[test]
    fn test_invalid_request_error() {
        let raw_request = b"INVALID REQUEST";
        let mut stream = &raw_request[..];
        let result = Request::read_from_stream(&mut stream, (Ipv4Addr::LOCALHOST, 12345).into());
        assert!(result.is_err());
    }

    #[test]
    fn test_write_to_stream() {
        let request = Request::new()
            .method(Method::Get)
            .url(Url::from_str("http://localhost/").unwrap())
            .header("Host", "localhost");

        let mut buffer = Vec::new();
        request.write_to_stream(&mut buffer);
        assert!(buffer.starts_with(b"GET / HTTP/1.1\r\n"));
    }

    #[test]
    fn test_write_to_stream_with_body() {
        let request = Request::new()
            .method(Method::Post)
            .url(Url::from_str("http://localhost/").unwrap())
            .header("Host", "localhost")
            .body("Hello, world!");

        let mut buffer = Vec::new();
        request.write_to_stream(&mut buffer);
        assert!(buffer.starts_with(b"POST / HTTP/1.1\r\n"));
    }
}
