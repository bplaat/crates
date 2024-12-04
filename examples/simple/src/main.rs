/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::net::{Ipv4Addr, TcpListener};
use std::thread;
use std::time::Duration;

use http::{Method, Request, Response, Status};
use serde::Deserialize;

const HTTP_PORT: u16 = 8000;

fn handler(req: &Request) -> Response {
    println!("{} {}", req.method, req.path);

    let res = Response::new().header("Content-Type", "text/html");

    if req.path == "/" {
        return res.body("<h1>Hello World!</h1>");
    }

    if req.path == "/greet" {
        if req.method != Method::Post {
            return res
                .status(Status::MethodNotAllowed)
                .body("<h1>405 Method Not Allowed</h1>");
        }

        #[derive(Deserialize)]
        struct GreetBody {
            name: String,
        }
        let body = match serde_urlencoded::from_str::<GreetBody>(&req.body) {
            Ok(body) => body,
            Err(_) => {
                return res
                    .status(Status::BadRequest)
                    .body("<h1>400 Bad Request</h1>");
            }
        };
        return res.body(format!("<h1>Hello {}!</h1>", body.name));
    }

    if req.path == "/redirect" {
        return Response::new().redirect("/");
    }

    if req.path == "/sleep" {
        thread::sleep(Duration::from_secs(5));
        return res.body("<h1>Sleeping done!</h1>");
    }

    res.status(Status::NotFound).body("<h1>404 Not Found</h1>")
}

fn main() {
    println!("Server is listening on: http://localhost:{}/", HTTP_PORT);
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, HTTP_PORT))
        .unwrap_or_else(|_| panic!("Can't bind to port: {}", HTTP_PORT));
    http::serve(listener, handler);
}
