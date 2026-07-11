/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Team-owned GitHub fine-grained personal access token connections.

use anyhow::Result;
use chrono::Utc;
use from_derive::FromStruct;
use serde::Deserialize;
use small_http::{Request, Response, Status};
use uuid::Uuid;
use validate::Validate;

use super::parse_body;
use crate::context::{Context, DatabaseHelpers};
use crate::models::{TeamGitHubToken, UserRole};
use crate::{api, github};

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

fn webhook_secret() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("Failed to generate webhook secret");
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn status_response(ctx: &Context, team_id: Uuid) -> Result<api::TeamGithubStatusResponse> {
    let connection = ctx.database.find_team_github_token_by_team_id(team_id)?;
    let connected = connection
        .as_ref()
        .is_some_and(|token| !token.access_token.is_empty());
    Ok(api::TeamGithubStatusResponse {
        connected,
        connection: connection
            .filter(|token| !token.access_token.is_empty())
            .map(Into::into),
    })
}

fn team_id(
    req: &Request,
    ctx: &Context,
    manage: bool,
) -> Result<std::result::Result<Uuid, Response>> {
    let Some(team_id) = req.params.get("team_id").and_then(|id| id.parse().ok()) else {
        return Ok(Err(Response::new().status(Status::BadRequest)));
    };
    if ctx.database.find_team_by_id(team_id)?.is_none() {
        return Ok(Err(Response::new().status(Status::NotFound)));
    }
    if if manage {
        !can_manage_team(ctx, team_id)?
    } else {
        !can_access_team(ctx, team_id)?
    } {
        return Ok(Err(Response::new().status(Status::Forbidden)));
    }
    Ok(Ok(team_id))
}

pub(crate) fn teams_github_show(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match team_id(req, ctx, false)? {
        Ok(team_id) => team_id,
        Err(response) => return Ok(response),
    };
    Ok(Response::with_json(status_response(ctx, team_id)?))
}

pub(crate) fn teams_github_update(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match team_id(req, ctx, true)? {
        Ok(team_id) => team_id,
        Err(response) => return Ok(response),
    };

    #[derive(Validate, FromStruct)]
    #[from_struct(api::TeamGithubConnectBody)]
    struct Body {
        #[validate(length(min = 1, max = 512))]
        access_token: String,
    }
    let body = parse_body!(req, api::TeamGithubConnectBody, Body);
    let access_token = body.access_token.trim().to_string();
    let Some(account_login) = github::account_login(&access_token) else {
        return Ok(Response::new().status(Status::BadRequest).json(serde_json::json!({
            "accessToken": ["GitHub rejected this token. Check that it is active and has Metadata: Read access."]
        })));
    };

    let now = Utc::now();
    let existing = ctx.database.find_team_github_token_by_team_id(team_id)?;
    let token = TeamGitHubToken {
        id: existing
            .as_ref()
            .map(|connection| connection.id)
            .unwrap_or_else(Uuid::now_v7),
        team_id,
        access_token,
        webhook_secret: existing
            .as_ref()
            .map(|token| token.webhook_secret.clone())
            .unwrap_or_else(webhook_secret),
        account_login,
        created_at: existing
            .as_ref()
            .map(|connection| connection.created_at)
            .unwrap_or(now),
        updated_at: now,
    };
    ctx.database.execute(
        const_format::formatcp!(
            "INSERT INTO team_github_tokens ({}) VALUES ({})\n             ON CONFLICT(team_id) DO UPDATE SET access_token = excluded.access_token, webhook_secret = excluded.webhook_secret, account_login = excluded.account_login, updated_at = excluded.updated_at",
            TeamGitHubToken::columns(),
            TeamGitHubToken::values()
        ),
        token,
    )?;
    Ok(Response::with_json(status_response(ctx, team_id)?))
}

pub(crate) fn teams_github_delete(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match team_id(req, ctx, true)? {
        Ok(team_id) => team_id,
        Err(response) => return Ok(response),
    };
    ctx.database
        .execute("DELETE FROM team_github_tokens WHERE team_id = ?", team_id)?;
    Ok(Response::new())
}

pub(crate) fn teams_github_repositories(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match team_id(req, ctx, false)? {
        Ok(team_id) => team_id,
        Err(response) => return Ok(response),
    };
    if !github::team_is_connected(ctx, team_id)? {
        return Ok(Response::with_json(api::GithubRepositoryIndexResponse {
            data: Vec::new(),
        }));
    }
    let Some(repositories) = github::list_repositories(ctx, team_id) else {
        return Ok(Response::new().status(Status::BadGateway));
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

    let team_id = match team_id(req, ctx, false)? {
        Ok(team_id) => team_id,
        Err(response) => return Ok(response),
    };
    let query = match req
        .url
        .query()
        .and_then(|query| serde_urlencoded::from_str::<Query>(query).ok())
    {
        Some(query) if !query.repository.is_empty() => query,
        _ => return Ok(Response::new().status(Status::BadRequest)),
    };
    if !github::team_is_connected(ctx, team_id)? {
        return Ok(Response::with_json(api::GithubBranchIndexResponse {
            data: Vec::new(),
        }));
    }
    let Some(branches) = github::list_branches(ctx, team_id, &query.repository) else {
        return Ok(Response::new().status(Status::BadGateway));
    };
    Ok(Response::with_json(api::GithubBranchIndexResponse {
        data: branches
            .into_iter()
            .map(|branch| api::GithubBranch { name: branch.name })
            .collect(),
    }))
}
