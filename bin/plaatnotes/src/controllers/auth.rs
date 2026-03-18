/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::LazyLock;
use std::time::{Duration, Instant};

use anyhow::Result;
use base64::prelude::*;
use bsqlite::execute_args;
use chrono::Utc;
use const_format::formatcp;
use from_derive::FromStruct;
use serde::Deserialize;
use simple_useragent::UserAgentParser;
use small_http::{Request, Response, Status};
use validate::Validate;

use crate::api;
use crate::consts::{
    LOGIN_RATE_LIMIT_MAX_ATTEMPTS, LOGIN_RATE_LIMIT_WINDOW_SECONDS, SESSION_EXPIRY_SECONDS,
    SESSION_TOKEN_LENGTH,
};
use crate::context::{Context, DatabaseHelpers};
use crate::models::{Session, User};

// MARK: Handlers
static USER_AGENT_PARSER: LazyLock<UserAgentParser> = LazyLock::new(UserAgentParser::new);

#[derive(Deserialize)]
struct IpInfo {
    city: String,
    country: String,
    loc: String,
}

#[derive(Validate, FromStruct)]
#[from_struct(api::LoginBody)]
struct LoginBody {
    #[validate(email)]
    email: String,
    #[validate(ascii, length(min = 8, max = 128))]
    password: String,
}

