/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::str::{self, FromStr};

use anyhow::{Context, Result};

// Method
#[derive(Eq, PartialEq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl FromStr for Method {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            _ => Err(anyhow::anyhow!("Unknown http method")),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Method::Get => "GET",
                Method::Post => "POST",
                Method::Put => "PUT",
                Method::Delete => "DELETE",
            }
        )
    }
}

// Status
#[derive(Eq, PartialEq)]
pub enum Status {
    Ok = 200,
    TemporaryRedirect = 307,
    BadRequest = 400,
    NotFound = 404,
    MethodNotAllowed = 405,
    InternalServerError = 500,
}

// Request
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

impl Request {
    fn from_stream(stream: &mut TcpStream) -> Result<Request> {
        let mut reader = BufReader::new(stream);

        let mut line = String::new();
        _ = reader.read_line(&mut line);
        let mut req = {
            let mut parts = line.split(" ");
            let method = parts
                .next()
                .context("Can't parse http header")?
                .trim()
                .to_string();
            let path = parts
                .next()
                .context("Can't parse http header")?
                .trim()
                .to_string();
            Request {
                method: method.parse()?,
                path,
                headers: BTreeMap::new(),
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
                        parts
                            .next()
                            .context("Can't parse http header")?
                            .trim()
                            .to_string(),
                        parts
                            .next()
                            .context("Can't parse http header")?
                            .trim()
                            .to_string(),
                    );
                }
                Err(_) => break,
            }
        }

        if let Some(content_length) = req.headers.get("Content-Length") {
            let content_length = content_length
                .parse()
                .context("Can't parse Content-Length header")?;
            let mut buffer = vec![0_u8; content_length];
            _ = reader.read(&mut buffer);
            if let Ok(text) = str::from_utf8(&buffer) {
                req.body.push_str(text);
            }
        }
        Ok(req)
    }
}

// Response
pub struct Response {
    status: Status,
    headers: BTreeMap<String, String>,
    body: String,
}

impl Response {
    pub fn new() -> Self {
        Self {
            status: Status::Ok,
            headers: BTreeMap::new(),
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

    fn write_to_stream(mut self, stream: &mut TcpStream) {
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

pub fn serve_with_ctx<T>(
    handler: fn(Request, ctx: T) -> Result<Response>,
    port: u16,
    ctx: T,
) -> Result<()>
where
    T: Clone + Send + Sync + 'static,
{
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))?;
    let pool = threadpool::ThreadPool::new(16);
    for mut stream in listener.incoming().flatten() {
        let ctx = ctx.clone();
        pool.execute(move || {
            if let Ok(request) = Request::from_stream(&mut stream) {
                match handler(request, ctx) {
                    Ok(response) => response.write_to_stream(&mut stream),
                    Err(err) => {
                        println!("Error: {}", err);
                        Response::new()
                            .status(Status::InternalServerError)
                            .body("500 Internal Server Error")
                            .write_to_stream(&mut stream);
                    }
                }
            }
        });
    }
    Ok(())
}
