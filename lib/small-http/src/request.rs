/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::str::{self, FromStr};

use url::Url;

use crate::enums::{Method, Version};
use crate::header_map::HeaderMap;
use crate::response::Response;
use crate::KEEP_ALIVE_TIMEOUT;

// MARK: Request
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
    pub headers: HeaderMap,
    /// Parameters (mostly added for small-router)
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
            headers: HeaderMap::new(),
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

    /// Create new request with method
    pub fn with_method(method: Method) -> Self {
        Self {
            method,
            ..Self::default()
        }
    }

    /// Create new request with URL
    pub fn with_url(url: impl AsRef<str>) -> Self {
        Self {
            url: url.as_ref().parse().expect("Invalid url"),
            ..Self::default()
        }
    }

    /// Create new request with specific method and URL
    fn with_method_and_url(method: Method, url: impl AsRef<str>) -> Self {
        Self {
            method,
            url: url.as_ref().parse().expect("Invalid url"),
            ..Self::default()
        }
    }

    /// Create new GET request with URL
    pub fn get(url: impl AsRef<str>) -> Self {
        Self::with_method_and_url(Method::Get, url)
    }

    /// Create new HEAD request with URL
    pub fn head(url: impl AsRef<str>) -> Self {
        Self::with_method_and_url(Method::Head, url)
    }

    /// Create new POST request with URL
    pub fn post(url: impl AsRef<str>) -> Self {
        Self::with_method_and_url(Method::Post, url)
    }

    /// Create new PUT request with URL
    pub fn put(url: impl AsRef<str>) -> Self {
        Self::with_method_and_url(Method::Put, url)
    }

    /// Create new DELETE request with URL
    pub fn delete(url: impl AsRef<str>) -> Self {
        Self::with_method_and_url(Method::Delete, url)
    }

    /// Create new CONNECT request with URL
    pub fn connect(url: impl AsRef<str>) -> Self {
        Self::with_method_and_url(Method::Connect, url)
    }

    /// Create new OPTIONS request with URL
    pub fn options(url: impl AsRef<str>) -> Self {
        Self::with_method_and_url(Method::Options, url)
    }

    /// Create new TRACE request with URL
    pub fn trace(url: impl AsRef<str>) -> Self {
        Self::with_method_and_url(Method::Trace, url)
    }

    /// Create new PATCH request with URL
    pub fn patch(url: impl AsRef<str>) -> Self {
        Self::with_method_and_url(Method::Patch, url)
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
            let mut parts = line.split(' ');
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
        let mut headers = HeaderMap::new();
        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|_| InvalidRequestError)?;
            if line == "\r\n" {
                break;
            }
            let split = line.find(':').ok_or(InvalidRequestError)?;
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

    /// Write request to TCP stream
    pub fn write_to_stream(mut self, stream: &mut dyn Write, keep_alive: bool) {
        // Finish headers
        let host = self.url.host().expect("No host in URL");
        self.headers.insert(
            "Host".to_string(),
            if let Some(port) = self.url.port() {
                format!("{}:{}", &host, port)
            } else {
                host.to_string()
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
            if keep_alive {
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

        // Write request
        let path = self.url.path();
        let path = if let Some(query) = self.url.query() {
            format!("{}?{}", &path, query)
        } else {
            path.to_string()
        };
        _ = write!(stream, "{} {} HTTP/1.1\r\n", self.method, path);
        for (name, value) in self.headers.iter() {
            _ = write!(stream, "{}: {}\r\n", name, value);
        }
        _ = write!(stream, "\r\n");
        if let Some(body) = &self.body {
            _ = stream.write_all(body);
        }
    }

    /// Fetch request with http client
    pub fn fetch(self) -> Result<Response, FetchError> {
        let mut stream = TcpStream::connect(format!(
            "{}:{}",
            self.url.host().expect("No host in URL"),
            self.url.port().unwrap_or(80)
        ))
        .map_err(|_| FetchError)?;
        self.write_to_stream(&mut stream, false);
        Response::read_from_stream(&mut stream).map_err(|_| FetchError)
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

// MARK: FetchError
#[derive(Debug)]
pub struct FetchError;

impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Fetch error")
    }
}

impl Error for FetchError {}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::io::Write;
    use std::net::{Ipv4Addr, TcpListener};
    use std::thread;

    use super::*;
    use crate::enums::Status;

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
        let request = Request::get("http://localhost/").header("Host", "localhost");

        let mut buffer = Vec::new();
        request.write_to_stream(&mut buffer, false);
        assert!(buffer.starts_with(b"GET / HTTP/1.1\r\n"));
    }

    #[test]
    fn test_write_to_stream_with_body() {
        let request = Request::post("http://localhost/")
            .header("Host", "localhost")
            .body("Hello, world!");

        let mut buffer = Vec::new();
        request.write_to_stream(&mut buffer, false);
        assert!(buffer.starts_with(b"POST / HTTP/1.1\r\n"));
    }

    #[test]
    fn test_fetch_http1_0() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let server_addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .write_all(b"HTTP/1.0 200 OK\r\nContent-Length: 4\r\n\r\ntest")
                .unwrap();
            stream.flush().unwrap();
        });

        let res = Request::get(format!("http://{}/", server_addr))
            .fetch()
            .unwrap();
        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.body, "test".as_bytes());
    }

    #[test]
    fn test_fetch_http1_1() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let server_addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: closed\r\n\r\ntest",
                )
                .unwrap();
            stream.flush().unwrap();
        });

        let res = Request::get(format!("http://{}/", server_addr))
            .fetch()
            .unwrap();
        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.body, "test".as_bytes());
    }
}
