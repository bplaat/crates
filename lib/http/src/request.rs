/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::str::{self};

use crate::method::Method;

pub struct Request {
    pub host: String,
    pub port: u16,
    pub path: String,
    pub method: Method,
    pub headers: BTreeMap<String, String>,
    pub body: String,
    pub client_addr: Option<SocketAddr>,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 80,
            method: Method::Get,
            path: "/".to_string(),
            headers: BTreeMap::new(),
            body: String::new(),
            client_addr: None,
        }
    }
}

impl Request {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn host(mut self, host: impl AsRef<str>) -> Self {
        self.host = host.as_ref().to_string();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn path(mut self, path: impl AsRef<str>) -> Self {
        self.path = path.as_ref().to_string();
        self
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
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

    pub(crate) fn read_from_stream(stream: &mut TcpStream) -> Result<Request, InvalidRequestError> {
        let local_addr = stream
            .local_addr()
            .expect("Can't get tcp stream local addr");
        let client_addr = stream.peer_addr().ok();
        let mut reader = BufReader::new(stream);

        // Read first line
        let mut req = {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|_| InvalidRequestError)?;
            let mut parts = line.split(" ");
            let method = parts.next().ok_or(InvalidRequestError)?.trim().to_string();
            let path = parts.next().ok_or(InvalidRequestError)?.trim().to_string();
            Request {
                host: local_addr.ip().to_string(),
                port: local_addr.port(),
                method: method.parse().map_err(|_| InvalidRequestError)?,
                path,
                headers: BTreeMap::new(),
                body: String::new(),
                client_addr,
            }
        };

        // Read headers
        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|_| InvalidRequestError)?;
            if line == "\r\n" {
                break;
            }
            let mut parts = line.split(":");
            req.headers.insert(
                parts.next().ok_or(InvalidRequestError)?.trim().to_string(),
                parts.next().ok_or(InvalidRequestError)?.trim().to_string(),
            );
        }

        // Read body
        if let Some(content_length) = req.headers.get("Content-Length") {
            let content_length = content_length.parse().map_err(|_| InvalidRequestError)?;
            if content_length > 0 {
                let mut buffer = vec![0_u8; content_length];
                reader.read(&mut buffer).map_err(|_| InvalidRequestError)?;
                if let Ok(text) = str::from_utf8(&buffer) {
                    req.body.push_str(text);
                }
            }
        }
        Ok(req)
    }

    pub(crate) fn write_to_stream(mut self, stream: &mut TcpStream) {
        // Finish headers
        self.headers
            .insert("Host".to_string(), format!("{}:{}", self.host, self.port));
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        self.headers
            .insert("Connection".to_string(), "close".to_string());

        // Write request
        _ = write!(stream, "{} {} HTTP/1.1\r\n", self.method, self.path);
        for (name, value) in &self.headers {
            _ = write!(stream, "{}: {}\r\n", name, value);
        }
        _ = write!(stream, "\r\n{}", self.body);
    }
}

// MARK: InvalidRequestError
#[derive(Debug)]
pub struct InvalidRequestError;

impl Display for InvalidRequestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid request")
    }
}

impl Error for InvalidRequestError {}
