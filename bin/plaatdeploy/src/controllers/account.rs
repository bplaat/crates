/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use chrono::Utc;
use small_http::{Request, Response, Status};
use zeroize::Zeroizing;

use crate::api;
use crate::context::Context;
use crate::controllers::{parse_body, require_auth};
use crate::models::UserTheme;

pub(crate) fn account_update(req: &Request, ctx: &Context) -> Result<Response> {
    let mut user = require_auth!(ctx).clone();
    let body = parse_body!(req, api::AccountUpdateBody, AccountUpdateBody);
    let first_name = body.first_name.trim();
    let last_name = body.last_name.trim();
    let email = body.email.trim().to_ascii_lowercase();
    let language = body.language.trim();
    if first_name.is_empty()
        || last_name.is_empty()
        || !validate::is_valid_email(&email)
        || language.len() != 2
        || !language.bytes().all(|byte| byte.is_ascii_lowercase())
    {
        return Ok(Response::with_status(Status::BadRequest));
    }
    if ctx.database.query_some::<i64>(
        "SELECT COUNT(id) FROM users WHERE email = ? AND id != ?",
        (email.clone(), user.id),
    )? > 0
    {
        return Ok(Response::with_status(Status::Conflict));
    }
    let theme = match body.theme {
        api::UserTheme::System => UserTheme::System,
        api::UserTheme::Light => UserTheme::Light,
        api::UserTheme::Dark => UserTheme::Dark,
    };
    user.first_name = first_name.to_string();
    user.last_name = last_name.to_string();
    user.email = email;
    user.theme = theme;
    user.language = language.to_string();
    user.updated_at = Utc::now();
    ctx.database.execute(
        "UPDATE users SET first_name = ?, last_name = ?, email = ?, theme = ?, language = ?, updated_at = ? WHERE id = ?",
        (
            user.first_name.clone(),
            user.last_name.clone(),
            user.email.clone(),
            user.theme as i64,
            user.language.clone(),
            user.updated_at,
            user.id,
        ),
    )?;
    Ok(Response::with_json(api::User::from(user)))
}

pub(crate) fn account_change_password(req: &Request, ctx: &Context) -> Result<Response> {
    let user = require_auth!(ctx);
    let body = parse_body!(req, api::ChangePasswordBody, ChangePasswordBody);
    if let Some(response) = super::auth::verify_password(&body.old_password, &user.password)? {
        return Ok(response);
    }
    let current_session_id = ctx
        .auth_session
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("auth context missing session"))?
        .id;
    ctx.database.transaction(|database| -> Result<()> {
        let now = Utc::now();
        database.execute(
            "UPDATE users SET password = ?, updated_at = ? WHERE id = ?",
            (
                crate::utils::password_hash(&body.new_password),
                now,
                user.id,
            ),
        )?;
        database.execute(
            "UPDATE sessions SET expires_at = ?, updated_at = ? WHERE user_id = ? AND id != ?",
            (now, now, user.id, current_session_id),
        )?;
        Ok(())
    })?;
    Ok(Response::with_status(Status::NoContent))
}

pub(crate) fn account_github_update(req: &Request, ctx: &Context) -> Result<Response> {
    let user = require_auth!(ctx);
    let body = parse_body!(req, api::GitHubTokenBody, GitHubTokenBody);
    let login = match crate::github::account_login(&body.token) {
        Ok(login) => login,
        Err(error) => {
            log::warn!("GitHub token validation failed: {error}");
            return Ok(Response::with_status(Status::BadRequest));
        }
    };
    ctx.database.execute(
        "UPDATE users SET github_token = ?, github_login = ?, updated_at = ? WHERE id = ?",
        (body.token, login.clone(), Utc::now(), user.id),
    )?;
    Ok(Response::with_json(api::GitHubConnection { login }))
}

pub(crate) fn account_github_delete(_: &Request, ctx: &Context) -> Result<Response> {
    let user = require_auth!(ctx);
    ctx.database.execute(
        "UPDATE users SET github_token = NULL, github_login = NULL, updated_at = ? WHERE id = ?",
        (Utc::now(), user.id),
    )?;
    Ok(Response::with_status(Status::NoContent))
}

#[derive(validate::Validate, from_derive::FromStruct)]
#[from_struct(api::GitHubTokenBody, only_from)]
struct GitHubTokenBody {
    #[validate(length(min = 1, max = 255))]
    token: String,
}

#[derive(validate::Validate, from_derive::FromStruct)]
#[from_struct(api::AccountUpdateBody, only_from)]
struct AccountUpdateBody {
    #[validate(length(min = 1, max = 100))]
    first_name: String,
    #[validate(length(min = 1, max = 100))]
    last_name: String,
    #[validate(length(min = 3, max = 254))]
    email: String,
    theme: api::UserTheme,
    #[validate(length(min = 2, max = 2))]
    language: String,
}

