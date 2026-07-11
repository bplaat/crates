/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::LazyLock;
use std::time::Duration;

use anyhow::Result;
use base64::prelude::*;
use bsqlite::execute_args;
use chrono::Utc;
use const_format::formatcp;
use from_derive::FromStruct;
use simple_useragent::UserAgentParser;
use small_http::{Request, Response, Status};
use validate::Validate;

use crate::api;
use crate::consts::{SESSION_EXPIRY_SECONDS, SESSION_TOKEN_LENGTH};
use crate::context::{Context, DatabaseHelpers};
use crate::controllers::parse_body;
use crate::models::{Session, User};

static USER_AGENT_PARSER: LazyLock<UserAgentParser> = LazyLock::new(UserAgentParser::new);

/// A precomputed hash used to run a password verification even when the account does not
/// exist, so response timing does not reveal whether an email is registered.
static DUMMY_PASSWORD_HASH: LazyLock<String> =
    LazyLock::new(|| crate::utils::password_hash("dummy-password-for-constant-time-login"));

#[derive(Validate, FromStruct)]
#[from_struct(api::LoginBody)]
struct LoginBody {
    #[validate(email)]
    email: String,
    #[validate(ascii, length(min = 1, max = 128))]
    password: String,
}

pub(crate) fn auth_login(req: &Request, ctx: &Context) -> Result<Response> {
    let body = parse_body!(req, api::LoginBody, LoginBody);
    // Rate limit by IP
    let ip_address = req.ip().to_string();
    {
        let mut attempts = ctx.login_attempts.lock().unwrap_or_else(|p| p.into_inner());
        let now = std::time::Instant::now();
        let window = Duration::from_secs(crate::consts::LOGIN_RATE_LIMIT_WINDOW_SECONDS);
        attempts.retain(|_, (_, ws)| now.duration_since(*ws) < window);
        if let Some((count, window_start)) = attempts.get_mut(&ip_address) {
            if now.duration_since(*window_start) < window {
                if *count >= crate::consts::LOGIN_RATE_LIMIT_MAX_ATTEMPTS {
                    return Ok(Response::new().status(Status::TooManyRequests));
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
        Some(u) => u,
        None => {
            // Run a dummy verification to equalize response timing with the found-user path.
            let _ = pbkdf2::password_verify(&body.password, &DUMMY_PASSWORD_HASH);
            return Ok(Response::new().status(Status::Unauthorized));
        }
    };

    match pbkdf2::password_verify(&body.password, &user.password) {
        Ok(true) => {}
        Ok(false) => return Ok(Response::new().status(Status::Unauthorized)),
        Err(_) => return Ok(Response::new().status(Status::InternalServerError)),
    }

    // Generate token
    let token = {
        let mut bytes = [0u8; SESSION_TOKEN_LENGTH];
        getrandom::fill(&mut bytes)?;
        BASE64_URL_SAFE_NO_PAD.encode(bytes)
    };

    // Get IP info from the local DB-IP city lite database when available.
    let (ip_latitude, ip_longitude, ip_country, ip_city) =
        if let Some(reader) = ctx.maxminddb_reader.get() {
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
            (None, None, None, None)
        };

    // Parse user agent
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

    // Clear rate limit on success
    ctx.login_attempts
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .remove(&ip_address);

    Ok(Response::with_json(api::LoginResponse {
        user_id: user.id,
        token,
    }))
}

pub(crate) fn auth_validate(_req: &Request, ctx: &Context) -> Result<Response> {
    let user = ctx.auth_user.clone().expect("auth context missing user");
    let session = ctx
        .auth_session
        .clone()
        .expect("auth context missing session");
    Ok(Response::with_json(api::AuthValidateResponse {
        user: user.into(),
        session: session.into(),
    }))
}

pub(crate) fn auth_logout(_req: &Request, ctx: &Context) -> Result<Response> {
    let token = ctx
        .auth_session
        .as_ref()
        .expect("auth context missing session")
        .token
        .clone();

    execute_args!(
        ctx.database,
        "UPDATE sessions SET expires_at = :now, updated_at = :now WHERE token = :token",
        Args {
            now: Utc::now(),
            token: token
        }
    )?;

    Ok(Response::new())
}
