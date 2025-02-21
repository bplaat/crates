/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple small-router example with context

use std::net::{Ipv4Addr, TcpListener};

use small_http::{Request, Response, Status};
use small_router::RouterBuilder;

#[derive(Clone)]
struct Context {
    data: String,
}

fn home(_req: &Request, ctx: &Context) -> Response {
    println!("{}", ctx.data);
    Response::with_body("Home")
}

fn about(_req: &Request, ctx: &Context) -> Response {
    println!("{}", ctx.data);
    Response::with_body("About")
}

fn not_found(_req: &Request, _ctx: &Context) -> Response {
    Response::with_status(Status::NotFound).body("404 Not Found")
}

fn main() {
    let ctx = Context {
        data: "Data".to_string(),
    };
    let router = RouterBuilder::with(ctx)
        .get("/", home)
        .get("/about", about)
        .fallback(not_found)
        .build();

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 8080))
        .unwrap_or_else(|_| panic!("Can't bind to port"));
    small_http::serve(listener, move |req| router.handle(req));
}
