/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
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
    pub version: Version,
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
    pub fn with_method_and_url(method: Method, url: impl AsRef<str>) -> Self {
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

    /// Set JSON body
    #[cfg(feature = "json")]
    pub fn json(mut self, value: impl serde::Serialize) -> Self {
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.body = Some(
            serde_json::to_string(&value)
                .expect("Can't serialize json")
                .into(),
        );
        self
    }

    /// Set URL-encoded form body from a slice of key-value pairs.
    #[cfg(feature = "form")]
    pub fn urlencoded(mut self, data: &[(&str, &str)]) -> Self {
        self.headers.insert(
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        );
        self.body = Some(
            serde_urlencoded::to_string(data)
                .expect("Can't serialize urlencoded")
                .into_bytes(),
        );
        self
    }

    /// Parse the request body based on the Content-Type header.
    ///
    /// Supports:
    /// - `application/json` (requires `json` feature)
    /// - `application/x-www-form-urlencoded` (requires `form` feature)
    ///
    /// Returns `Status::UnsupportedMediaType` if the Content-Type is missing or unsupported.
    /// Returns `Status::BadRequest` if the body cannot be deserialized.
    #[cfg(any(feature = "json", feature = "form"))]
    pub fn parse_body<T: serde::de::DeserializeOwned>(&self) -> Result<T, crate::Status> {
        let content_type = self
            .headers
            .get("Content-Type")
            .unwrap_or_default()
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_lowercase();

        let body = self.body.as_deref().unwrap_or(&[]);

        match content_type.as_str() {
            #[cfg(feature = "json")]
            "application/json" => {
                serde_json::from_slice(body).map_err(|_| crate::Status::BadRequest)
            }
            #[cfg(feature = "form")]
            "application/x-www-form-urlencoded" => {
                serde_urlencoded::from_bytes(body).map_err(|_| crate::Status::BadRequest)
            }
            _ => Err(crate::Status::UnsupportedMediaType),
        }
    }
}

