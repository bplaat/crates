/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

use std::env;
use std::net::{Ipv4Addr, TcpListener};
use std::path::PathBuf;

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
mod deploy;
mod github;
mod layers;
mod models;
mod tasks;
#[cfg(test)]
mod test_utils;
mod utils;

pub(crate) fn router(ctx: Context) -> Router<Context> {
    RouterBuilder::<Context>::with(ctx)
        .pre_layer(layers::log_pre_layer)
        .pre_layer(layers::cors_pre_layer)
        .post_layer(layers::cors_post_layer)
        .post_layer(layers::security_headers_post_layer)
        .pre_layer(layers::auth_optional_pre_layer)
        .get("/api", home)
        .post("/api/auth/login", auth_login)
        .post("/api/webhooks/github/:project_id", github_webhook)
        .pre_layer(layers::spa_file_server_pre_layer)
        .pre_layer(layers::auth_required_pre_layer)
        .get("/api/auth/validate", auth_validate)
        .post("/api/auth/logout", auth_logout)
        .put("/api/account", account_update)
        .post("/api/account/password", account_change_password)
        .put("/api/account/github", account_github_update)
        .delete("/api/account/github", account_github_delete)
        .get("/api/sessions", sessions_index)
        .delete("/api/sessions/:session_id", sessions_delete)
        .get("/api/github/repositories", github_repositories)
        .post("/api/github/repositories/scan", github_repository_scan)
        .get("/api/projects", projects_index)
        .post("/api/projects", projects_create)
        .get("/api/projects/:project_id", projects_show)
        .delete("/api/projects/:project_id", projects_delete)
        .get("/api/projects/:project_id/deployments", project_deployments)
        .post("/api/projects/:project_id/deployments", projects_deploy)
        .build()
}

fn main() {
    _ = dotenvy::dotenv();
    simple_logger::init().expect("Failed to init logger");
    let data_path = PathBuf::from(env::var("DATA_PATH").unwrap_or_else(|_| ".".to_string()));
    std::fs::create_dir_all(data_path.join("projects")).expect("Can't create data directory");
    let server_origin =
        env::var("SERVER_ORIGIN").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let deployments_domain =
        env::var("DEPLOYMENTS_DOMAIN").unwrap_or_else(|_| "*.localhost".to_string());
    let (deploy_tx, deploy_rx) = std::sync::mpsc::channel();
    let context = Context::with_database(
        data_path.join("database.db"),
        server_origin,
        deployments_domain,
        data_path.clone(),
        deploy_tx,
    )
    .expect("Can't open/create database");
    tasks::start_task_runner(
        context.clone(),
        data_path
            .join("dbip-city-lite.mmdb")
            .to_string_lossy()
            .into_owned(),
    );
    deploy::start(context.clone(), deploy_rx);

    let port = env::var("SERVER_PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(8080);
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
        .unwrap_or_else(|_| panic!("Can't bind to port: {port}"));
    info!("Server is listening on: http://localhost:{port}/");
    let router = router(context);
    small_http::serve(listener, move |request| router.handle(request));
}
