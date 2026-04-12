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
use serde::Deserialize;
use simple_useragent::UserAgentParser;
use small_http::{Request, Response, Status};

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

pub(crate) fn auth_login(req: &Request, ctx: &Context) -> Result<Response> {
    #[derive(Deserialize)]
    struct LoginBody {
        email: String,
        password: String,
    }

    let body = match serde_urlencoded::from_bytes::<LoginBody>(req.body.as_deref().unwrap_or(&[])) {
        Ok(b) => b,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };
    if body.email.is_empty() || body.password.is_empty() {
        return Ok(Response::new().status(Status::BadRequest));
    }

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
        None => return Ok(Response::new().status(Status::Unauthorized)),
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

    // Get IP info
    let (ip_latitude, ip_longitude, ip_country, ip_city) =
        match Request::get(format!("https://ipinfo.io/{ip_address}/json")).fetch() {
            Ok(res) => {
                if let Ok(ip_info) = serde_json::from_slice::<IpInfo>(&res.body) {
                    let (lat, lon) = ip_info
                        .loc
                        .split_once(',')
                        .and_then(|(la, lo)| {
                            Some((la.parse::<f64>().ok()?, lo.parse::<f64>().ok()?))
                        })
                        .map(|(la, lo)| (Some(la), Some(lo)))
                        .unwrap_or((None, None));
                    (lat, lon, Some(ip_info.country), Some(ip_info.city))
                } else {
                    (None, None, None, None)
                }
            }
            Err(_) => (None, None, None, None),
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
