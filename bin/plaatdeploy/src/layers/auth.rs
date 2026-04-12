/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Bearer token auth pre-layers

use std::time::Duration;

use anyhow::Result;
use bsqlite::Connection;
use chrono::Utc;
use const_format::formatcp;
use small_http::{Request, Response, Status};

use crate::consts::{SESSION_EXPIRY_SECONDS, SESSION_REFRESH_THRESHOLD_SECONDS};
use crate::context::Context;
use crate::models::{Session, User};

// MARK: Auth optional
pub(crate) fn auth_optional_pre_layer(
    req: &Request,
    ctx: &mut Context,
) -> Option<Result<Response>> {
    let authorization = req
        .headers
        .get("Authorization")
        .or(req.headers.get("authorization"))?;
    let token = authorization.strip_prefix("Bearer ")?.trim();

    if let Some((session, user)) = lookup_session_and_user(token, &ctx.database) {
        ctx.auth_session = Some(session);
        ctx.auth_user = Some(user);
    }

    None
}

// MARK: Auth required
pub(crate) fn auth_required_pre_layer(
    req: &Request,
    ctx: &mut Context,
) -> Option<Result<Response>> {
    let authorization = match req
        .headers
        .get("Authorization")
        .or(req.headers.get("authorization"))
    {
        Some(a) => a,
        None => {
            return Some(Ok(Response::new()
                .status(Status::Unauthorized)
                .body("401 Unauthorized")));
        }
    };
    let token = match authorization.strip_prefix("Bearer ") {
        Some(t) => t.trim(),
        None => {
            return Some(Ok(Response::new()
                .status(Status::Unauthorized)
                .body("401 Unauthorized")));
        }
    };

    match lookup_session_and_user(token, &ctx.database) {
        Some((session, user)) => {
            ctx.auth_session = Some(session);
            ctx.auth_user = Some(user);
            None
        }
        None => Some(Ok(Response::new()
            .status(Status::Unauthorized)
            .body("401 Unauthorized"))),
    }
}

// MARK: Utils
fn lookup_session_and_user(token: &str, db: &Connection) -> Option<(Session, User)> {
    let session = db
        .query::<Session>(
            formatcp!(
                "SELECT {} FROM sessions WHERE token = ? AND expires_at > ? LIMIT 1",
                Session::columns()
            ),
            (token.to_string(), Utc::now()),
        )
        .expect("Database error")
        .next()
        .map(|r| r.expect("Database error"))?;

    let user = db
        .query::<User>(
            formatcp!("SELECT {} FROM users WHERE id = ? LIMIT 1", User::columns()),
            session.user_id,
        )
        .expect("Database error")
        .next()
        .map(|r| r.expect("Database error"))?;

    // Sliding-window refresh: extend expiry when below threshold
    let refresh_threshold = Utc::now() + Duration::from_secs(SESSION_REFRESH_THRESHOLD_SECONDS);
    if session.expires_at.timestamp() < refresh_threshold.timestamp() {
        let new_expires_at = Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS);
        db.execute(
            "UPDATE sessions SET expires_at = ? WHERE token = ?",
            (new_expires_at, token.to_string()),
        )
        .expect("Database error");
    }

    Some((session, user))
}
