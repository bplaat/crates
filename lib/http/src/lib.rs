/*
 * Copyright (c) 2023-2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Display, Formatter};
use std::net::{Ipv4Addr, TcpListener};
use std::str::{self, FromStr};

use anyhow::{anyhow, Result};
use threadpool::ThreadPool;

pub use crate::request::Request;
pub use crate::response::Response;

mod request;
mod response;

// MARK: Method
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl FromStr for Method {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            _ => Err(anyhow!("Unknown http method")),
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

// MARK: Status
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Status {
    Ok = 200,
    TemporaryRedirect = 307,
    BadRequest = 400,
    NotFound = 404,
    MethodNotAllowed = 405,
    InternalServerError = 500,
}

// MARK: Serve
pub fn serve<F>(handler: F, port: u16) -> Result<()>
where
    F: Fn(&Request) -> Response + Clone + Send + Sync + 'static,
{
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))?;
    let pool = ThreadPool::new(16);
    for mut stream in listener.incoming().flatten() {
        let handler = handler.clone();
        pool.execute(move || match Request::from_stream(&mut stream) {
            Ok(req) => handler(&req).write_to_stream(&mut stream),
            Err(err) => println!("Error: Invalid http request: {:?}", err),
        });
    }
    Ok(())
}