#[derive(validate::Validate, from_derive::FromStruct)]
#[from_struct(api::ChangePasswordBody, only_from)]
struct ChangePasswordBody {
    #[validate(length(min = 1, max = 128))]
    old_password: Zeroizing<String>,
    #[validate(length(min = 8, max = 128))]
    new_password: Zeroizing<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router;
    use crate::test_utils::authenticated_user;

    #[test]
    fn updates_account_profile() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (_, token) = authenticated_user(&ctx);

        let response = router.handle(
            &Request::put("http://localhost/api/account")
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(
                    r#"{"firstName":" Updated ","lastName":" User ","email":"updated@example.com","theme":"dark","language":"nl"}"#,
                ),
        );

        assert_eq!(response.status, Status::Ok);
        let user = serde_json::from_slice::<api::User>(&response.body).expect("account body");
        assert_eq!(user.first_name, "Updated");
        assert_eq!(user.last_name, "User");
        assert_eq!(user.email, "updated@example.com");
        assert!(matches!(user.theme, api::UserTheme::Dark));
        assert_eq!(user.language, "nl");
    }

    #[test]
    fn rejects_invalid_account_profile() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (_, token) = authenticated_user(&ctx);

        let response = router.handle(
            &Request::put("http://localhost/api/account")
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(
                    r#"{"firstName":"","lastName":"User","email":"invalid","theme":"system","language":"english"}"#,
                ),
        );

        assert_eq!(response.status, Status::BadRequest);
    }

    #[test]
    fn rejects_an_email_used_by_another_account_case_insensitively() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (_, token) = authenticated_user(&ctx);
        let other = crate::test_utils::insert_test_user(&ctx);

        let response = router.handle(
            &Request::put("http://localhost/api/account")
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(format!(
                    r#"{{"firstName":"Test","lastName":"User","email":"{}","theme":"system","language":"en"}}"#,
                    other.email.to_ascii_uppercase()
                )),
        );

        assert_eq!(response.status, Status::Conflict);
    }

    #[test]
    fn changes_password_with_valid_old_password() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (user, token) = authenticated_user(&ctx);
        let other_session = crate::test_utils::insert_test_session(&ctx, user.id, "other-token");

        let response = router.handle(
            &Request::post("http://localhost/api/account/password")
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(r#"{"oldPassword":"password123","newPassword":"new-password-123"}"#),
        );
        let password = ctx
            .database
            .query_some::<String>("SELECT password FROM users WHERE id = ?", user.id)
            .expect("query password");

        assert_eq!(response.status, Status::NoContent);
        assert!(pbkdf2::password_verify("new-password-123", &password).expect("verify password"));
        let other_expires_at = ctx
            .database
            .query_some::<chrono::DateTime<Utc>>(
                "SELECT expires_at FROM sessions WHERE id = ?",
                other_session.id,
            )
            .expect("query other session");
        assert!(other_expires_at.timestamp() <= Utc::now().timestamp());
        assert_eq!(
            router
                .handle(
                    &Request::get("http://localhost/api/auth/validate")
                        .header("Authorization", format!("Bearer {token}")),
                )
                .status,
            Status::Ok
        );
    }

    #[test]
    fn rejects_invalid_password_changes() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (_, token) = authenticated_user(&ctx);
        let request = |body| {
            Request::post("http://localhost/api/account/password")
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(body)
        };

        let wrong_old = router.handle(&request(
            r#"{"oldPassword":"wrong","newPassword":"new-password-123"}"#,
        ));
        let short = router.handle(&request(
            r#"{"oldPassword":"password123","newPassword":"short"}"#,
        ));

        assert_eq!(wrong_old.status, Status::Unauthorized);
        assert_eq!(short.status, Status::BadRequest);
    }

    #[test]
    fn disconnects_github_account() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (user, token) = authenticated_user(&ctx);
        ctx.database
            .execute(
                "UPDATE users SET github_token = ?, github_login = ? WHERE id = ?",
                ("github-token".to_string(), "octocat".to_string(), user.id),
            )
            .expect("connect GitHub account");

        let response = router.handle(
            &Request::delete("http://localhost/api/account/github")
                .header("Authorization", format!("Bearer {token}")),
        );
        let connected = ctx
            .database
            .query_some::<i64>(
                "SELECT COUNT(id) FROM users WHERE id = ? AND github_token IS NOT NULL",
                user.id,
            )
            .expect("query GitHub connection");

        assert_eq!(response.status, Status::NoContent);
        assert_eq!(connected, 0);
    }
}
