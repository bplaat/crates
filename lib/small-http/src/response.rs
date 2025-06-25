/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

use crate::enums::{Status, Version};
use crate::header_map::HeaderMap;
use crate::request::Request;
use crate::KEEP_ALIVE_TIMEOUT;

// MARK: Response
/// HTTP response
#[derive(Default)]
pub struct Response {
    /// Status
    pub status: Status,
    /// Headers
    pub headers: HeaderMap,
    /// Body
    pub body: Vec<u8>,
    pub(crate) takeover: Option<Box<dyn FnOnce(TcpStream) + Send + 'static>>,
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
    pub fn with_header(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self::default().header(name.into(), value.into())
    }

    /// Set header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
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
    pub fn with_redirect(location: impl Into<String>) -> Self {
        Self::default().redirect(location.into())
    }

    /// Set redirect header
    pub fn redirect(mut self, location: impl Into<String>) -> Self {
        self.status = Status::TemporaryRedirect;
        self.headers.insert("Location".to_string(), location.into());
        self
    }

    /// Set takeover function
    pub fn takeover(mut self, f: impl FnOnce(TcpStream) + Send + 'static) -> Self {
        self.takeover = Some(Box::new(f));
        self
    }

    /// Parse json out of body
    #[cfg(feature = "json")]
    pub fn into_json<T: serde::de::DeserializeOwned>(self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    /// Read response from stream
    pub fn read_from_stream(stream: &mut dyn Read) -> Result<Self, InvalidResponseError> {
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
            let split = line.find(':').ok_or(InvalidResponseError)?;
            res.headers.insert(
                line[0..split].trim().to_string(),
                line[split + 1..].trim().to_string(),
            );
        }

        // Read body
        if let Some(transfer_encoding) = res.headers.get("Transfer-Encoding") {
            if transfer_encoding == "chunked" {
                let mut body = Vec::new();
                loop {
                    // Read chunk size
                    let mut size_line = String::new();
                    reader
                        .read_line(&mut size_line)
                        .map_err(|_| InvalidResponseError)?;
                    let size = usize::from_str_radix(size_line.trim(), 16)
                        .map_err(|_| InvalidResponseError)?;
                    if size == 0 {
                        break;
                    }

                    // Read chunk
                    let mut chunk = vec![0; size];
                    reader
                        .read_exact(&mut chunk)
                        .map_err(|_| InvalidResponseError)?;
                    body.extend_from_slice(&chunk);

                    // Read the trailing \r\n after each chunk
                    let mut crlf = [0; 2];
                    reader
                        .read_exact(&mut crlf)
                        .map_err(|_| InvalidResponseError)?;
                }
                res.body = body;
                return Ok(res);
            }
        }
        if let Some(content_length) = res.headers.get("Content-Length") {
            let content_length = content_length.parse().map_err(|_| InvalidResponseError)?;
            if content_length > 0 {
                res.body = vec![0; content_length];
                reader
                    .read_exact(&mut res.body)
                    .map_err(|_| InvalidResponseError)?;
            }
        }
        Ok(res)
    }

    pub(crate) fn write_to_stream(
        &mut self,
        stream: &mut dyn Write,
        req: &Request,
        keep_alive: bool,
    ) {
        self.finish_headers(req, keep_alive);

        _ = write!(stream, "{} {}\r\n", req.version, self.status);
        for (name, value) in self.headers.iter() {
            _ = write!(stream, "{}: {}\r\n", name, value);
        }
        _ = write!(stream, "\r\n");
        _ = stream.write_all(&self.body);
    }

    fn finish_headers(&mut self, req: &Request, keep_alive: bool) {
        #[cfg(feature = "date")]
        self.headers
            .insert("Date".to_string(), chrono::Utc::now().to_rfc2822());
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        if req.version == Version::Http1_1 {
            if keep_alive && req.headers.get("Connection").map(|v| v.as_str()) != Some("close") {
                if self.headers.get("Connection").is_none() {
                    self.headers
                        .insert("Connection".to_string(), "keep-alive".to_string());
                    self.headers.insert(
                        "Keep-Alive".to_string(),
                        format!("timeout={}", KEEP_ALIVE_TIMEOUT.as_secs()),
                    );
                }
            } else {
                self.headers
                    .insert("Connection".to_string(), "close".to_string());
            }
        }
    }
}

