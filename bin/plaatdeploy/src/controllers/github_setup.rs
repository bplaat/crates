/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Team-owned GitHub App manifest callback flow

use std::process::Command;

use anyhow::Result;
use serde::Deserialize;
use small_http::{Request, Response, Status};

use crate::context::{Context, DatabaseHelpers};
use crate::github;

// MARK: Setup callback
#[derive(Deserialize)]
struct SetupQuery {
    code: String,
    state: String,
}

pub(crate) fn github_setup(req: &Request, ctx: &Context) -> Result<Response> {
    let query: SetupQuery = match serde_urlencoded::from_str(req.url.query().unwrap_or("")) {
        Ok(q) => q,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };

    let connection = match ctx
        .database
        .find_team_github_connection_by_setup_state(&query.state)?
    {
        Some(connection) => connection,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    // Exchange code for app credentials
    let url = format!(
        "https://api.github.com/app-manifests/{}/conversions",
        query.code
    );
    let output = Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "X-GitHub-Api-Version: 2022-11-28",
            "-H",
            "User-Agent: plaatdeploy",
            &url,
        ])
        .output()?;

    let val: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(v) => v,
        Err(_) => {
            log::warn!("GitHub App manifest conversion returned invalid JSON");
            return Ok(Response::new().status(Status::BadGateway));
        }
    };

    if let Some(msg) = val["message"].as_str() {
        log::warn!("GitHub App manifest conversion error: {msg}");
        return Ok(Response::new().status(Status::BadGateway));
    }

    let app_id = match val["id"].as_u64() {
        Some(id) => id.to_string(),
        None => {
            log::warn!("GitHub App manifest conversion: missing id");
            return Ok(Response::new().status(Status::BadGateway));
        }
    };
    let pem = match val["pem"].as_str() {
        Some(p) => p.to_string(),
        None => {
            log::warn!("GitHub App manifest conversion: missing pem");
            return Ok(Response::new().status(Status::BadGateway));
        }
    };
    let webhook_secret = match val["webhook_secret"].as_str() {
        Some(s) => s.to_string(),
        None => {
            log::warn!("GitHub App manifest conversion: missing webhook_secret");
            return Ok(Response::new().status(Status::BadGateway));
        }
    };
    let app_slug = val["slug"].as_str().map(str::to_string);

    ctx.database.execute(
        "UPDATE team_github_connections
         SET app_id = ?, app_private_key = ?, webhook_secret = ?, app_slug = ?, setup_state = NULL, installation_id = NULL, account_login = NULL, account_type = NULL, updated_at = ?
         WHERE id = ?",
        (
            app_id.clone(),
            pem,
            webhook_secret,
            app_slug,
            chrono::Utc::now(),
            connection.id,
        ),
    )?;

    github::clear_token_cache();

    log::info!(
        "Team GitHub App configured successfully (team_id: {}, app_id: {app_id})",
        connection.team_id
    );
    Ok(Response::new()
        .status(Status::Found)
        .header("Location", "/teams"))
}