pub(crate) fn auth_login(req: &Request, ctx: &Context) -> Result<Response> {
    // Parse and validate body
    let body =
        match serde_urlencoded::from_bytes::<api::LoginBody>(req.body.as_deref().unwrap_or(&[])) {
            Ok(body) => LoginBody::from(body),
            Err(_) => return Ok(Response::with_status(Status::BadRequest)),
        };
    if let Err(report) = body.validate() {
        return Ok(Response::with_status(Status::BadRequest).json(api::Report::from(report)));
    }

    // Check login rate limit
    let ip_address = req.ip().to_string();
    {
        let mut attempts = ctx.login_attempts.lock().expect("Mutex poisoned");
        let now = Instant::now();
        let window = Duration::from_secs(LOGIN_RATE_LIMIT_WINDOW_SECONDS);
        if let Some((count, window_start)) = attempts.get_mut(&ip_address) {
            if now.duration_since(*window_start) < window {
                if *count >= LOGIN_RATE_LIMIT_MAX_ATTEMPTS {
                    return Ok(Response::with_status(Status::TooManyRequests));
                }
                *count += 1;
            } else {
                *count = 1;
                *window_start = now;
            }
        } else {
            attempts.insert(ip_address.clone(), (1, now));
        }
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
        )?
        .next()
        .transpose()?
    {
        Some(user) => user,
        None => return Ok(Response::with_status(Status::Unauthorized)),
    };

    // Verify password
    if let Some(err) = verify_password(&body.password, &user.password)? {
        return Ok(err);
    }

    // Generate secure random token
    let token = {
        let mut bytes = [0u8; SESSION_TOKEN_LENGTH];
        getrandom::fill(&mut bytes)?;
        BASE64_URL_SAFE_NO_PAD.encode(bytes)
    };

    // Get IP information
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
    let (client_name, client_version, client_os) =
        if let Some(ua_str) = req.headers.get("User-Agent") {
            let ua = USER_AGENT_PARSER.parse(ua_str);
            (
                Some(ua.client.family),
                ua.client.version,
                Some(ua.os.family),
            )
        } else {
            (None, None, None)
        };

    // Create session
    let session = Session {
        user_id: user.id,
        token: token.clone(),
        ip_address: ip_address.clone(),
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
    ctx.database.insert_session(session)?;

    // Clear rate limit counter on successful login
    ctx.login_attempts
        .lock()
        .expect("Mutex poisoned")
        .remove(&ip_address);

    // Return session token
    Ok(Response::with_json(api::LoginResponse {
        user_id: user.id,
        token,
    }))
}

pub(crate) fn auth_validate(_req: &Request, ctx: &Context) -> Result<Response> {
    Ok(Response::with_json(api::AuthValidateResponse {
        user: ctx.auth_user.clone().expect("Should be authed").into(),
        session: ctx.auth_session.clone().expect("Should be authed").into(),
    }))
}

pub(crate) fn auth_logout(_req: &Request, ctx: &Context) -> Result<Response> {
    let token = ctx
        .auth_session
        .as_ref()
        .expect("Should be authed")
        .token
        .clone();

    // Expire the session by setting expires_at to now
    execute_args!(
        ctx.database,
        "UPDATE sessions SET expires_at = :now, updated_at = :now WHERE token = :token",
        Args {
            now: Utc::now(),
            token: token
        }
    )?;

    // Success response
    Ok(Response::new())
}

// MARK: Utils
pub(crate) fn verify_password(plain: &str, hash: &str) -> Result<Option<Response>> {
    match pbkdf2::password_verify(plain, hash) {
        Ok(true) => Ok(None),
        Ok(false) => Ok(Some(Response::with_status(Status::Unauthorized))),
        Err(_) => Ok(Some(Response::with_status(Status::InternalServerError))),
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use bsqlite::query_args;

    use super::*;
    use crate::router;
    use crate::test_utils::{insert_test_session, insert_test_user};

    #[test]
    fn test_auth_login() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        insert_test_user(&ctx, "John", "Doe", "john@example.com");

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
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        insert_test_user(&ctx, "John", "Doe", "john@example.com");

        let res = router.handle(
            &Request::post("http://localhost/api/auth/login")
                .body("email=john@example.com&password=wrongpassword"),
        );
        assert_eq!(res.status, Status::Unauthorized);
    }

    #[test]
    fn test_auth_login_non_existent_email() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());

        let res = router.handle(
            &Request::post("http://localhost/api/auth/login")
                .body("email=notfound@example.com&password=password123"),
        );
        assert_eq!(res.status, Status::Unauthorized);
    }

    #[test]
    fn test_auth_logout() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let user = insert_test_user(&ctx, "John", "Doe", "john@example.com");
        insert_test_session(&ctx, user.id, "test-token-123");

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
        .unwrap()
        .next()
        .map(|r| r.unwrap())
        .unwrap();
        assert!(expired_session.expires_at.timestamp() <= Utc::now().timestamp());
    }

    #[test]
    fn test_get_authenticated_user_valid_token() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let user = insert_test_user(&ctx, "John", "Doe", "john@example.com");
        insert_test_session(&ctx, user.id, "valid-token-456");

        let res = router.handle(
            &Request::get("http://localhost/api/auth/validate")
                .header("Authorization", "Bearer valid-token-456"),
        );
        assert_eq!(res.status, Status::Ok);
    }

    #[test]
    fn test_get_authenticated_user_invalid_token() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());

        let res = router.handle(
            &Request::get("http://localhost/api/auth/validate")
                .header("Authorization", "Bearer invalid-token"),
        );
        assert_eq!(res.status, Status::Unauthorized);
    }

    #[test]
    fn test_get_authenticated_user_expired_session() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let user = insert_test_user(&ctx, "John", "Doe", "john@example.com");

        // Expired session - can't use insert_test_session since it sets a future expiry
        ctx.database
            .insert_session(Session {
                user_id: user.id,
                token: "expired-token-789".to_string(),
                expires_at: Utc::now() - Duration::from_secs(3600),
                ..Default::default()
            })
            .unwrap();

        let res = router.handle(
            &Request::get("http://localhost/api/auth/validate")
                .header("Authorization", "Bearer expired-token-789"),
        );
        assert_eq!(res.status, Status::Unauthorized);
    }

    #[test]
    fn test_auth_login_rate_limiting() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        insert_test_user(&ctx, "Jane", "Doe", "jane@example.com");

        // The first LOGIN_RATE_LIMIT_MAX_ATTEMPTS attempts should proceed (wrong password → 401)
        for _ in 0..LOGIN_RATE_LIMIT_MAX_ATTEMPTS {
            let res = router.handle(
                &Request::post("http://localhost/api/auth/login")
                    .body("email=jane@example.com&password=wrongpassword"),
            );
            assert_eq!(res.status, Status::Unauthorized);
        }

        // The next attempt must be rejected with 429 before even checking credentials
        let res = router.handle(
            &Request::post("http://localhost/api/auth/login")
                .body("email=jane@example.com&password=password123"),
        );
        assert_eq!(res.status, Status::TooManyRequests);
    }
}