// MARK: InvalidResponseError
/// Invalid response error
#[derive(Debug)]
pub struct InvalidResponseError;

impl Display for InvalidResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid response")
    }
}

impl Error for InvalidResponseError {}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_response() {
        let response_text = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, world!";
        let mut response_stream = response_text.as_bytes();
        let response = Response::read_from_stream(&mut response_stream).unwrap();

        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.headers.get("Content-Length").unwrap(), "13");
        assert_eq!(response.body, b"Hello, world!");
    }

    #[test]
    fn test_parse_response_with_headers() {
        let response_text =
            "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nX-Custom-Header: Value\r\n\r\n";
        let mut response_stream = response_text.as_bytes();
        let response = Response::read_from_stream(&mut response_stream).unwrap();

        assert_eq!(response.status, Status::NotFound);
        assert_eq!(response.headers.get("Content-Length").unwrap(), "0");
        assert_eq!(response.headers.get("X-Custom-Header").unwrap(), "Value");
        assert!(response.body.is_empty());
    }

    #[test]
    fn test_parse_response_invalid() {
        let response_text = "INVALID RESPONSE";
        let mut response_stream = response_text.as_bytes();
        let result = Response::read_from_stream(&mut response_stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_response_chunked_encoding() {
        let response_text = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nBast\r\n4\r\niaan\r\n0\r\n\r\n";
        let mut response_stream = response_text.as_bytes();
        let response = Response::read_from_stream(&mut response_stream).unwrap();

        assert_eq!(response.status, Status::Ok);
        assert_eq!(
            response.headers.get("Transfer-Encoding").unwrap(),
            "chunked"
        );
        assert_eq!(response.body, b"Bastiaan");
    }

    #[test]
    #[cfg(feature = "json")]
    fn test_parse_response_with_json() {
        let response_text = "HTTP/1.1 200 OK\r\nContent-Length: 15\r\nContent-Type: application/json\r\n\r\n{\"key\":\"value\"}";
        let mut response_stream = response_text.as_bytes();
        let response = Response::read_from_stream(&mut response_stream).unwrap();

        assert_eq!(response.status, Status::Ok);
        assert_eq!(
            response.headers.get("Content-Type").unwrap(),
            "application/json"
        );
        assert_eq!(response.body, b"{\"key\":\"value\"}");

        let json_value: serde_json::Value = response.into_json().unwrap();
        assert_eq!(json_value["key"], "value");
    }

    #[test]
    fn test_write_response() {
        let mut response = Response::with_status(Status::Ok)
            .header("Content-Length", "13")
            .body("Hello, world!");
        let mut response_stream = Vec::new();
        let request = Request {
            version: Version::Http1_1,
            ..Default::default()
        };
        response.write_to_stream(&mut response_stream, &request, true);

        let response_text = String::from_utf8(response_stream).unwrap();
        assert!(response_text.contains("HTTP/1.1 200 OK"));
        assert!(response_text.contains("Content-Length: 13"));
        assert!(response_text.contains("\r\n\r\nHello, world!"));
    }

    #[test]
    fn test_write_response_with_headers() {
        let mut response = Response::with_status(Status::NotFound)
            .header("Content-Length", "0")
            .header("X-Custom-Header", "Value");
        let mut response_stream = Vec::new();
        let request = Request {
            version: Version::Http1_1,
            ..Default::default()
        };
        response.write_to_stream(&mut response_stream, &request, true);

        let response_text = String::from_utf8(response_stream).unwrap();
        assert!(response_text.contains("HTTP/1.1 404 Not Found"));
        assert!(response_text.contains("Content-Length: 0"));
        assert!(response_text.contains("X-Custom-Header: Value"));
        assert!(response_text.contains("\r\n\r\n"));
    }

    #[test]
    #[cfg(feature = "json")]
    fn test_write_response_with_json() {
        let mut response = Response::with_json(serde_json::json!({"key": "value"}));
        let mut response_stream = Vec::new();
        let request = Request {
            version: Version::Http1_1,
            ..Default::default()
        };
        response.write_to_stream(&mut response_stream, &request, true);

        let response_text = String::from_utf8(response_stream).unwrap();
        assert!(response_text.contains("HTTP/1.1 200 OK"));
        assert!(response_text.contains("Content-Type: application/json"));
        assert!(response_text.contains("\r\n\r\n{\"key\":\"value\"}"));
    }
}
