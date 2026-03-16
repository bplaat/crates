/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use bsqlite::query_args;
use const_format::formatcp;
use small_http::{Request, Response, Status};
use uuid::Uuid;

use crate::api;
use crate::context::Context;
use crate::controllers::users::get_user;
use crate::controllers::{not_found, parse_index_query, require_auth};
use crate::models::session::policies;
use crate::models::{IndexQuery, Session, UserRole};
use crate::utils::preprocess_fts_query;

// MARK: Handlers
pub(crate) fn sessions_index(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = require_auth!(ctx);
    let query = parse_index_query!(req);

    let (total, sessions) = if policies::can_index(auth_user) {
        list_all_sessions(ctx, &query)?
    } else {
        list_sessions(ctx, auth_user.id, false, &query)?
    };

    Ok(Response::with_json(api::SessionIndexResponse {
        pagination: api::Pagination {
            page: query.page,
            limit: query.limit,
            total,
        },
        data: sessions,
    }))
}

pub(crate) fn sessions_show(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = require_auth!(ctx);

    // Get session
    let session = match get_session(req, ctx)? {
        Some(session) => session,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_show(auth_user, &session) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    // Return session
    Ok(Response::with_json(api::Session::from(session)))
}

pub(crate) fn sessions_delete(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = require_auth!(ctx);

    // Get session
    let session = match get_session(req, ctx)? {
        Some(session) => session,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_delete(auth_user, &session) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    // Delete session
    ctx.database
        .execute("DELETE FROM sessions WHERE id = ?", session.id)?;

    // Success response
    Ok(Response::new())
}

pub(crate) fn users_sessions(req: &Request, ctx: &Context) -> Result<Response> {
    sessions_for_user(req, ctx, false)
}

pub(crate) fn users_sessions_active(req: &Request, ctx: &Context) -> Result<Response> {
    sessions_for_user(req, ctx, true)
}

pub(crate) fn sessions_active(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = require_auth!(ctx);
    let query = parse_index_query!(req);
    let (total, sessions) = list_sessions(ctx, auth_user.id, true, &query)?;
    Ok(Response::with_json(api::SessionIndexResponse {
        pagination: api::Pagination {
            page: query.page,
            limit: query.limit,
            total,
        },
        data: sessions,
    }))
}

// MARK: Utils
fn list_sessions(
    ctx: &Context,
    user_id: Uuid,
    active_only: bool,
    query: &IndexQuery,
) -> Result<(i64, Vec<api::Session>)> {
    let now = chrono::Utc::now();
    let offset = (query.page - 1) * query.limit;
    if query.query.is_empty() {
        if active_only {
            let total = ctx.database.query_some::<i64>(
                "SELECT COUNT(id) FROM sessions WHERE user_id = ? AND expires_at > ?",
                (user_id, now),
            )?;
            let sessions = query_args!(
                Session, ctx.database,
                formatcp!("SELECT {} FROM sessions WHERE user_id = :user_id AND expires_at > :now ORDER BY created_at DESC LIMIT :limit OFFSET :offset", Session::columns()),
                Args { user_id: user_id, now: now, limit: query.limit, offset: offset }
            )?.map(|r| r.map(Into::into)).collect::<Result<Vec<_>, _>>()?;
            Ok((total, sessions))
        } else {
            let total = ctx
                .database
                .query_some::<i64>("SELECT COUNT(id) FROM sessions WHERE user_id = ?", user_id)?;
            let sessions = query_args!(
                Session, ctx.database,
                formatcp!("SELECT {} FROM sessions WHERE user_id = :user_id ORDER BY created_at DESC LIMIT :limit OFFSET :offset", Session::columns()),
                Args { user_id: user_id, limit: query.limit, offset: offset }
            )?.map(|r| r.map(Into::into)).collect::<Result<Vec<_>, _>>()?;
            Ok((total, sessions))
        }
    } else {
        let fts_query = preprocess_fts_query(&query.query);
        if active_only {
            let total = ctx.database.query_some::<i64>(
                "SELECT COUNT(id) FROM sessions WHERE user_id = ? AND expires_at > ? AND id IN (SELECT id FROM sessions_fts WHERE sessions_fts MATCH ?)",
                (user_id, now, fts_query.clone()),
            )?;
            let sessions = query_args!(
                Session, ctx.database,
                formatcp!("SELECT {} FROM sessions WHERE user_id = :user_id AND expires_at > :now AND id IN (SELECT id FROM sessions_fts WHERE sessions_fts MATCH :fts_query) ORDER BY created_at DESC LIMIT :limit OFFSET :offset", Session::columns()),
                Args { user_id: user_id, now: now, fts_query: fts_query, limit: query.limit, offset: offset }
            )?.map(|r| r.map(Into::into)).collect::<Result<Vec<_>, _>>()?;
            Ok((total, sessions))
        } else {
            let total = ctx.database.query_some::<i64>(
                "SELECT COUNT(id) FROM sessions WHERE user_id = ? AND id IN (SELECT id FROM sessions_fts WHERE sessions_fts MATCH ?)",
                (user_id, fts_query.clone()),
            )?;
            let sessions = query_args!(
                Session, ctx.database,
                formatcp!("SELECT {} FROM sessions WHERE user_id = :user_id AND id IN (SELECT id FROM sessions_fts WHERE sessions_fts MATCH :fts_query) ORDER BY created_at DESC LIMIT :limit OFFSET :offset", Session::columns()),
                Args { user_id: user_id, fts_query: fts_query, limit: query.limit, offset: offset }
            )?.map(|r| r.map(Into::into)).collect::<Result<Vec<_>, _>>()?;
            Ok((total, sessions))
        }
    }
}

fn list_all_sessions(ctx: &Context, query: &IndexQuery) -> Result<(i64, Vec<api::Session>)> {
    let offset = (query.page - 1) * query.limit;
    if query.query.is_empty() {
        let total = ctx
            .database
            .query_some::<i64>("SELECT COUNT(id) FROM sessions", ())?;
        let sessions = query_args!(
            Session,
            ctx.database,
            formatcp!(
                "SELECT {} FROM sessions ORDER BY created_at DESC LIMIT :limit OFFSET :offset",
                Session::columns()
            ),
            Args {
                limit: query.limit,
                offset: offset
            }
        )?
        .map(|r| r.map(Into::into))
        .collect::<Result<Vec<_>, _>>()?;
        Ok((total, sessions))
    } else {
        let fts_query = preprocess_fts_query(&query.query);
        let total = ctx.database.query_some::<i64>(
            "SELECT COUNT(id) FROM sessions WHERE id IN (SELECT id FROM sessions_fts WHERE sessions_fts MATCH ?)",
            fts_query.clone(),
        )?;
        let sessions = query_args!(
            Session, ctx.database,
            formatcp!("SELECT {} FROM sessions WHERE id IN (SELECT id FROM sessions_fts WHERE sessions_fts MATCH :fts_query) ORDER BY created_at DESC LIMIT :limit OFFSET :offset", Session::columns()),
            Args { fts_query: fts_query, limit: query.limit, offset: offset }
        )?.map(|r| r.map(Into::into)).collect::<Result<Vec<_>, _>>()?;
        Ok((total, sessions))
    }
}

fn sessions_for_user(req: &Request, ctx: &Context, active_only: bool) -> Result<Response> {
    let auth_user = require_auth!(ctx);
    let query = parse_index_query!(req);

    // Get target user
    let user = match get_user(req, ctx)? {
        Some(user) => user,
        None => return not_found(req, ctx),
    };

    // Check authorization: admin can see any user's sessions; normal user only their own
    if auth_user.role != UserRole::Admin && auth_user.id != user.id {
        return Ok(Response::with_status(Status::Forbidden));
    }

    let (total, sessions) = list_sessions(ctx, user.id, active_only, &query)?;

    Ok(Response::with_json(api::SessionIndexResponse {
        pagination: api::Pagination {
            page: query.page,
            limit: query.limit,
            total,
        },
        data: sessions,
    }))
}

fn get_session(req: &Request, ctx: &Context) -> Result<Option<Session>> {
    let session_id = match req
        .params
        .get("session_id")
        .expect("Should be some")
        .parse::<Uuid>()
    {
        Ok(id) => id,
        Err(_) => return Ok(None),
    };

    Ok(ctx
        .database
        .query::<Session>(
            formatcp!(
                "SELECT {} FROM sessions WHERE id = ? LIMIT 1",
                Session::columns()
            ),
            session_id,
        )?
        .next()
        .transpose()?)
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
    fn test_sessions_index_search() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_user1, token1) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        let user2 = User {
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            email: "jane@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            ..Default::default()
        };
        ctx.database.insert_user(user2.clone()).unwrap();

        // Session with recognizable client_name and country
        ctx.database
            .insert_session(Session {
                user_id: user2.id,
                token: "token-jane".to_string(),
                ip_address: "1.2.3.4".to_string(),
                ip_country: Some("Netherlands".to_string()),
                client_name: Some("Firefox".to_string()),
                expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
                ..Default::default()
            })
            .unwrap();
        // Session with different client
        ctx.database
            .insert_session(Session {
                user_id: user2.id,
                token: "token-jane2".to_string(),
                ip_address: "5.6.7.8".to_string(),
                client_name: Some("Chrome".to_string()),
                expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
                ..Default::default()
            })
            .unwrap();

        // Search by client_name
        let res = router.handle(
            &Request::get("http://localhost/api/sessions?q=Firefox")
                .header("Authorization", format!("Bearer {token1}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::SessionIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].client.name, Some("Firefox".to_string()));

        // Search by ip_country
        let res = router.handle(
            &Request::get("http://localhost/api/sessions?q=Netherlands")
                .header("Authorization", format!("Bearer {token1}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::SessionIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].ip.country, Some("Netherlands".to_string()));
    }

    #[test]
    fn test_sessions_index_admin() {
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        ctx.database.insert_user(user2.clone()).unwrap();

        let session2 = Session {
            user_id: user2.id,
            token: "token-jane".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session2).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        ctx.database.insert_user(user2.clone()).unwrap();

        let session2 = Session {
            user_id: user2.id,
            token: "token-jane".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session2).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        ctx.database.insert_user(user.clone()).unwrap();

        let session = Session {
            user_id: user.id,
            token: "token-jane".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session.clone()).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        // Create another session for the same user
        let session = Session {
            user_id: user.id,
            token: "token-other".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session.clone()).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        ctx.database.insert_user(user2.clone()).unwrap();

        let session2 = Session {
            user_id: user2.id,
            token: "token-jane".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session2.clone()).unwrap();

        // User1 cannot view user2's session
        let res = router.handle(
            &Request::get(format!("http://localhost/api/sessions/{}", session2.id))
                .header("Authorization", format!("Bearer {token_user1}")),
        );
        assert_eq!(res.status, Status::Forbidden);
    }

    #[test]
    fn test_sessions_delete() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        // Create another session for the user
        let session = Session {
            user_id: user.id,
            token: "token-to-delete".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session.clone()).unwrap();

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
        .unwrap()
        .next()
        .map(|r| r.unwrap());
        assert!(deleted.is_none());
    }

    #[test]
    fn test_sessions_delete_forbidden() {
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        ctx.database.insert_user(user2.clone()).unwrap();

        let session2 = Session {
            user_id: user2.id,
            token: "token-jane".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session2.clone()).unwrap();

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
        .unwrap()
        .next()
        .map(|r| r.unwrap());
        assert!(existing.is_some());
    }

    #[test]
    fn test_users_sessions_admin() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token_admin) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create another user with two sessions
        let user = User {
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            email: "jane@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone()).unwrap();
        for i in 0..2 {
            ctx.database
                .insert_session(Session {
                    user_id: user.id,
                    token: format!("token-jane-{i}"),
                    expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
                    ..Default::default()
                })
                .unwrap();
        }

        // Admin can list any user's sessions
        let res = router.handle(
            &Request::get(format!("http://localhost/api/users/{}/sessions", user.id))
                .header("Authorization", format!("Bearer {token_admin}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::SessionIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 2);
        assert!(response.data.iter().all(|s| s.user_id == user.id));
    }

    #[test]
    fn test_users_sessions_own() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        // Add a second session for the same user
        ctx.database
            .insert_session(Session {
                user_id: user.id,
                token: "token-second".to_string(),
                expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
                ..Default::default()
            })
            .unwrap();

        // Normal user can list own sessions
        let res = router.handle(
            &Request::get(format!("http://localhost/api/users/{}/sessions", user.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::SessionIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 2);
    }

    #[test]
    fn test_users_sessions_forbidden() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token_user1) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);
        let (user2, _) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        // Normal user cannot list another user's sessions
        let res = router.handle(
            &Request::get(format!("http://localhost/api/users/{}/sessions", user2.id))
                .header("Authorization", format!("Bearer {token_user1}")),
        );
        assert_eq!(res.status, Status::Forbidden);
    }

    #[test]
    fn test_users_sessions_not_found() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        let res = router.handle(
            &Request::get(format!(
                "http://localhost/api/users/{}/sessions",
                Uuid::now_v7()
            ))
            .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::NotFound);
    }

    #[test]
    fn test_users_sessions_active_filters_expired() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        // Add an already-expired session
        ctx.database
            .insert_session(Session {
                user_id: user.id,
                token: "token-expired".to_string(),
                expires_at: Utc::now() - Duration::from_secs(3600),
                ..Default::default()
            })
            .unwrap();

        let res = router.handle(
            &Request::get(format!(
                "http://localhost/api/users/{}/sessions/active",
                user.id
            ))
            .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::SessionIndexResponse>(&res.body).unwrap();
        // Only the valid session from create_test_user_with_session_and_role is returned
        assert_eq!(response.data.len(), 1);
        assert!(response.data.iter().all(|s| s.user_id == user.id));
    }

    #[test]
    fn test_users_sessions_active_forbidden() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token_user1) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);
        let (user2, _) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        let res = router.handle(
            &Request::get(format!(
                "http://localhost/api/users/{}/sessions/active",
                user2.id
            ))
            .header("Authorization", format!("Bearer {token_user1}")),
        );
        assert_eq!(res.status, Status::Forbidden);
    }

    #[test]
    fn test_sessions_active_filters_expired() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        // Add an expired session for the same user
        ctx.database
            .insert_session(Session {
                user_id: user.id,
                token: "token-expired".to_string(),
                expires_at: Utc::now() - Duration::from_secs(3600),
                ..Default::default()
            })
            .unwrap();

        // Add a valid session for another user (must not appear)
        let (other_user, _) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);
        ctx.database
            .insert_session(Session {
                user_id: other_user.id,
                token: "token-other".to_string(),
                expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
                ..Default::default()
            })
            .unwrap();

        let res = router.handle(
            &Request::get("http://localhost/api/sessions/active")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::SessionIndexResponse>(&res.body).unwrap();
        // Only the one valid session belonging to this user
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].user_id, user.id);
    }

    #[test]
    fn test_sessions_active_own_only() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        // Valid session for a different user
        let (other, _) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);
        ctx.database
            .insert_session(Session {
                user_id: other.id,
                token: "token-other-valid".to_string(),
                expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
                ..Default::default()
            })
            .unwrap();

        let res = router.handle(
            &Request::get("http://localhost/api/sessions/active")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::SessionIndexResponse>(&res.body).unwrap();
        assert!(response.data.iter().all(|s| s.user_id == user.id));
    }
}
