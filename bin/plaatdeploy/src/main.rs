/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal self-hosted deployment service

use std::env;
use std::net::{Ipv4Addr, TcpListener};
use std::path::Path;
use std::sync::mpsc;

use log::info;
use small_router::{Router, RouterBuilder};

use crate::context::Context;
use crate::controllers::*;
use crate::deploy::DeployTask;

mod api {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/api.rs"));
}
mod consts;
mod context;
mod controllers;
mod deploy;
mod github;
mod layers;
mod models;
mod proxy;
mod tasks;
#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod tests;
mod utils;

pub(crate) fn router(ctx: Context) -> Router<Context> {
    RouterBuilder::<Context>::with(ctx)
        .pre_layer(layers::log_pre_layer)
        .pre_layer(layers::cors_pre_layer)
        .post_layer(layers::cors_post_layer)
        .pre_layer(layers::auth_optional_pre_layer)
        // SPA: serve frontend assets before auth-required gate
        .pre_layer(layers::spa_pre_layer)
        // Misc
        .get("/api", home)
        // Auth
        .post("/api/auth/login", auth_login)
        // Webhook (public but HMAC-verified)
        .post("/api/webhook/github", webhook_github)
        // Authed routes
        .pre_layer(layers::auth_required_pre_layer)
        .get("/api/auth/validate", auth_validate)
        .post("/api/auth/logout", auth_logout)
        // Users (admin only)
        .get("/api/users", users_index)
        .post("/api/users", users_create)
        .get("/api/users/:user_id", users_show)
        .put("/api/users/:user_id", users_update)
        .post("/api/users/:user_id/change-password", users_change_password)
        .delete("/api/users/:user_id", users_delete)
        // Teams
        .get("/api/teams", teams_index)
        .post("/api/teams", teams_create)
        .get("/api/teams/:team_id", teams_show)
        .put("/api/teams/:team_id", teams_update)
        .delete("/api/teams/:team_id", teams_delete)
        .get("/api/teams/:team_id/github", teams_github_show)
        .put("/api/teams/:team_id/github", teams_github_update)
        .delete("/api/teams/:team_id/github", teams_github_delete)
        .get(
            "/api/teams/:team_id/github/repositories",
            teams_github_repositories,
        )
        .get("/api/teams/:team_id/github/branches", teams_github_branches)
        .post("/api/teams/:team_id/members", teams_members_create)
        .put("/api/teams/:team_id/members/:user_id", teams_members_update)
        .delete("/api/teams/:team_id/members/:user_id", teams_members_delete)
        // Sessions
        .get("/api/sessions", sessions_index)
        .delete("/api/sessions/:session_id", sessions_delete)
        // Projects
        .get("/api/projects", projects_index)
        .post("/api/projects", projects_create)
        .get("/api/projects/:project_id", projects_show)
        .put("/api/projects/:project_id", projects_update)
        .delete("/api/projects/:project_id", projects_delete)
        .post("/api/projects/:project_id/deploy", projects_deploy)
        .get(
            "/api/projects/:project_id/deployments",
            projects_deployments,
        )
        // Deployments
        .get("/api/deployments/:deployment_id", deployments_show)
        .build()
}

fn main() {
    _ = dotenv::dotenv();
    simple_logger::init().expect("Failed to init logger");

    let server_origin =
        env::var("SERVER_ORIGIN").unwrap_or_else(|_| "http://localhost".to_string());
    let deployments_origin = env::var("SERVER_DEPLOYMENTS_ORIGIN")
        .unwrap_or_else(|_| Context::default_deployments_origin(&server_origin));
    let data_path = env::var("DATA_PATH").unwrap_or_else(|_| ".".to_string());
    std::fs::create_dir_all(&data_path).expect("Can't create data directory");
    let database_path = Path::new(&data_path).join("database.db");
    let deploy_path = Path::new(&data_path).join("projects");
    std::fs::create_dir_all(&deploy_path).expect("Can't create projects directory");

    let (deploy_tx, deploy_rx) = mpsc::channel::<DeployTask>();

    let ctx = Context::with_database(
        &database_path,
        server_origin,
        deployments_origin,
        deploy_path.to_string_lossy().into_owned(),
        deploy_tx,
    )
    .expect("Can't open/create database");

    // Start task runner (also handles DB-IP database download on startup)
    let mmdb_path = Path::new(&data_path)
        .join("dbip-city-lite.mmdb")
        .to_string_lossy()
        .into_owned();
    tasks::start_task_runner(ctx.clone(), mmdb_path);

    // Start deploy runner thread
    deploy::runner::start(
        ctx.clone(),
        deploy_rx,
        deploy_path.to_string_lossy().into_owned(),
    );

    // Stop all child containers when we receive SIGTERM or SIGINT
    ctrlc::set_handler(|| {
        log::info!("Shutdown signal received, stopping all containers...");
        deploy::runner::stop_all_containers();
        std::process::exit(0);
    })
    .expect("Failed to set signal handler");

    let http_port = match env::var("SERVER_PORT") {
        Ok(value) => value.parse::<u16>().unwrap_or_else(|_| {
            log::warn!("Invalid SERVER_PORT '{value}', falling back to 8080");
            8080
        }),
        Err(_) => 8080,
    };
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, http_port))
        .unwrap_or_else(|_| panic!("Can't bind to port {http_port}"));
    info!("Server is listening on: http://localhost:{http_port}/");

    let admin_router = router(ctx.clone());

    small_http::serve(listener, move |req| {
        proxy::dispatch(
            req,
            &ctx,
            &admin_router,
            &ctx.server_host,
            deploy_path.to_string_lossy().as_ref(),
        )
    });
}
