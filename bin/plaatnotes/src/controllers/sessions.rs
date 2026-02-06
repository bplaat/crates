/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::query_args;
use const_format::formatcp;
use small_http::{Request, Response, Status};
use uuid::Uuid;
use validate::Validate;

use crate::api;
use crate::context::Context;
use crate::controllers::not_found;
use crate::models::session::policies;
use crate::models::{IndexQuery, Session};

pub(crate) fn sessions_index(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let auth_user = match &ctx.auth_user {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Parse request query
    let query = match req.url.query() {
        Some(query) => match serde_urlencoded::from_str::<IndexQuery>(query) {
            Ok(query) => query,
            Err(_) => return Response::with_status(Status::BadRequest),
        },
        None => IndexQuery::default(),
    };
    if let Err(report) = query.validate() {
        return Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report));
    }

    // Get sessions for authenticated user or all sessions if admin
    let search_query = format!("%{}%", query.query.replace("%", "\\%"));
    let (total, sessions) = match policies::can_index(auth_user) {
        true => {
            // Admin sees all sessions
            let total = query_args!(
                i64,
                ctx.database,
                "SELECT COUNT(id) FROM sessions WHERE token LIKE :search_query",
                Args {
                    search_query: search_query.clone()
                }
            )
            .next()
            .unwrap_or(0);
            let sessions = query_args!(
                Session,
                ctx.database,
                formatcp!(
                    "SELECT {} FROM sessions WHERE token LIKE :search_query ORDER BY created_at DESC LIMIT :limit OFFSET :offset",
                    Session::columns()
                ),
                Args {
                    search_query: search_query,
                    limit: query.limit,
                    offset: (query.page - 1) * query.limit
                }
            )
            .map(Into::<api::Session>::into)
            .collect::<Vec<_>>();
            (total, sessions)
        }
        false => {
            // Normal user sees only their own sessions
            let total = query_args!(
                i64,
                ctx.database,
                "SELECT COUNT(id) FROM sessions WHERE user_id = :user_id AND token LIKE :search_query",
                Args {
                    user_id: auth_user.id,
                    search_query: search_query.clone()
                }
            )
            .next()
            .unwrap_or(0);
            let sessions = query_args!(
                Session,
                ctx.database,
                formatcp!(
                    "SELECT {} FROM sessions WHERE user_id = :user_id AND token LIKE :search_query ORDER BY created_at DESC LIMIT :limit OFFSET :offset",
                    Session::columns()
                ),
                Args {
                    user_id: auth_user.id,
                    search_query: search_query,
                    limit: query.limit,
                    offset: (query.page - 1) * query.limit
                }
            )
            .map(Into::<api::Session>::into)
            .collect::<Vec<_>>();
            (total, sessions)
        }
    };

    // Return sessions
    Response::with_json(api::SessionIndexResponse {
        pagination: api::Pagination {
            page: query.page,
            limit: query.limit,
            total,
        },
        data: sessions,
    })
}

