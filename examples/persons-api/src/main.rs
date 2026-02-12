/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple persons REST API example

#![forbid(unsafe_code)]

use std::env;
use std::net::{Ipv4Addr, TcpListener};

use log::info;
use small_router::{Router, RouterBuilder};

use crate::context::Context;
use crate::controllers::{
    home, not_found, persons_create, persons_delete, persons_index, persons_show, persons_update,
};

mod api {
    include!(concat!(env!("OUT_DIR"), "/persons_api.rs"));
}
mod context;
mod controllers;
mod layers;
mod models;
mod validators;

fn router(ctx: Context) -> Router<Context> {
    RouterBuilder::<Context>::with(ctx)
        .pre_layer(layers::log_pre_layer)
        .pre_layer(layers::cors_pre_layer)
        .post_layer(layers::cors_post_layer)
        .get("/", home)
        .get("/persons", persons_index)
        .post("/persons", persons_create)
        .get("/persons/:person_id", persons_show)
        .put("/persons/:person_id", persons_update)
        .delete("/persons/:person_id", persons_delete)
        .fallback(not_found)
        .build()
}

fn main() {
    // Init logger
    if env::var("GATEWAY_INTERFACE").is_err() {
        simple_logger::init().expect("Failed to init logger");
    }

    // Load environment variables
    _ = dotenv::dotenv();

    // Create router and load database
    let database_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "database.db".to_string());
    let context = Context::with_database(&database_path);
    let router = router(context);

    if env::var("GATEWAY_INTERFACE").is_ok() {
        small_http::serve_cgi(move |req| router.handle(req));
    } else {
        // Start server
        let http_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|port| port.parse::<u16>().ok())
            .unwrap_or(8080);
        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, http_port))
            .unwrap_or_else(|_| panic!("Can't bind to port: {http_port}"));
        info!("Server is listening on: http://localhost:{http_port}/");

        small_http::serve(listener, move |req| router.handle(req));
    }
}
