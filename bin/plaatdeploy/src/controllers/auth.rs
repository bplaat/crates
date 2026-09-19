/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::LazyLock;
use std::time::Duration;

use anyhow::Result;
use base64::prelude::*;
use bsql::execute_args;
use chrono::Utc;
use from_derive::FromStruct;
use serde::Deserialize;
use simple_useragent::UserAgentParser;
use small_http::{Request, Response, Status};
use uuid::Uuid;
use validate::Validate;
use zeroize::Zeroizing;

use crate::api;
use crate::consts::{SESSION_EXPIRY_SECONDS, SESSION_TOKEN_LENGTH};
use crate::context::{Context, DatabaseHelpers};
use crate::controllers::parse_body;
use crate::models::{Session, User};

// MARK: Handlers
static USER_AGENT_PARSER: LazyLock<UserAgentParser> = LazyLock::new(UserAgentParser::new);
static DUMMY_PASSWORD_HASH: LazyLock<String> =
    LazyLock::new(|| crate::utils::password_hash("invalid account password"));

#[derive(Deserialize)]
struct IpInfo {
    city: String,
    country: String,
    loc: String,
}

#[derive(Validate, FromStruct)]
#[from_struct(api::LoginBody, only_from)]
struct LoginBody {
    #[validate(email, length(max = 254))]
    email: String,
    #[validate(ascii, length(min = 8, max = 128))]
    password: Zeroizing<String>,
}

pub(crate) fn auth_login(req: &Request, ctx: &Context) -> Result<Response> {
    // Parse and validate body
    let body = parse_body!(req, api::LoginBody, LoginBody);

    // Check login rate limit
    let ip_address = req
        .ip_from_trusted_proxies(&ctx.trusted_proxies)
        .to_string();
    if !ctx.is_e2e {
        let mut attempts = ctx.login_attempts.lock().unwrap_or_else(|p| p.into_inner());
        let now = std::time::Instant::now();
        let window = Duration::from_secs(crate::consts::LOGIN_RATE_LIMIT_WINDOW_SECONDS);
        attempts.retain(|_, (_, window_start)| now.duration_since(*window_start) < window);

        if let Some((count, window_start)) = attempts.get_mut(&ip_address) {
            if now.duration_since(*window_start) < window {
                *count += 1;
                if *count > crate::consts::LOGIN_RATE_LIMIT_MAX_ATTEMPTS {
                    return Ok(Response::with_status(Status::TooManyRequests));
                }
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
            format!(
                "SELECT {} FROM users WHERE email = ? COLLATE NOCASE LIMIT 1",
                User::columns()
            ),
            body.email.to_ascii_lowercase(),
        )?
        .next()
        .transpose()?
    {
        Some(user) => user,
        None => {
            _ = verify_password(&body.password, &DUMMY_PASSWORD_HASH)?;
            return Ok(Response::with_status(Status::Unauthorized));
        }
    };

    // Verify password
    if let Some(err) = verify_password(&body.password, &user.password)? {
        return Ok(err);
    }

    // Create a new session for the user
    let token = create_session(req, ctx, user.id)?;

    // Clear rate limit counter on successful login
    if !ctx.is_e2e {
        ctx.login_attempts
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&ip_address);
    }

    // Return session token
    Ok(Response::with_json(api::LoginResponse {
        user_id: user.id,
        token,
    }))
}

pub(crate) fn auth_validate(_req: &Request, ctx: &Context) -> Result<Response> {
    let user = ctx
        .auth_user
        .clone()
        .ok_or_else(|| anyhow::anyhow!("auth context missing user"))?;
    let session = ctx
        .auth_session
        .clone()
        .ok_or_else(|| anyhow::anyhow!("auth context missing session"))?;
    Ok(Response::with_json(api::AuthValidateResponse {
        user: user.into(),
        session: session.into(),
    }))
}

pub(crate) fn auth_logout(_req: &Request, ctx: &Context) -> Result<Response> {
    let token = ctx
        .auth_session
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("auth context missing session"))?
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

    Ok(Response::with_status(Status::NoContent))
}

