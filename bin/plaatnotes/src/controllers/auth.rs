/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::LazyLock;
use std::time::Duration;

use base64::prelude::*;
use bsqlite::execute_args;
use chrono::Utc;
use const_format::formatcp;
use serde::Deserialize;
use simple_useragent::UserAgentParser;
use small_http::{Request, Response, Status};
use validate::Validate;

use crate::api;
use crate::consts::{SESSION_EXPIRY_SECONDS, SESSION_TOKEN_LENGTH};
use crate::context::{Context, DatabaseHelpers};
use crate::models::{Session, User};

static USER_AGENT_PARSER: LazyLock<UserAgentParser> = LazyLock::new(UserAgentParser::new);

#[derive(Deserialize)]
struct IpInfo {
    city: String,
    country: String,
    loc: String,
}

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

    // Get IP information
    let ip_address = req.client_addr.ip().to_string();
    let (ip_latitude, ip_longitude, ip_country, ip_city) = {
        match Request::get(format!("https://ipinfo.io/{ip_address}/json")).fetch() {
            Ok(res) => {
                if let Ok(ip_info) = serde_json::from_slice::<IpInfo>(&res.body) {
                    let (lat, lon) = if let Some((lat_str, lon_str)) = ip_info.loc.split_once(',') {
                        (lat_str.parse::<f64>().ok(), lon_str.parse::<f64>().ok())
                    } else {
                        (None, None)
                    };
                    (lat, lon, Some(ip_info.country), Some(ip_info.city))
                } else {
                    (None, None, None, None)
                }
            }
            Err(_) => (None, None, None, None),
        }
    };

    // Parse User-Agent header
    let (client_name, client_version, client_os) = req
        .headers
        .get("User-Agent")
        .map(|ua_str| {
            let ua = USER_AGENT_PARSER.parse(ua_str);
            (
                Some(ua.client.family),
                ua.client.version,
                Some(ua.os.family),
            )
        })
        .unwrap_or((None, None, None));

    // Create session
    let session = Session {
        user_id: user.id,
        token: token.clone(),
        ip_address,
        ip_latitude,
        ip_longitude,
        ip_country,
        ip_city,
        client_name,
        client_version,
        client_os,
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

pub(crate) fn auth_validate(_req: &Request, ctx: &Context) -> Response {
    Response::with_json(api::AuthValidateResponse {
        user: ctx.auth_user.clone().expect("Should be authed").into(),
        session: ctx.auth_session.clone().expect("Should be authed").into(),
    })
}

pub(crate) fn auth_logout(_req: &Request, ctx: &Context) -> Response {
    let session = ctx.auth_session.clone().expect("Should be authed");

    // Expire the session by setting expires_at to now
    execute_args!(
        ctx.database,
        "UPDATE sessions SET expires_at = :now, updated_at = :now WHERE token = :token",
        Args {
            now: Utc::now(),
            token: session.token
        }
    );

    // Success response
    Response::new()
}

// MARK: Tests
#[cfg(test)]
mod test {
    use bsqlite::query_args;

    use super::*;
    use crate::consts::SESSION_EXPIRY_SECONDS;
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
            token: "valid-token-456".to_string(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session.clone());

        // Test with valid token
        let req = Request::get("http://localhost/api/auth/validate")
            .header("Authorization", "Bearer valid-token-456");
        let res = router.handle(&req);
        assert_eq!(res.status, Status::Ok);
    }

    #[test]
    fn test_get_authenticated_user_invalid_token() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Test with invalid token
        let req = Request::get("http://localhost/api/auth/validate")
            .header("Authorization", "Bearer invalid-token");
        let res = router.handle(&req);
        assert_eq!(res.status, Status::Unauthorized);
    }

    #[test]
    fn test_get_authenticated_user_expired_session() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

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
        let req = Request::get("http://localhost/api/auth/validate")
            .header("Authorization", "Bearer expired-token-789");
        let res = router.handle(&req);
        assert_eq!(res.status, Status::Unauthorized);
    }
}
