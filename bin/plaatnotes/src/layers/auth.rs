/*
 * Copyright (c) 2024-2026 PlaatSoft
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::{Ok, Result};
use bsqlite::Connection;
use chrono::Utc;
use const_format::formatcp;
use small_http::{Request, Response, Status};

use crate::Context;
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
    let token = authorization[7..].trim();

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
        Some(authorization) => authorization,
        None => {
            return Some(Ok(Response::new()
                .status(Status::Unauthorized)
                .body("401 Unauthorized")));
        }
    };
    let token = authorization[7..].trim();

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

    Some((session, user))
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
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());

        let res = router.handle(&Request::with_url("http://localhost/api/auth/validate"));
        assert_eq!(res.status, Status::Unauthorized);
    }

    #[test]
    fn test_authed() {
        let ctx = Context::with_test_database().expect("Can't create test database");
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
