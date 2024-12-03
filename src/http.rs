/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::str;

// Status
pub enum Status {
    Ok = 200,
    TemporaryRedirect = 307,
    BadRequest = 400,
    NotFound = 404,
    MethodNotAllowed = 405,
}

// Request
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Request {
    fn from_stream(stream: &mut TcpStream) -> Option<Request> {
        let mut reader = BufReader::new(stream);

        let mut line = String::new();
        _ = reader.read_line(&mut line);
        let mut req = {
            let mut parts = line.split(" ");
            let method = parts.next().unwrap().trim().to_string();
            let path = parts.next().unwrap().trim().to_string();
            Request {
                method,
                path,
                headers: HashMap::new(),
                body: String::new(),
            }
        };

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(_) => {
                    if line == "\r\n" {
                        break;
                    }
                    let mut parts = line.split(":");
                    req.headers.insert(
                        parts.next().unwrap().trim().to_string(),
                        parts.next().unwrap().trim().to_string(),
                    );
                }
                Err(_) => break,
            }
        }

        if req.method == "POST" {
            let length: usize = req.headers["Content-Length"].parse().unwrap();
            let mut buffer = vec![0_u8; length];
            _ = reader.read(&mut buffer);
            if let Ok(text) = str::from_utf8(&buffer) {
                req.body.push_str(text);
            }
        }
        Some(req)
    }
}

// Response
pub struct Response {
    protocol: String,
    status: Status,
    headers: HashMap<String, String>,
    body: String,
}

impl Response {
    pub fn new() -> Self {
        Self {
            protocol: "HTTP/1.1".to_string(),
            status: Status::Ok,
            headers: HashMap::new(),
            body: String::new(),
        }
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

    // Helpers
    pub fn html(mut self, body: impl AsRef<str>) -> Self {
        self.headers
            .insert("Content-Type".to_string(), "text/html".to_string());
        self.body = body.as_ref().to_string();
        self
    }

    pub fn json(mut self, value: &impl serde::Serialize) -> Self {
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.body = serde_json::to_string(value).unwrap();
        self
    }

    pub fn redirect(mut self, location: impl AsRef<str>) -> Self {
        self.status = Status::TemporaryRedirect;
        self.headers
            .insert("Location".to_string(), location.as_ref().to_string());
        self
    }

    fn write_to_stream(mut self, stream: &mut TcpStream) {
        // Finish headers
        if self.protocol != "HTTP/1.0" {
            self.headers
                .insert("Connection".to_string(), "close".to_string());
        }
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());

        // Write response
        _ = stream.write(self.protocol.as_bytes());
        _ = stream.write(b" ");
        _ = stream.write(match self.status {
            Status::Ok => b"200 OK\r\n",
            Status::TemporaryRedirect => b"307 Temporary Redirect\r\n",
            Status::BadRequest => b"400 Bad Request\r\n",
            Status::NotFound => b"404 Not Found\r\n",
            Status::MethodNotAllowed => b"405 Method Not Allowed\r\n",
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

pub fn serve_with_ctx<T>(handler: fn(Request, ctx: T) -> Response, port: u16, ctx: T)
where
    T: Clone + Send + Sync + 'static,
{
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).unwrap();
    let pool = threadpool::ThreadPool::new(16);
    for mut stream in listener.incoming().flatten() {
        let ctx = ctx.clone();
        pool.execute(move || {
            if let Some(request) = Request::from_stream(&mut stream) {
                handler(request, ctx).write_to_stream(&mut stream);
            }
        });
    }
}
