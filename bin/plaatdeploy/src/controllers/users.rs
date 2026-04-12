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

use super::parse_pagination;
use crate::api;
use crate::context::{Context, DatabaseHelpers};
use crate::models::{User, UserRole};

pub(crate) fn users_index(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    if auth_user.role != UserRole::Admin {
        return Ok(Response::new().status(Status::Forbidden));
    }

    let q = match parse_pagination(req) {
        Ok(q) => q,
        Err(r) => return Ok(r),
    };
    let page = q.page;
    let limit = q.limit;
    let offset = (page - 1) * limit;

    let total = ctx
        .database
        .query_some::<i64>("SELECT COUNT(id) FROM users", ())?;
    let users: Vec<api::User> = ctx
        .database
        .query::<User>(
            formatcp!(
                "SELECT {} FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?",
                User::columns()
            ),
            (limit, offset),
        )?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Response::with_json(api::UserIndexResponse {
        pagination: api::Pagination { page, limit, total },
        data: users,
    }))
}

pub(crate) fn users_create(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    if auth_user.role != UserRole::Admin {
        return Ok(Response::new().status(Status::Forbidden));
    }

    #[derive(Deserialize)]
    struct Body {
        first_name: String,
        last_name: String,
        email: String,
        password: String,
        role: Option<String>,
    }

    let body = match serde_urlencoded::from_bytes::<Body>(req.body.as_deref().unwrap_or(&[])) {
        Ok(b) => b,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };

    // Check email uniqueness
    let count = ctx.database.query_some::<i64>(
        "SELECT COUNT(id) FROM users WHERE email = ?",
        body.email.clone(),
    )?;
    if count != 0 {
        return Ok(Response::new()
            .status(Status::BadRequest)
            .json(serde_json::json!({
                "email": ["Email is already taken"]
            })));
    }

    let role = match body.role.as_deref() {
        Some("admin") => UserRole::Admin,
        _ => UserRole::Normal,
    };

    let user = User {
        first_name: body.first_name,
        last_name: body.last_name,
        email: body.email,
        password: pbkdf2::password_hash(&body.password),
        role,
        ..Default::default()
    };
    ctx.database.create_user_with_default_team(user.clone())?;

    Ok(Response::with_json(api::User::from(user)))
}

pub(crate) fn users_show(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    if auth_user.role != UserRole::Admin {
        return Ok(Response::new().status(Status::Forbidden));
    }

    let user_id = match req.params.get("user_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    match ctx.database.find_user_by_id(user_id)? {
        Some(user) => Ok(Response::with_json(api::User::from(user))),
        None => Ok(Response::new().status(Status::NotFound)),
    }
}

pub(crate) fn users_update(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    let user_id = match req.params.get("user_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };
    let is_self = auth_user.id == user_id;
    if auth_user.role != UserRole::Admin && !is_self {
        return Ok(Response::new().status(Status::Forbidden));
    }

    let mut user = match ctx.database.find_user_by_id(user_id)? {
        Some(u) => u,
        None => return Ok(Response::new().status(Status::NotFound)),
    };

    #[derive(Deserialize)]
    struct Body {
        first_name: String,
        last_name: String,
        email: String,
        password: Option<String>,
        role: Option<String>,
    }

    let body = match serde_urlencoded::from_bytes::<Body>(req.body.as_deref().unwrap_or(&[])) {
        Ok(b) => b,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };

    if let Some(existing_user) = ctx.database.find_user_by_email(&body.email)?
        && existing_user.id != user.id
    {
        return Ok(Response::new()
            .status(Status::BadRequest)
            .json(serde_json::json!({
                "email": ["Email is already taken"]
            })));
    }

    user.first_name = body.first_name;
    user.last_name = body.last_name;
    user.email = body.email;
    if auth_user.role == UserRole::Admin
        && let Some(pw) = body.password.filter(|p| !p.is_empty())
    {
        user.password = pbkdf2::password_hash(&pw);
    }
    if auth_user.role == UserRole::Admin
        && let Some(role_str) = body.role
    {
        user.role = match role_str.as_str() {
            "admin" => UserRole::Admin,
            _ => UserRole::Normal,
        };
    }
    user.updated_at = Utc::now();

    ctx.database.execute(
        "UPDATE users SET first_name = ?, last_name = ?, email = ?, password = ?, role = ?, updated_at = ? WHERE id = ?",
        (
            user.first_name.clone(),
            user.last_name.clone(),
            user.email.clone(),
            user.password.clone(),
            user.role as i64,
            user.updated_at,
            user.id,
        ),
    )?;

    Ok(Response::with_json(api::User::from(user)))
}

pub(crate) fn users_change_password(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    let user_id = match req.params.get("user_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };
    if auth_user.id != user_id {
        return Ok(Response::new().status(Status::Forbidden));
    }

    #[derive(Deserialize)]
    struct Body {
        old_password: String,
        new_password: String,
    }

    let body = match serde_urlencoded::from_bytes::<Body>(req.body.as_deref().unwrap_or(&[])) {
        Ok(b) => b,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };

    let mut user = match ctx.database.find_user_by_id(user_id)? {
        Some(u) => u,
        None => return Ok(Response::new().status(Status::NotFound)),
    };

    match pbkdf2::password_verify(&body.old_password, &user.password) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(Response::new()
                .status(Status::BadRequest)
                .json(serde_json::json!({
                    "old_password": ["Current password is incorrect"]
                })));
        }
        Err(_) => return Ok(Response::new().status(Status::InternalServerError)),
    }

    if body.new_password.is_empty() {
        return Ok(Response::new()
            .status(Status::BadRequest)
            .json(serde_json::json!({
                "new_password": ["New password is required"]
            })));
    }

    user.password = pbkdf2::password_hash(&body.new_password);
    user.updated_at = Utc::now();

    ctx.database.execute(
        "UPDATE users SET password = ?, updated_at = ? WHERE id = ?",
        (user.password, user.updated_at, user.id),
    )?;

    Ok(Response::new())
}

pub(crate) fn users_delete(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    if auth_user.role != UserRole::Admin {
        return Ok(Response::new().status(Status::Forbidden));
    }

    let user_id = match req.params.get("user_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    match ctx.database.find_user_by_id(user_id)? {
        Some(_) => {
            let team_count = ctx.database.query_some::<i64>(
                "SELECT COUNT(team_id) FROM team_users WHERE user_id = ?",
                user_id,
            )?;
            if team_count != 0 {
                return Ok(Response::new()
                    .status(Status::BadRequest)
                    .json(serde_json::json!({
                        "user": ["Remove the user from all teams first"]
                    })));
            }

            ctx.database
                .execute("DELETE FROM sessions WHERE user_id = ?", user_id)?;
            ctx.database
                .execute("DELETE FROM team_users WHERE user_id = ?", user_id)?;
            ctx.database
                .execute("DELETE FROM users WHERE id = ?", user_id)?;
            Ok(Response::new())
        }
        None => Ok(Response::new().status(Status::NotFound)),
    }
}
