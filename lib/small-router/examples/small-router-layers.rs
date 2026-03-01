/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple small-router example with context

use std::net::{Ipv4Addr, TcpListener};

use small_http::{Method, Request, Response, Status};
use small_router::RouterBuilder;

/// Pre-layer that processes CORS requests
fn cors_pre_layer(req: &Request, _: &mut ()) -> Option<Response> {
    if req.method == Method::Options && req.headers.get("Access-Control-Request-Method").is_some() {
        Some(
            Response::with_status(Status::NoContent)
                .header("Access-Control-Allow-Origin", "*")
                .header(
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, PATCH, DELETE, OPTIONS",
                )
                .header("Access-Control-Max-Age", "86400"),
        )
    } else {
        None
    }
}

/// Post-layer that processes CORS requests
fn cors_post_layer(req: &Request, _: &mut (), res: Response) -> Response {
    if !(req.method == Method::Options
        && req.headers.get("Access-Control-Request-Method").is_some())
    {
        res.header("Access-Control-Allow-Origin", "*")
    } else {
        res
    }
}

fn home(_req: &Request, _ctx: &()) -> Response {
    Response::with_body("Home")
}

fn about(_req: &Request, _ctx: &()) -> Response {
    Response::with_body("About")
}

fn not_found(_req: &Request, _ctx: &()) -> Response {
    Response::with_status(Status::NotFound).body("404 Not Found")
}

fn main() {
    let router = RouterBuilder::new()
        .pre_layer(cors_pre_layer)
        .post_layer(cors_post_layer)
        .get("/", home)
        .get("/about", about)
        .fallback(not_found)
        .build();

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 8080))
        .unwrap_or_else(|_| panic!("Can't bind to port"));
    small_http::serve(listener, move |req| router.handle(req));
}
