/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use chrono::Utc;
use const_format::formatcp;
use small_http::{Request, Response, Status};

use super::parse_pagination;
use crate::api;
use crate::context::{Context, DatabaseHelpers};
use crate::models::{Session, UserRole};

pub(crate) fn sessions_index(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    let now = Utc::now();

    let q = match parse_pagination(req) {
        Ok(q) => q,
        Err(r) => return Ok(r),
    };
    let page = q.page;
    let limit = q.limit;
    let offset = (page - 1) * limit;

    let (total, sessions) = if auth_user.role == UserRole::Admin {
        let total = ctx
            .database
            .query_some::<i64>("SELECT COUNT(id) FROM sessions WHERE expires_at > ?", now)?;
        let sessions: Vec<api::Session> = ctx
            .database
            .query::<Session>(
                formatcp!(
                    "SELECT {} FROM sessions WHERE expires_at > ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
                    Session::columns()
                ),
                (now, limit, offset),
            )?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Into::into)
            .collect();
        (total, sessions)
    } else {
        let total = ctx.database.query_some::<i64>(
            "SELECT COUNT(id) FROM sessions WHERE user_id = ? AND expires_at > ?",
            (auth_user.id, now),
        )?;
        let sessions: Vec<api::Session> = ctx
            .database
            .query::<Session>(
                formatcp!(
                    "SELECT {} FROM sessions WHERE user_id = ? AND expires_at > ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
                    Session::columns()
                ),
                (auth_user.id, now, limit, offset),
            )?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Into::into)
            .collect();
        (total, sessions)
    };

    Ok(Response::with_json(api::SessionIndexResponse {
        pagination: api::Pagination { page, limit, total },
        data: sessions,
    }))
}

pub(crate) fn sessions_delete(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");

    let session_id = match req.params.get("session_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let session = match ctx.database.find_session_by_id(session_id)? {
        Some(s) => s,
        None => return Ok(Response::new().status(Status::NotFound)),
    };

    if auth_user.role != UserRole::Admin && auth_user.id != session.user_id {
        return Ok(Response::new().status(Status::Forbidden));
    }

    ctx.database.execute(
        "UPDATE sessions SET expires_at = ?, updated_at = ? WHERE id = ?",
        (Utc::now(), Utc::now(), session.id),
    )?;

    Ok(Response::new())
}
