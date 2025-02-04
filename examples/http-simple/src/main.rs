/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple HTTP server example

use std::net::{Ipv4Addr, TcpListener};
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

use http::{Method, Request, Response, Status};
use serde::Deserialize;
use simple_useragent::UserAgentParser;

const HTTP_PORT: u16 = 8080;

static USER_AGENT_PARSER: LazyLock<UserAgentParser> = LazyLock::new(UserAgentParser::new);

fn handler(req: &Request) -> Response {
    let path = req.url.path.as_str();
    println!("{} {}", req.method, path);

    let res = Response::new().header("Content-Type", "text/html");

    if path == "/" {
        return res.body("<h1>Hello World!</h1>");
    }

    if path == "/greet" {
        if req.method != Method::Post {
            return res
                .status(Status::MethodNotAllowed)
                .body("<h1>405 Method Not Allowed</h1>");
        }

        #[derive(Deserialize)]
        struct GreetBody {
            name: String,
        }
        let body =
            match serde_urlencoded::from_bytes::<GreetBody>(req.body.as_deref().unwrap_or(&[])) {
                Ok(body) => body,
                Err(_) => {
                    return res
                        .status(Status::BadRequest)
                        .body("<h1>400 Bad Request</h1>");
                }
            };
        return res.body(format!("<h1>Hello {}!</h1>", body.name));
    }

    if path == "/redirect" {
        return Response::new().redirect("/");
    }

    if path == "/sleep" {
        thread::sleep(Duration::from_secs(5));
        return res.body("<h1>Sleeping done!</h1>");
    }

    if path == "/ipinfo" {
        let data_res = match http::fetch(Request::with_url("http://ipinfo.io/json")) {
            Ok(res) => res,
            Err(_) => {
                return res
                    .status(Status::InternalServerError)
                    .body("<h1>Can't fetch ipinfo.io</h1>");
            }
        };
        return res
            .header("Content-Type", "application/json")
            .body(data_res.body);
    }

    if path == "/useragent" {
        if let Some(user_agent) = req.headers.get("User-Agent") {
            return res.json(USER_AGENT_PARSER.parse(user_agent));
        }
        return res
            .status(Status::BadRequest)
            .body("Can't parse user agent");
    }

    res.status(Status::NotFound).body("<h1>404 Not Found</h1>")
}

fn main() {
    println!("Server is listening on: http://localhost:{}/", HTTP_PORT);
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, HTTP_PORT))
        .unwrap_or_else(|_| panic!("Can't bind to port: {}", HTTP_PORT));
    http::serve(listener, handler);
}