impl Request {
    pub(crate) fn read_from_stream(
        stream: &mut dyn Read,
        client_addr: SocketAddr,
    ) -> Result<Request, InvalidRequestError> {
        let mut reader = BufReader::new(stream);

        // Read first line
        let (method, path, version) = {
            let mut line = String::new();
            reader
                .by_ref()
                .take(crate::MAX_HEADER_LINE)
                .read_line(&mut line)
                .map_err(|_| InvalidRequestError("Can't read first line".to_string()))?;
            let mut parts = line.split(' ');
            (
                parts
                    .next()
                    .ok_or(InvalidRequestError(
                        "Can't read 1st part of first line".to_string(),
                    ))?
                    .trim()
                    .parse()
                    .map_err(|_| InvalidRequestError("Can't parse method".to_string()))?,
                parts
                    .next()
                    .ok_or(InvalidRequestError(
                        "Can't read 2st part of first line".to_string(),
                    ))?
                    .trim()
                    .to_string(),
                parts
                    .next()
                    .ok_or(InvalidRequestError(
                        "Can't read 3st part of first line".to_string(),
                    ))?
                    .trim()
                    .to_string()
                    .parse()
                    .map_err(|_| InvalidRequestError("Can't parse HTTP version".to_string()))?,
            )
        };

        // Read headers
        let mut headers = HeaderMap::new();
        loop {
            let mut line = String::new();
            reader
                .by_ref()
                .take(crate::MAX_HEADER_LINE)
                .read_line(&mut line)
                .map_err(|_| InvalidRequestError("Can't read header line".to_string()))?;
            if line == "\r\n" {
                break;
            }
            if headers.len() >= crate::MAX_HEADERS {
                return Err(InvalidRequestError("Too many headers".to_string()));
            }
            let split = line
                .find(':')
                .ok_or(InvalidRequestError("Can't parse header line".to_string()))?;
            headers.insert(
                line[0..split].trim().to_string(),
                line[split + 1..].trim().to_string(),
            );
        }

        // Read body
        let mut body = None;
        let transfer_encoding = headers.get("Transfer-Encoding").map(|s| s.to_lowercase());
        if transfer_encoding.as_deref() == Some("chunked") {
            let mut chunks: Vec<u8> = Vec::new();
            loop {
                let mut size_line = String::new();
                reader
                    .by_ref()
                    .take(crate::MAX_HEADER_LINE)
                    .read_line(&mut size_line)
                    .map_err(|_| InvalidRequestError("Can't read chunk size".to_string()))?;
                // Strip optional chunk extensions (;...) and whitespace
                let hex = size_line.split(';').next().unwrap_or("").trim();
                let chunk_size = usize::from_str_radix(hex, 16)
                    .map_err(|_| InvalidRequestError("Can't parse chunk size".to_string()))?;
                if chunk_size == 0 {
                    break;
                }
                if chunks.len().saturating_add(chunk_size) > crate::MAX_REQUEST_BODY {
                    return Err(InvalidRequestError("Chunked body too large".to_string()));
                }
                let prev_len = chunks.len();
                chunks.resize(prev_len + chunk_size, 0);
                reader
                    .read_exact(&mut chunks[prev_len..])
                    .map_err(|_| InvalidRequestError("Can't read chunk data".to_string()))?;
                // Consume trailing CRLF after chunk data
                let mut crlf = [0u8; 2];
                reader.read_exact(&mut crlf).map_err(|_| {
                    InvalidRequestError("Can't read chunk trailing CRLF".to_string())
                })?;
            }
            body = Some(chunks);
        } else if transfer_encoding.is_some() {
            return Err(InvalidRequestError(
                "Unsupported Transfer-Encoding".to_string(),
            ));
        } else if let Some(content_length) = headers.get("Content-Length") {
            let content_length = content_length
                .parse()
                .map_err(|_| InvalidRequestError("Can't parse Content-Length".to_string()))?;
            if content_length > crate::MAX_REQUEST_BODY {
                return Err(InvalidRequestError("Content-Length too large".to_string()));
            }
            if content_length > 0 {
                let mut buffer = vec![0; content_length];
                reader.read_exact(&mut buffer).map_err(|_| {
                    InvalidRequestError(
                        "Can't read Content-Length amount of bytes from stream".to_string(),
                    )
                })?;
                body = Some(buffer);
            }
        }

        // Parse URL
        let url = Url::from_str(&if version == Version::Http1_1 {
            format!(
                "http://{}{}",
                headers.get("Host").ok_or(InvalidRequestError(
                    "HTTP version is 1.1 but Host header is not set".to_string()
                ))?,
                path
            )
        } else {
            format!("http://localhost{path}")
        })
        .map_err(|_| InvalidRequestError("Can't parse request url".to_string()))?;

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

    #[cfg(feature = "cgi")]
    pub(crate) fn from_cgi_env() -> Result<Request, InvalidRequestError> {
        use std::env;

        // Read method, path and version
        let method = env::var("REQUEST_METHOD")
            .ok()
            .and_then(|m| m.parse().ok())
            .ok_or(InvalidRequestError(
                "Can't read REQUEST_METHOD from env".to_string(),
            ))?;
        let mut path = env::var("PATH_INFO")
            .map_err(|_| InvalidRequestError("Can't read PATH_INFO from env".to_string()))?;
        if path.is_empty() {
            path = "/".to_string();
        }
        if let Ok(query_string) = env::var("QUERY_STRING") {
            if !query_string.is_empty() {
                path = format!("{path}?{query_string}");
            }
        }
        let version = match env::var("SERVER_PROTOCOL").as_deref() {
            Ok("HTTP/1.0") => Version::Http1_0,
            _ => Version::Http1_1,
        };

        // Read headers
        let mut headers = HeaderMap::new();
        for (key, value) in env::vars() {
            if let Some(key) = key.strip_prefix("HTTP_") {
                headers.insert(key.replace('_', "-"), value);
            }
        }

        // Read body
        let mut body = None;
        if let Ok(content_length) = env::var("CONTENT_LENGTH") {
            if let Ok(content_length) = content_length.parse::<usize>() {
                if content_length > crate::MAX_REQUEST_BODY {
                    return Err(InvalidRequestError("Content-Length too large".to_string()));
                }
                if content_length > 0 {
                    let mut buffer = vec![0; content_length];
                    std::io::stdin().read_exact(&mut buffer).map_err(|_| {
                        InvalidRequestError(
                            "Can't read Content-Length amount of bytes from stdin".to_string(),
                        )
                    })?;
                    body = Some(buffer);
                }
            }
        }

        // Read remote address
        let client_addr = if let Ok(mut remote_addr) = env::var("REMOTE_ADDR") {
            if remote_addr.starts_with("::ffff:") {
                remote_addr = remote_addr.trim_start_matches("::ffff:").to_string();
            }
            let remote_port = env::var("REMOTE_PORT").unwrap_or_else(|_| "0".to_string());
            format!("{remote_addr}:{remote_port}")
                .parse()
                .map_err(|_| {
                    InvalidRequestError("Can't parse REMOTE_ADDR and REMOTE_PORT".to_string())
                })?
        } else {
            (Ipv4Addr::LOCALHOST, 0).into()
        };

        // Parse URL
        let url = Url::from_str(&if version == Version::Http1_1 {
            format!(
                "http://{}{}",
                headers.get("Host").ok_or(InvalidRequestError(
                    "HTTP version is 1.1 but Host header is not set".to_string()
                ))?,
                path
            )
        } else {
            format!("http://localhost{path}")
        })
        .map_err(|_| InvalidRequestError("Can't parse request url".to_string()))?;

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

    /// Get client IP address, respecting X-Forwarded-For and X-Real-IP proxy headers
    pub fn ip(&self) -> IpAddr {
        self.headers
            .get("X-Forwarded-For")
            .and_then(|v| v.split(',').next())
            .or_else(|| self.headers.get("X-Real-IP"))
            .and_then(|ip_str| ip_str.trim().parse::<IpAddr>().ok())
            .unwrap_or_else(|| self.client_addr.ip())
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
        for (name, value) in &self.headers {
            let safe_name = name.replace(['\r', '\n'], "");
            let safe_value = value.replace(['\r', '\n'], "");
            _ = write!(stream, "{safe_name}: {safe_value}\r\n");
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
pub(crate) struct InvalidRequestError(String);

impl Display for InvalidRequestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid request: {}", self.0)
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
    fn test_read_from_stream_with_body_lowercase_headers() {
        let raw_request =
            b"POST / HTTP/1.1\r\nhost: localhost\r\ncontent-Length: 13\r\n\r\nHello, world!";
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

        let res = Request::get(format!("http://{server_addr}/"))
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

        let res = Request::get(format!("http://{server_addr}/"))
            .fetch()
            .unwrap();
        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.body, "test".as_bytes());
    }
}
