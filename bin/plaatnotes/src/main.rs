/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple note-taking app

#![forbid(unsafe_code)]

use std::net::{Ipv4Addr, TcpListener};

use log::info;
use small_router::{Router, RouterBuilder};

use crate::context::Context;
use crate::controllers::*;

mod api {
    include!(concat!(env!("OUT_DIR"), "/api.rs"));
}
mod consts;
mod context;
mod controllers;
mod layers;
mod models;
#[cfg(test)]
mod test_utils;

const HTTP_PORT: u16 = 8080;

pub(crate) fn router(ctx: Context) -> Router<Context> {
    // Guests routes
    let router = RouterBuilder::<Context>::with(ctx)
        .pre_layer(layers::log_pre_layer)
        .pre_layer(layers::cors_pre_layer)
        .post_layer(layers::cors_post_layer)
        .pre_layer(layers::auth_optional_pre_layer)
        .get("/api", home)
        // Auth
        .post("/api/auth/login", auth_login)
        .pre_layer(layers::spa_file_server_pre_layer);

    // Authed routes
    router
        .pre_layer(layers::auth_required_pre_layer)
        // Auth
        .get("/api/auth/validate", auth_validate)
        .post("/api/auth/logout", auth_logout)
        // Users
        .get("/api/users", users_index)
        .post("/api/users", users_create)
        .get("/api/users/:user_id", users_show)
        .put("/api/users/:user_id", users_update)
        .delete("/api/users/:user_id", users_delete)
        .post("/api/users/:user_id/change-password", users_change_password)
        .get("/api/users/:user_id/notes", users_notes)
        // Notes
        .get("/api/notes", notes_index)
        .post("/api/notes", notes_create)
        .get("/api/notes/:note_id", notes_show)
        .put("/api/notes/:note_id", notes_update)
        .delete("/api/notes/:note_id", notes_delete)
        .build()
}

fn main() {
    simple_logger::init().expect("Failed to init logger");

    let context = Context::with_database(if let Ok(path) = std::env::var("DATABASE_PATH") {
        path
    } else {
        "database.db".to_string()
    });

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, HTTP_PORT))
        .unwrap_or_else(|_| panic!("Can't bind to port: {HTTP_PORT}"));
    info!("Server is listening on: http://localhost:{HTTP_PORT}/");

    let router = router(context);
    small_http::serve(listener, move |req| router.handle(req));
}
