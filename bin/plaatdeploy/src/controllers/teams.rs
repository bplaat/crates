/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use chrono::Utc;
use const_format::formatcp;
use serde::Deserialize;
use small_http::{Request, Response, Status};
use uuid::Uuid;

use super::parse_pagination;
use crate::api;
use crate::context::{Context, DatabaseHelpers};
use crate::models::{Team, TeamUser, TeamUserRole, TeamUserRow, UserRole};

fn parse_team_user_role(role: Option<&str>) -> TeamUserRole {
    match role {
        Some("owner") => TeamUserRole::Owner,
        _ => TeamUserRole::Member,
    }
}

fn team_users(ctx: &Context, team_id: Uuid) -> Result<Vec<api::TeamUser>> {
    Ok(ctx
        .database
        .query::<TeamUserRow>(
            "SELECT team_users.id, team_users.team_id, team_users.user_id, users.first_name, users.last_name, users.email, team_users.role, team_users.created_at, team_users.updated_at
              FROM team_users
              INNER JOIN users ON users.id = team_users.user_id
              WHERE team_users.team_id = ?
              ORDER BY team_users.role DESC, users.first_name ASC, users.last_name ASC, users.email ASC",
            team_id,
        )?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(Into::into)
        .collect())
}

fn can_manage_team(ctx: &Context, team_id: Uuid) -> Result<bool> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    if auth_user.role == UserRole::Admin {
        return Ok(true);
    }
    ctx.database.user_is_team_owner(auth_user.id, team_id)
}

fn can_access_team(ctx: &Context, team_id: Uuid) -> Result<bool> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    if auth_user.role == UserRole::Admin {
        return Ok(true);
    }
    ctx.database.user_is_team_member(auth_user.id, team_id)
}

pub(crate) fn teams_index(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");

    let q = match parse_pagination(req) {
        Ok(q) => q,
        Err(r) => return Ok(r),
    };
    let page = q.page;
    let limit = q.limit;
    let offset = (page - 1) * limit;

    let (total, teams) = if auth_user.role == UserRole::Admin {
        let total = ctx
            .database
            .query_some::<i64>("SELECT COUNT(id) FROM teams", ())?;
        let teams: Vec<api::Team> = ctx
            .database
            .query::<Team>(
                formatcp!(
                    "SELECT {} FROM teams ORDER BY created_at DESC LIMIT ? OFFSET ?",
                    Team::columns()
                ),
                (limit, offset),
            )?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Into::into)
            .collect();
        (total, teams)
    } else {
        let total = ctx.database.query_some::<i64>(
            "SELECT COUNT(teams.id)
             FROM teams
             INNER JOIN team_users ON team_users.team_id = teams.id
             WHERE team_users.user_id = ?",
            auth_user.id,
        )?;
        let teams: Vec<api::Team> = ctx
            .database
            .query::<Team>(
                formatcp!(
                    "SELECT {} FROM teams
                     INNER JOIN team_users ON team_users.team_id = teams.id
                     WHERE team_users.user_id = ?
                     ORDER BY teams.created_at DESC
                     LIMIT ? OFFSET ?",
                    Team::columns()
                ),
                (auth_user.id, limit, offset),
            )?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Into::into)
            .collect();
        (total, teams)
    };

    Ok(Response::with_json(api::TeamIndexResponse {
        pagination: api::Pagination { page, limit, total },
        data: teams,
    }))
}

pub(crate) fn teams_create(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");

    #[derive(Deserialize)]
    struct Body {
        name: String,
    }

    let body = match serde_urlencoded::from_bytes::<Body>(req.body.as_deref().unwrap_or(&[])) {
        Ok(b) => b,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };
    if body.name.is_empty() {
        return Ok(Response::new().status(Status::BadRequest));
    }

    let team = Team {
        name: body.name,
        ..Default::default()
    };
    ctx.database
        .create_team_with_owner(team.clone(), auth_user.id)?;

    Ok(Response::with_json(api::Team::from(team)))
}

