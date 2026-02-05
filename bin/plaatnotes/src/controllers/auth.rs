/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::time::Duration;

use base64::prelude::*;
use bsqlite::{execute_args, query_args};
use chrono::Utc;
use const_format::formatcp;
use small_http::{Request, Response, Status};
use validate::Validate;

use crate::api;
use crate::consts::{SESSION_EXPIRY_SECONDS, SESSION_TOKEN_LENGTH};
use crate::context::{Context, DatabaseHelpers};
use crate::models::{Session, User};

#[derive(Validate)]
struct LoginBody {
    #[validate(email)]
    email: String,
    #[validate(ascii, length(min = 8, max = 128))]
    password: String,
}

impl From<api::LoginBody> for LoginBody {
    fn from(body: api::LoginBody) -> Self {
        Self {
            email: body.email,
            password: body.password,
        }
    }
}

pub(crate) fn auth_login(req: &Request, ctx: &Context) -> Response {
    // Parse and validate body
    let body =
        match serde_urlencoded::from_bytes::<api::LoginBody>(req.body.as_deref().unwrap_or(&[])) {
            Ok(body) => Into::<LoginBody>::into(body),
            Err(_) => return Response::with_status(Status::BadRequest),
        };
    if let Err(report) = body.validate() {
        return Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report));
    }

    // Find user by email
    let user = match ctx
        .database
        .query::<User>(
            formatcp!(
                "SELECT {} FROM users WHERE email = ? LIMIT 1",
                User::columns()
            ),
            body.email,
        )
        .next()
    {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Verify password
    match pbkdf2::password_verify(&body.password, &user.password) {
        Ok(true) => {}
        Ok(false) => return Response::with_status(Status::Unauthorized),
        Err(_) => return Response::with_status(Status::InternalServerError),
    }

    // Generate secure random token
    let token = {
        let mut bytes = [0u8; SESSION_TOKEN_LENGTH];
        getrandom::fill(&mut bytes).expect("Failed to generate random token");
        BASE64_URL_SAFE_NO_PAD.encode(bytes)
    };

    // Create session
    let session = Session {
        user_id: user.id,
        token: token.clone(),
        expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
        ..Default::default()
    };
    ctx.database.insert_session(session);

    // Return session token
    Response::with_json(api::LoginResponse {
        user_id: user.id,
        token,
    })
}

pub(crate) fn auth_logout(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    if get_auth_user(req, ctx).is_none() {
        return Response::with_status(Status::Unauthorized);
    }
    let token = get_auth_token(req).expect("Should be some");

    // Expire the session by setting expires_at to now
    execute_args!(
        ctx.database,
        "UPDATE sessions SET expires_at = :now, updated_at = :now WHERE token = :token",
        Args {
            now: Utc::now(),
            token: token
        }
    );

    // Success response
    Response::new()
}

// MARK: Utils
pub(crate) fn get_auth_token(req: &Request) -> Option<String> {
    Some(
        req.headers
            .get("authorization")?
            .strip_prefix("Bearer ")?
            .to_string(),
    )
}

pub(crate) fn get_auth_user(req: &Request, ctx: &Context) -> Option<User> {
    let token = get_auth_token(req)?;

    // Find valid session
    let session = query_args!(
        Session,
        ctx.database,
        formatcp!(
            "SELECT {} FROM sessions WHERE token = :token AND expires_at > :now LIMIT 1",
            Session::columns()
        ),
        Args {
            token: token.to_string(),
            now: Utc::now()
        }
    )
    .next()?;

    // Get user
    ctx.database
        .query::<User>(
            formatcp!("SELECT {} FROM users WHERE id = ? LIMIT 1", User::columns()),
            session.user_id,
        )
        .next()
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;
    use crate::router;

    #[test]
    fn test_auth_login() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create user
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        // Login with correct credentials
        let res = router.handle(
            &Request::post("http://localhost/api/auth/login")
                .body("email=john@example.com&password=password123"),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::LoginResponse>(&res.body).unwrap();
        assert!(!response.token.is_empty());
    }

    #[test]
    fn test_auth_login_incorrect_password() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create user
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        // Login with incorrect password
        let res = router.handle(
            &Request::post("http://localhost/api/auth/login")
                .body("email=john@example.com&password=wrongpassword"),
        );
        assert_eq!(res.status, Status::Unauthorized);
    }

    #[test]
    fn test_auth_login_non_existent_email() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Login with non-existent email
        let res = router.handle(
            &Request::post("http://localhost/api/auth/login")
                .body("email=notfound@example.com&password=password123"),
        );
        assert_eq!(res.status, Status::Unauthorized);
    }

    #[test]
    fn test_auth_logout() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create user and session
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        let session = Session {
            user_id: user.id,
            token: "test-token-123".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session.clone());

        // Logout with valid token
        let res = router.handle(
            &Request::post("http://localhost/api/auth/logout")
                .header("Authorization", "Bearer test-token-123"),
        );
        assert_eq!(res.status, Status::Ok);

        // Verify session is expired
        let expired_session = query_args!(
            Session,
            ctx.database,
            formatcp!(
                "SELECT {} FROM sessions WHERE token = :token LIMIT 1",
                Session::columns()
            ),
            Args {
                token: "test-token-123".to_string()
            }
        )
        .next()
        .unwrap();
        assert!(expired_session.expires_at.timestamp() <= Utc::now().timestamp());
    }

    #[test]
    fn test_get_authenticated_user_valid_token() {
        let ctx = Context::with_test_database();

        // Create user and session
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        let session = Session {
            user_id: user.id,
            token: "valid-token-456".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session.clone());

        // Test with valid token
        let req = Request::get("http://localhost/api/users")
            .header("Authorization", "Bearer valid-token-456");
        let authenticated_user = get_auth_user(&req, &ctx);
        assert!(authenticated_user.is_some());
        assert_eq!(authenticated_user.unwrap().email, "john@example.com");
    }

    #[test]
    fn test_get_authenticated_user_invalid_token() {
        let ctx = Context::with_test_database();

        // Test with invalid token
        let req = Request::get("http://localhost/api/users")
            .header("Authorization", "Bearer invalid-token");
        let authenticated_user = get_auth_user(&req, &ctx);
        assert!(authenticated_user.is_none());
    }

    #[test]
    fn test_get_authenticated_user_expired_session() {
        let ctx = Context::with_test_database();

        // Create user and expired session
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        let expired_session = Session {
            user_id: user.id,
            token: "expired-token-789".to_string(),
            expires_at: Utc::now() - Duration::from_secs(3600),
            ..Default::default()
        };
        ctx.database.insert_session(expired_session);

        // Test with expired session
        let req = Request::get("http://localhost/api/users")
            .header("Authorization", "Bearer expired-token-789");
        let authenticated_user = get_auth_user(&req, &ctx);
        assert!(authenticated_user.is_none());
    }
}
