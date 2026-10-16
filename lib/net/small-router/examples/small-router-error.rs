/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A small-router example with error handling

use std::net::{Ipv4Addr, TcpListener};

use anyhow::Result;
use small_http::{Request, Response, Status};
use small_router::RouterBuilder;

fn home(_req: &Request, _ctx: &()) -> Result<Response> {
    Ok(Response::with_body("Home"))
}

fn error(_req: &Request, _ctx: &()) -> Result<Response> {
    Err(anyhow::anyhow!("Something went wrong"))
}

fn main() {
    let router = RouterBuilder::new()
        .get("/", home)
        .get("/error", error)
        .error(|_req, _ctx, err| {
            eprintln!("Oops, an error occurred: {err}");
            Response::with_status(Status::InternalServerError).body("500 Internal Server Error")
        })
        .build();

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 8080))
        .unwrap_or_else(|_| panic!("Can't bind to port"));
    small_http::serve(listener, move |req| router.handle(req));
}