pub(crate) fn sessions_show(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let auth_user = match &ctx.auth_user {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Get session
    let session = match get_session(req, ctx) {
        Some(session) => session,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_show(auth_user, &session) {
        return Response::with_status(Status::Forbidden);
    }

    // Return session
    Response::with_json(Into::<api::Session>::into(session))
}

pub(crate) fn sessions_delete(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let auth_user = match &ctx.auth_user {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Get session
    let session = match get_session(req, ctx) {
        Some(session) => session,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_delete(auth_user, &session) {
        return Response::with_status(Status::Forbidden);
    }

    // Delete session
    ctx.database
        .execute("DELETE FROM sessions WHERE id = ?", session.id);

    // Success response
    Response::new()
}

// MARK: Utils
fn get_session(req: &Request, ctx: &Context) -> Option<Session> {
    let session_id = match req
        .params
        .get("session_id")
        .expect("session_id param should be present")
        .parse::<Uuid>()
    {
        Ok(id) => id,
        Err(_) => return None,
    };

    ctx.database
        .query::<Session>(
            formatcp!(
                "SELECT {} FROM sessions WHERE id = ? LIMIT 1",
                Session::columns()
            ),
            session_id,
        )
        .next()
}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::time::Duration;

    use chrono::Utc;

    use super::*;
    use crate::consts::SESSION_EXPIRY_SECONDS;
    use crate::context::DatabaseHelpers;
    use crate::models::{User, UserRole};
    use crate::router;
    use crate::test_utils::create_test_user_with_session_and_role;

    #[test]
    fn test_sessions_index_admin() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_user1, token1) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create another user with session
        let user2 = User {
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            email: "jane@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            ..Default::default()
        };
        ctx.database.insert_user(user2.clone());

        let session2 = Session {
            user_id: user2.id,
            token: "token-jane".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session2);

        // Admin can see all sessions
        let res = router.handle(
            &Request::get("http://localhost/api/sessions")
                .header("Authorization", format!("Bearer {token1}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::SessionIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 2);
    }

    #[test]
    fn test_sessions_index_normal_user() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (user1, token1) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        // Create another user with session
        let user2 = User {
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            email: "jane@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            role: UserRole::Normal,
            ..Default::default()
        };
        ctx.database.insert_user(user2.clone());

        let session2 = Session {
            user_id: user2.id,
            token: "token-jane".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session2);

        // Normal user only sees their own sessions
        let res = router.handle(
            &Request::get("http://localhost/api/sessions")
                .header("Authorization", format!("Bearer {token1}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::SessionIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].user_id, user1.id);
    }

    #[test]
    fn test_sessions_show_admin() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token_admin) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create another user with session
        let user = User {
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            email: "jane@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        let session = Session {
            user_id: user.id,
            token: "token-jane".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session.clone());

        // Admin can view any session
        let res = router.handle(
            &Request::get(format!("http://localhost/api/sessions/{}", session.id))
                .header("Authorization", format!("Bearer {token_admin}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::Session>(&res.body).unwrap();
        assert_eq!(response.id, session.id);
    }

    #[test]
    fn test_sessions_show_own_session() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        // Create another session for the same user
        let session = Session {
            user_id: user.id,
            token: "token-other".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session.clone());

        // User can view their own session
        let res = router.handle(
            &Request::get(format!("http://localhost/api/sessions/{}", session.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::Session>(&res.body).unwrap();
        assert_eq!(response.id, session.id);
    }

    #[test]
    fn test_sessions_show_forbidden() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token_user1) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        // Create another user with session
        let user2 = User {
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            email: "jane@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            role: UserRole::Normal,
            ..Default::default()
        };
        ctx.database.insert_user(user2.clone());

        let session2 = Session {
            user_id: user2.id,
            token: "token-jane".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session2.clone());

        // User1 cannot view user2's session
        let res = router.handle(
            &Request::get(format!("http://localhost/api/sessions/{}", session2.id))
                .header("Authorization", format!("Bearer {token_user1}")),
        );
        assert_eq!(res.status, Status::Forbidden);
    }

    #[test]
    fn test_sessions_delete() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        // Create another session for the user
        let session = Session {
            user_id: user.id,
            token: "token-to-delete".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session.clone());

        // Delete the session
        let res = router.handle(
            &Request::delete(format!("http://localhost/api/sessions/{}", session.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);

        // Verify session is deleted
        let deleted = query_args!(
            Session,
            ctx.database,
            formatcp!(
                "SELECT {} FROM sessions WHERE id = :id LIMIT 1",
                Session::columns()
            ),
            Args { id: session.id }
        )
        .next();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_sessions_delete_forbidden() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token_user1) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        // Create another user with session
        let user2 = User {
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            email: "jane@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            role: UserRole::Normal,
            ..Default::default()
        };
        ctx.database.insert_user(user2.clone());

        let session2 = Session {
            user_id: user2.id,
            token: "token-jane".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session2.clone());

        // User1 cannot delete user2's session
        let res = router.handle(
            &Request::delete(format!("http://localhost/api/sessions/{}", session2.id))
                .header("Authorization", format!("Bearer {token_user1}")),
        );
        assert_eq!(res.status, Status::Forbidden);

        // Verify session still exists
        let existing = query_args!(
            Session,
            ctx.database,
            formatcp!(
                "SELECT {} FROM sessions WHERE id = :id LIMIT 1",
                Session::columns()
            ),
            Args { id: session2.id }
        )
        .next();
        assert!(existing.is_some());
    }
}