pub(crate) fn teams_show(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match req.params.get("team_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let team = match ctx.database.find_team_by_id(team_id)? {
        Some(team) => team,
        None => return Ok(Response::new().status(Status::NotFound)),
    };
    if !can_access_team(ctx, team.id)? {
        return Ok(Response::new().status(Status::Forbidden));
    }

    Ok(Response::with_json(api::TeamShowResponse {
        team: team.clone().into(),
        members: team_users(ctx, team.id)?,
    }))
}

pub(crate) fn teams_update(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match req.params.get("team_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let mut team = match ctx.database.find_team_by_id(team_id)? {
        Some(team) => team,
        None => return Ok(Response::new().status(Status::NotFound)),
    };
    if !can_manage_team(ctx, team.id)? {
        return Ok(Response::new().status(Status::Forbidden));
    }

    #[derive(Deserialize)]
    struct Body {
        name: String,
    }

    let body = match serde_urlencoded::from_bytes::<Body>(req.body.as_deref().unwrap_or(&[])) {
        Ok(b) => b,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };
    if body.name.is_empty() {
        return Ok(Response::new().status(Status::BadRequest));
    }

    team.name = body.name;
    team.updated_at = Utc::now();
    ctx.database.execute(
        "UPDATE teams SET name = ?, updated_at = ? WHERE id = ?",
        (team.name.clone(), team.updated_at, team.id),
    )?;

    Ok(Response::with_json(api::Team::from(team)))
}

pub(crate) fn teams_delete(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match req.params.get("team_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let team = match ctx.database.find_team_by_id(team_id)? {
        Some(team) => team,
        None => return Ok(Response::new().status(Status::NotFound)),
    };
    if !can_manage_team(ctx, team.id)? {
        return Ok(Response::new().status(Status::Forbidden));
    }
    let project_count = ctx
        .database
        .query_some::<i64>("SELECT COUNT(id) FROM projects WHERE team_id = ?", team.id)?;
    if project_count != 0 {
        return Ok(Response::new()
            .status(Status::BadRequest)
            .json(serde_json::json!({ "team": ["Move or delete the team's projects first"] })));
    }

    ctx.database
        .execute("DELETE FROM team_users WHERE team_id = ?", team.id)?;
    ctx.database
        .execute("DELETE FROM teams WHERE id = ?", team.id)?;
    Ok(Response::new())
}

pub(crate) fn teams_members_create(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match req.params.get("team_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let team = match ctx.database.find_team_by_id(team_id)? {
        Some(team) => team,
        None => return Ok(Response::new().status(Status::NotFound)),
    };
    if !can_manage_team(ctx, team.id)? {
        return Ok(Response::new().status(Status::Forbidden));
    }
    #[derive(Deserialize)]
    struct Body {
        email: String,
        role: Option<String>,
    }

    let body = match serde_urlencoded::from_bytes::<Body>(req.body.as_deref().unwrap_or(&[])) {
        Ok(b) => b,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };
    let user = match ctx.database.find_user_by_email(&body.email)? {
        Some(user) => user,
        None => {
            return Ok(Response::new()
                .status(Status::BadRequest)
                .json(serde_json::json!({ "email": ["User not found"] })));
        }
    };
    if ctx.database.user_is_team_member(user.id, team.id)? {
        return Ok(Response::new()
            .status(Status::BadRequest)
            .json(serde_json::json!({ "email": ["User is already a team member"] })));
    }

    let now = Utc::now();
    ctx.database.insert_team_user(TeamUser {
        team_id: team.id,
        user_id: user.id,
        role: parse_team_user_role(body.role.as_deref()),
        created_at: now,
        updated_at: now,
        ..Default::default()
    })?;

    let team_user = ctx
        .database
        .query::<TeamUserRow>(
            "SELECT team_users.id, team_users.team_id, team_users.user_id, users.first_name, users.last_name, users.email, team_users.role, team_users.created_at, team_users.updated_at
              FROM team_users
              INNER JOIN users ON users.id = team_users.user_id
              WHERE team_users.team_id = ? AND team_users.user_id = ? LIMIT 1",
            (team.id, user.id),
        )?
        .next()
        .transpose()?
        .expect("created team user should exist");

    Ok(Response::with_json(api::TeamUser::from(team_user)))
}

