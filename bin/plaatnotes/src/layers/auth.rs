/*
 * Copyright (c) 2024-2025 PlaatSoft
 *
 * SPDX-License-Identifier: MIT
 */

use chrono::Utc;
use const_format::formatcp;
use small_http::{Request, Response, Status};

use crate::Context;
use crate::models::{self, Session};

// MARK: Auth optional
pub(crate) fn auth_optional_pre_layer(req: &Request, ctx: &mut Context) -> Option<Response> {
    // Get token from Authorization header
    let authorization = req
        .headers
        .get("Authorization")
        .or(req.headers.get("authorization"))?;
    let token = authorization[7..].trim().to_string();

    // Get active session by token
    let session = ctx
        .database
        .query::<Session>(
            formatcp!(
                "SELECT {} FROM sessions WHERE token = ? AND expires_at > ? LIMIT 1",
                Session::columns()
            ),
            (token, Utc::now()),
        )
        .expect("Database error")
        .next()
        .map(|r| r.expect("Database error"));
    let session = session?;

    // Get user by session user_id
    ctx.auth_user = ctx
        .database
        .query::<models::User>(
            formatcp!(
                "SELECT {} FROM users WHERE id = ? LIMIT 1",
                models::User::columns()
            ),
            session.user_id,
        )
        .expect("Database error")
        .next()
        .map(|r| r.expect("Database error"));
    ctx.auth_session = Some(session);

    None
}

// MARK: Auth required
pub(crate) fn auth_required_pre_layer(req: &Request, ctx: &mut Context) -> Option<Response> {
    // Get token from Authorization header
    let authorization = match req
        .headers
        .get("Authorization")
        .or(req.headers.get("authorization"))
    {
        Some(authorization) => authorization,
        None => {
            return Some(
                Response::new()
                    .status(Status::Unauthorized)
                    .body("401 Unauthorized"),
            );
        }
    };
    let token = authorization[7..].trim().to_string();

    // Get active session by token
    let session = ctx
        .database
        .query::<Session>(
            formatcp!(
                "SELECT {} FROM sessions WHERE token = ? AND expires_at > ? LIMIT 1",
                Session::columns()
            ),
            (token, Utc::now()),
        )
        .expect("Database error")
        .next()
        .map(|r| r.expect("Database error"));
    let session = match session {
        Some(session) => session,
        None => {
            return Some(
                Response::new()
                    .status(Status::Unauthorized)
                    .body("401 Unauthorized"),
            );
        }
    };

    // Get user by session user_id
    ctx.auth_user = ctx
        .database
        .query::<models::User>(
            formatcp!(
                "SELECT {} FROM users WHERE id = ? LIMIT 1",
                models::User::columns()
            ),
            session.user_id,
        )
        .expect("Database error")
        .next()
        .map(|r| r.expect("Database error"));
    ctx.auth_session = Some(session);

    None
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;
    use crate::models::UserRole;
    use crate::router;
    use crate::test_utils::create_test_user_with_session_and_role;

    #[test]
    fn test_unauthed() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        let res = router.handle(&Request::with_url("http://localhost/api/auth/validate"));
        assert_eq!(res.status, Status::Unauthorized);
    }

    #[test]
    fn test_authed() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create a test user and session
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Add Authorization header to request
        let req = Request::with_url("http://localhost/api/auth/validate")
            .header("Authorization", format!("Bearer {token}"));
        let res = router.handle(&req);
        assert_eq!(res.status, Status::Ok);
    }
}
