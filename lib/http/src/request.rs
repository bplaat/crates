/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::str::{self};

use anyhow::{Context, Result};

use crate::Method;

pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

impl Request {
    pub(crate) fn from_stream(stream: &mut TcpStream) -> Result<Request> {
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
