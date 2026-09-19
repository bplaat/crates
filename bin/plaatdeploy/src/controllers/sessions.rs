/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use chrono::Utc;
use small_http::{Request, Response, Status};
use uuid::Uuid;

use crate::api;
use crate::context::Context;
use crate::controllers::require_auth;
use crate::models::Session;

pub(crate) fn sessions_index(_: &Request, ctx: &Context) -> Result<Response> {
    let user = require_auth!(ctx);
    let sessions = ctx.database.query::<Session>(
        format!("SELECT {} FROM sessions WHERE user_id = ? AND expires_at > ? ORDER BY created_at DESC", Session::columns()),
        (user.id, Utc::now()),
    )?.map(|row| row.map(api::Session::from)).collect::<Result<Vec<_>, _>>()?;
    Ok(Response::with_json(sessions))
}

pub(crate) fn sessions_delete(req: &Request, ctx: &Context) -> Result<Response> {
    let user = require_auth!(ctx);
    let Some(id) = req
        .params
        .get("session_id")
        .and_then(|id| id.parse::<Uuid>().ok())
    else {
        return Ok(Response::with_status(Status::BadRequest));
    };
    ctx.database.execute(
        "UPDATE sessions SET expires_at = ?, updated_at = ? WHERE id = ? AND user_id = ?",
        (Utc::now(), Utc::now(), id, user.id),
    )?;
    Ok(Response::with_status(Status::NoContent))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router;
    use crate::test_utils::{authenticated_user, insert_test_session, insert_test_user};

    #[test]
    fn lists_only_the_current_users_active_sessions() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (user, token) = authenticated_user(&ctx);
        insert_test_session(&ctx, user.id, "second-token");
        let other = insert_test_user(&ctx);
        insert_test_session(&ctx, other.id, "other-token");

        let response = router.handle(
            &Request::get("http://localhost/api/sessions")
                .header("Authorization", format!("Bearer {token}")),
        );
        let sessions =
            serde_json::from_slice::<Vec<api::Session>>(&response.body).expect("sessions response");

        assert_eq!(response.status, Status::Ok);
        assert_eq!(sessions.len(), 2);
        assert!(sessions.iter().all(|session| session.user_id == user.id));
    }

    #[test]
    fn revokes_an_owned_session() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (user, token) = authenticated_user(&ctx);
        let session = insert_test_session(&ctx, user.id, "revoke-token");

        let response = router.handle(
            &Request::delete(format!("http://localhost/api/sessions/{}", session.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        let expires_at = ctx
            .database
            .query_some::<chrono::DateTime<Utc>>(
                "SELECT expires_at FROM sessions WHERE id = ?",
                session.id,
            )
            .expect("query session");

        assert_eq!(response.status, Status::NoContent);
        assert!(expires_at.timestamp() <= Utc::now().timestamp());
    }

    #[test]
    fn cannot_revoke_another_users_session() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (_, token) = authenticated_user(&ctx);
        let other = insert_test_user(&ctx);
        let session = insert_test_session(&ctx, other.id, "other-session");

        let response = router.handle(
            &Request::delete(format!("http://localhost/api/sessions/{}", session.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        let expires_at = ctx
            .database
            .query_some::<chrono::DateTime<Utc>>(
                "SELECT expires_at FROM sessions WHERE id = ?",
                session.id,
            )
            .expect("query session");

        assert_eq!(response.status, Status::NoContent);
        assert!(expires_at.timestamp() > Utc::now().timestamp());
    }

    #[test]
    fn rejects_an_invalid_session_id() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (_, token) = authenticated_user(&ctx);

        let response = router.handle(
            &Request::delete("http://localhost/api/sessions/not-a-uuid")
                .header("Authorization", format!("Bearer {token}")),
        );

        assert_eq!(response.status, Status::BadRequest);
    }
}