// MARK: Utils
/// Create a new session for the given user and return its token. The session is enriched with
/// IP geolocation and User-Agent information derived from the request.
pub(crate) fn create_session(req: &Request, ctx: &Context, user_id: Uuid) -> Result<String> {
    // Generate secure random token
    let token = {
        let mut bytes = [0u8; SESSION_TOKEN_LENGTH];
        getrandom::fill(&mut bytes)?;
        BASE64_URL_SAFE_NO_PAD.encode(bytes)
    };

    // Get IP information
    let ip_address = req
        .ip_from_trusted_proxies(&ctx.trusted_proxies)
        .to_string();
    let (ip_latitude, ip_longitude, ip_country, ip_city) = {
        if let Some(reader) = ctx.maxminddb_reader.get() {
            // Use local MaxMind DB
            match ip_address.parse::<std::net::IpAddr>() {
                Ok(ip) => match reader.lookup(ip) {
                    Ok(result) => match result.decode::<maxminddb::geoip2::City>() {
                        Ok(Some(city)) => (
                            city.location.latitude,
                            city.location.longitude,
                            city.country.iso_code,
                            city.city.names.english,
                        ),
                        _ => (None, None, None, None),
                    },
                    Err(_) => (None, None, None, None),
                },
                Err(_) => (None, None, None, None),
            }
        } else {
            // Fall back to ipinfo.io
            match Request::get(format!("https://ipinfo.io/{ip_address}/json")).fetch() {
                Ok(res) => {
                    if let Ok(ip_info) = serde_json::from_slice::<IpInfo>(&res.body) {
                        let (lat, lon) =
                            if let Some((lat_str, lon_str)) = ip_info.loc.split_once(',') {
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
        user_id,
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
    ctx.database.insert_session(session)?;

    Ok(token)
}

pub(crate) fn verify_password(plain: &str, hash: &str) -> Result<Option<Response>> {
    match pbkdf2::password_verify(plain, hash) {
        Ok(true) => Ok(None),
        Ok(false) => Ok(Some(Response::with_status(Status::Unauthorized))),
        Err(_) => Ok(Some(Response::with_status(Status::InternalServerError))),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;

    use super::*;
    use crate::context::DatabaseHelpers;
    use crate::models::Session;
    use crate::test_utils::{insert_test_session, insert_test_user};
    use crate::{api, router};

    #[test]
    fn logs_in_with_valid_credentials() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let user = insert_test_user(&ctx);

        let response = router.handle(
            &Request::post("http://localhost/api/auth/login")
                .header("Content-Type", "application/json")
                .body(format!(
                    r#"{{"email":"{}","password":"password123"}}"#,
                    user.email
                )),
        );

        assert_eq!(response.status, Status::Ok);
        let body =
            serde_json::from_slice::<api::LoginResponse>(&response.body).expect("login body");
        assert!(!body.token.is_empty());
    }

    #[test]
    fn rejects_invalid_login_credentials() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let user = insert_test_user(&ctx);

        let response = router.handle(
            &Request::post("http://localhost/api/auth/login")
                .header("Content-Type", "application/json")
                .body(format!(
                    r#"{{"email":"{}","password":"wrong-password"}}"#,
                    user.email
                )),
        );

        assert_eq!(response.status, Status::Unauthorized);
    }

    #[test]
    fn login_email_is_case_insensitive() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let user = insert_test_user(&ctx);

        let response = router.handle(
            &Request::post("http://localhost/api/auth/login")
                .header("Content-Type", "application/json")
                .body(format!(
                    r#"{{"email":"{}","password":"password123"}}"#,
                    user.email.to_ascii_uppercase()
                )),
        );

        assert_eq!(response.status, Status::Ok);
    }

    #[test]
    fn validates_only_active_sessions() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let user = insert_test_user(&ctx);
        insert_test_session(&ctx, user.id, "active-token");
        ctx.database
            .insert_session(Session {
                user_id: user.id,
                token: "expired-token".to_string(),
                expires_at: Utc::now() - Duration::from_secs(1),
                ..Default::default()
            })
            .expect("insert expired session");

        let active = router.handle(
            &Request::get("http://localhost/api/auth/validate")
                .header("Authorization", "Bearer active-token"),
        );
        let expired = router.handle(
            &Request::get("http://localhost/api/auth/validate")
                .header("Authorization", "Bearer expired-token"),
        );
        let missing = router.handle(&Request::get("http://localhost/api/auth/validate"));

        assert_eq!(active.status, Status::Ok);
        assert_eq!(expired.status, Status::Unauthorized);
        assert_eq!(missing.status, Status::Unauthorized);
    }

    #[test]
    fn returns_the_refreshed_session_expiry() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let user = insert_test_user(&ctx);
        ctx.database
            .insert_session(Session {
                user_id: user.id,
                token: "expiring-token".to_string(),
                expires_at: Utc::now() + Duration::from_secs(60),
                ..Default::default()
            })
            .expect("insert expiring session");

        let response = router.handle(
            &Request::get("http://localhost/api/auth/validate")
                .header("Authorization", "Bearer expiring-token"),
        );
        let body = serde_json::from_slice::<api::AuthValidateResponse>(&response.body)
            .expect("validate response");

        assert_eq!(response.status, Status::Ok);
        assert!(
            body.session.expires_at.timestamp()
                > (Utc::now() + Duration::from_secs(60 * 24 * 60 * 60)).timestamp()
        );
    }

    #[test]
    fn logout_expires_the_current_session() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let user = insert_test_user(&ctx);
        insert_test_session(&ctx, user.id, "logout-token");

        let response = router.handle(
            &Request::post("http://localhost/api/auth/logout")
                .header("Authorization", "Bearer logout-token"),
        );
        let validate = router.handle(
            &Request::get("http://localhost/api/auth/validate")
                .header("Authorization", "Bearer logout-token"),
        );

        assert_eq!(response.status, Status::NoContent);
        assert_eq!(validate.status, Status::Unauthorized);
    }

    #[test]
    fn reports_malformed_password_hashes_as_server_errors() {
        let response = verify_password("password", "not-a-password-hash")
            .expect("verify password")
            .expect("error response");

        assert_eq!(response.status, Status::InternalServerError);
    }
}
