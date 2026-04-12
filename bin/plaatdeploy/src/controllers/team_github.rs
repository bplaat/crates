/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use base64::Engine as _;
use chrono::Utc;
use serde::Deserialize;
use small_http::{Request, Response, Status};
use uuid::Uuid;

use crate::context::{Context, DatabaseHelpers};
use crate::models::UserRole;
use crate::{api, github};

fn generate_state() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("Failed to generate random bytes");
    base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(bytes)
}

fn can_access_team(ctx: &Context, team_id: Uuid) -> Result<bool> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    if auth_user.role == UserRole::Admin {
        return Ok(true);
    }
    ctx.database.user_is_team_member(auth_user.id, team_id)
}

fn can_manage_team(ctx: &Context, team_id: Uuid) -> Result<bool> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    if auth_user.role == UserRole::Admin {
        return Ok(true);
    }
    ctx.database.user_is_team_owner(auth_user.id, team_id)
}

fn team_github_status_response(
    ctx: &Context,
    team_id: Uuid,
) -> Result<api::TeamGithubStatusResponse> {
    let explicit_connection = ctx
        .database
        .find_team_github_connection_by_team_id(team_id)?;
    let app_configured = explicit_connection
        .as_ref()
        .map(|connection| {
            connection.app_id.is_some()
                && connection.app_private_key.is_some()
                && connection.webhook_secret.is_some()
        })
        .unwrap_or(false);
    let installations = if app_configured {
        github::list_installations(ctx, team_id)
            .unwrap_or_default()
            .into_iter()
            .map(|installation| api::GithubInstallation {
                id: installation.id,
                account_login: installation.account_login,
                account_type: installation.account_type,
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(api::TeamGithubStatusResponse {
        app_configured,
        connected: explicit_connection
            .as_ref()
            .and_then(|connection| connection.installation_id)
            .is_some(),
        inherited_connection: false,
        install_url: explicit_connection.as_ref().and_then(|connection| {
            connection
                .app_slug
                .as_ref()
                .map(|slug| format!("https://github.com/apps/{slug}/installations/new"))
        }),
        connection: explicit_connection
            .filter(|connection| {
                connection.installation_id.is_some() && connection.account_login.is_some()
            })
            .map(Into::into),
        installations,
    })
}

pub(crate) fn teams_github_setup_start(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match req.params.get("team_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };
    if ctx.database.find_team_by_id(team_id)?.is_none() {
        return Ok(Response::new().status(Status::NotFound));
    }
    if !can_manage_team(ctx, team_id)? {
        return Ok(Response::new().status(Status::Forbidden));
    }

    let origin = &ctx.server_origin;
    let now = Utc::now();
    let state = generate_state();
    let existing_connection = ctx
        .database
        .find_team_github_connection_by_team_id(team_id)?;
    let created_at = existing_connection
        .as_ref()
        .map(|connection| connection.created_at)
        .unwrap_or(now);
    ctx.database.execute(
        "INSERT INTO team_github_connections (id, team_id, app_id, app_private_key, webhook_secret, app_slug, setup_state, installation_id, account_login, account_type, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(team_id) DO UPDATE SET setup_state = excluded.setup_state, updated_at = excluded.updated_at",
        (
            existing_connection
                .map(|connection| connection.id)
                .unwrap_or_else(Uuid::now_v7),
            team_id,
            Option::<String>::None,
            Option::<String>::None,
            Option::<String>::None,
            Option::<String>::None,
            Some(state.clone()),
            Option::<i64>::None,
            Option::<String>::None,
            Option::<String>::None,
            created_at,
            now,
        ),
    )?;

    let manifest = serde_json::json!({
        "name": format!("PlaatDeploy ({origin}) - {team_id}"),
        "url": origin,
        "hook_attributes": {
            "url": format!("{origin}/api/webhook/github"),
            "active": true
        },
        "redirect_url": format!("{origin}/api/github/setup"),
        "description": "Minimal self-hosted deployment service companion app",
        "public": false,
        "default_permissions": {
            "metadata": "read",
            "contents": "read",
            "actions": "read",
            "deployments": "write"
        },
        "default_events": ["push", "workflow_run"]
    });

    Ok(Response::new().json(serde_json::json!({
        "state": state,
        "manifest": manifest
    })))
}

pub(crate) fn teams_github_show(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match req.params.get("team_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };
    if ctx.database.find_team_by_id(team_id)?.is_none() {
        return Ok(Response::new().status(Status::NotFound));
    }
    if !can_access_team(ctx, team_id)? {
        return Ok(Response::new().status(Status::Forbidden));
    }

    Ok(Response::with_json(team_github_status_response(
        ctx, team_id,
    )?))
}

pub(crate) fn teams_github_update(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match req.params.get("team_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };
    if ctx.database.find_team_by_id(team_id)?.is_none() {
        return Ok(Response::new().status(Status::NotFound));
    }
    if !can_manage_team(ctx, team_id)? {
        return Ok(Response::new().status(Status::Forbidden));
    }

    #[derive(Deserialize)]
    struct Body {
        installation_id: i64,
    }

    let body = match serde_urlencoded::from_bytes::<Body>(req.body.as_deref().unwrap_or(&[])) {
        Ok(body) => body,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };

    let installation = match github::list_installations(ctx, team_id)
        .unwrap_or_default()
        .into_iter()
        .find(|installation| installation.id == body.installation_id)
    {
        Some(installation) => installation,
        None => {
            return Ok(Response::new()
                .status(Status::BadRequest)
                .json(serde_json::json!({
                    "installationId": ["Installation not found"]
                })));
        }
    };

    let now = Utc::now();
    let existing_connection = ctx
        .database
        .find_team_github_connection_by_team_id(team_id)?;
    let created_at = existing_connection
        .as_ref()
        .map(|connection| connection.created_at)
        .unwrap_or(now);
    ctx.database.execute(
        "INSERT INTO team_github_connections (id, team_id, app_id, app_private_key, webhook_secret, app_slug, setup_state, installation_id, account_login, account_type, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
          ON CONFLICT(team_id) DO UPDATE SET installation_id = excluded.installation_id, account_login = excluded.account_login, account_type = excluded.account_type, updated_at = excluded.updated_at",
        (
            existing_connection.clone().map(|connection| connection.id).unwrap_or_else(Uuid::now_v7),
            team_id,
            existing_connection.as_ref().and_then(|connection| connection.app_id.clone()),
            existing_connection
                .as_ref()
                .and_then(|connection| connection.app_private_key.clone()),
            existing_connection
                .as_ref()
                .and_then(|connection| connection.webhook_secret.clone()),
            existing_connection.as_ref().and_then(|connection| connection.app_slug.clone()),
            existing_connection.as_ref().and_then(|connection| connection.setup_state.clone()),
            installation.id,
            Some(installation.account_login),
            installation.account_type,
            created_at,
            now,
        ),
    )?;
    github::clear_token_cache();

    Ok(Response::with_json(team_github_status_response(
        ctx, team_id,
    )?))
}

pub(crate) fn teams_github_delete(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match req.params.get("team_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };
    if ctx.database.find_team_by_id(team_id)?.is_none() {
        return Ok(Response::new().status(Status::NotFound));
    }
    if !can_manage_team(ctx, team_id)? {
        return Ok(Response::new().status(Status::Forbidden));
    }

    ctx.database.execute(
        "UPDATE team_github_connections SET installation_id = NULL, account_login = NULL, account_type = NULL, updated_at = ? WHERE team_id = ?",
        (Utc::now(), team_id),
    )?;
    github::clear_token_cache();
    Ok(Response::new())
}

pub(crate) fn teams_github_repositories(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match req.params.get("team_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };
    if ctx.database.find_team_by_id(team_id)?.is_none() {
        return Ok(Response::new().status(Status::NotFound));
    }
    if !can_access_team(ctx, team_id)? {
        return Ok(Response::new().status(Status::Forbidden));
    }

    let installation = match github::team_installation(ctx, team_id)? {
        Some(installation) => installation,
        None => {
            return Ok(Response::with_json(api::GithubRepositoryIndexResponse {
                data: Vec::new(),
            }));
        }
    };
    let repositories = match github::list_repositories(ctx, installation.id) {
        Some(repositories) => repositories,
        None => return Ok(Response::new().status(Status::BadGateway)),
    };

    Ok(Response::with_json(api::GithubRepositoryIndexResponse {
        data: repositories
            .into_iter()
            .map(|repository| api::GithubRepository {
                full_name: repository.full_name,
            })
            .collect(),
    }))
}

pub(crate) fn teams_github_branches(req: &Request, ctx: &Context) -> Result<Response> {
    #[derive(Deserialize)]
    struct Query {
        repository: String,
    }

    let team_id = match req.params.get("team_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };
    if ctx.database.find_team_by_id(team_id)?.is_none() {
        return Ok(Response::new().status(Status::NotFound));
    }
    if !can_access_team(ctx, team_id)? {
        return Ok(Response::new().status(Status::Forbidden));
    }

    let query = match req.url.query() {
        Some(query) => match serde_urlencoded::from_str::<Query>(query) {
            Ok(query) if !query.repository.is_empty() => query,
            _ => return Ok(Response::new().status(Status::BadRequest)),
        },
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let installation = match github::team_installation(ctx, team_id)? {
        Some(installation) => installation,
        None => {
            return Ok(Response::with_json(api::GithubBranchIndexResponse {
                data: Vec::new(),
            }));
        }
    };
    let branches = match github::list_branches(ctx, installation.id, &query.repository) {
        Some(branches) => branches,
        None => return Ok(Response::new().status(Status::BadGateway)),
    };

    Ok(Response::with_json(api::GithubBranchIndexResponse {
        data: branches
            .into_iter()
            .map(|branch| api::GithubBranch { name: branch.name })
            .collect(),
    }))
}