pub(crate) fn teams_members_update(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match req.params.get("team_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };
    let user_id = match req.params.get("user_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let team = match ctx.database.find_team_by_id(team_id)? {
        Some(team) => team,
        None => return Ok(Response::new().status(Status::NotFound)),
    };
    if !can_manage_team(ctx, team.id)? {
        return Ok(Response::new().status(Status::Forbidden));
    }
    #[derive(Deserialize)]
    struct Body {
        role: String,
    }

    let body = match serde_urlencoded::from_bytes::<Body>(req.body.as_deref().unwrap_or(&[])) {
        Ok(b) => b,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };
    let role = parse_team_user_role(Some(&body.role));
    let current_role = match ctx.database.team_user_role(user_id, team.id)? {
        Some(role) => role,
        None => return Ok(Response::new().status(Status::NotFound)),
    };
    if current_role == TeamUserRole::Owner && role != TeamUserRole::Owner {
        let owner_count = ctx.database.query_some::<i64>(
            "SELECT COUNT(user_id) FROM team_users WHERE team_id = ? AND role = ?",
            (team.id, TeamUserRole::Owner as i64),
        )?;
        if owner_count <= 1 {
            return Ok(Response::new()
                .status(Status::BadRequest)
                .json(serde_json::json!({ "role": ["Teams must keep at least one owner"] })));
        }
    }

    ctx.database.execute(
        "UPDATE team_users SET role = ?, updated_at = ? WHERE team_id = ? AND user_id = ?",
        (role as i64, Utc::now(), team.id, user_id),
    )?;

    let team_user = ctx
        .database
        .query::<TeamUserRow>(
            "SELECT team_users.id, team_users.team_id, team_users.user_id, users.first_name, users.last_name, users.email, team_users.role, team_users.created_at, team_users.updated_at
              FROM team_users
              INNER JOIN users ON users.id = team_users.user_id
              WHERE team_users.team_id = ? AND team_users.user_id = ? LIMIT 1",
            (team.id, user_id),
        )?
        .next()
        .transpose()?
        .expect("updated team user should exist");

    Ok(Response::with_json(api::TeamUser::from(team_user)))
}

pub(crate) fn teams_members_delete(req: &Request, ctx: &Context) -> Result<Response> {
    let team_id = match req.params.get("team_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };
    let user_id = match req.params.get("user_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let team = match ctx.database.find_team_by_id(team_id)? {
        Some(team) => team,
        None => return Ok(Response::new().status(Status::NotFound)),
    };
    if !can_manage_team(ctx, team.id)? {
        return Ok(Response::new().status(Status::Forbidden));
    }
    let current_role = match ctx.database.team_user_role(user_id, team.id)? {
        Some(role) => role,
        None => return Ok(Response::new().status(Status::NotFound)),
    };
    if current_role == TeamUserRole::Owner {
        let owner_count = ctx.database.query_some::<i64>(
            "SELECT COUNT(user_id) FROM team_users WHERE team_id = ? AND role = ?",
            (team.id, TeamUserRole::Owner as i64),
        )?;
        if owner_count <= 1 {
            return Ok(Response::new()
                .status(Status::BadRequest)
                .json(serde_json::json!({ "team": ["Teams must keep at least one owner"] })));
        }
    }

    ctx.database.execute(
        "DELETE FROM team_users WHERE team_id = ? AND user_id = ?",
        (team.id, user_id),
    )?;
    Ok(Response::new())
}
