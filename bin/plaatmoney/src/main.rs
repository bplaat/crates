/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

use std::env;
use std::net::{Ipv4Addr, TcpListener};

use small_router::{Router, RouterBuilder};

use crate::context::Context;
use crate::controllers::{chart, health, search, sync_status, sync_stream};
use crate::layers::spa_file_server_pre_layer;

mod context;
mod controllers;
mod database;
mod layers;
mod market_data;
mod models;
mod sync;

fn router(context: Context) -> Router<Context> {
    RouterBuilder::with(context)
        .get("/api/health", health)
        .get("/api/search", search)
        .get("/api/chart/:symbol", chart)
        .get("/api/sync", sync_status)
        .get("/api/sync/stream", sync_stream)
        .pre_layer(spa_file_server_pre_layer)
        .build()
}

fn main() {
    simple_logger::init().expect("Failed to initialize logger");
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8080);
    let data_path = env::var("DATA_PATH").unwrap_or_else(|_| "data".to_string());
    std::fs::create_dir_all(&data_path)
        .unwrap_or_else(|error| panic!("Failed to create data directory {data_path}: {error}"));

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
        .unwrap_or_else(|error| panic!("Failed to bind port {port}: {error}"));
    log::info!("PlaatMoney listening on http://localhost:{port}");

    let context = Context::new(format!("{data_path}/plaatmoney.db"))
        .expect("Failed to initialize PlaatMoney database");
    sync::start(context.clone());
    let router = router(context);
    small_http::serve(listener, move |request| router.handle(request));
}
