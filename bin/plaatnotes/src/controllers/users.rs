/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use bsqlite::{execute_args, query_args};
use chrono::Utc;
use const_format::formatcp;
use from_derive::FromStruct;
use small_http::{Request, Response, Status};
use uuid::Uuid;
use validate::Validate;

use crate::api;
use crate::context::{Context, DatabaseHelpers};
use crate::controllers::auth::verify_password;
use crate::controllers::{not_found, parse_body, parse_body_ctx, parse_index_query, require_auth};
use crate::models::User;
use crate::models::note::{FILTER_ARCHIVED, FILTER_NORMAL, FILTER_PINNED, FILTER_TRASHED};
use crate::models::user::validators::{is_unique_email, is_unique_email_or_target_user_email};
use crate::models::user::{UserRole, UserTheme, policies};
use crate::utils::preprocess_fts_query;

// MARK: Handlers
pub(crate) fn users_index(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = require_auth!(ctx);

    // Check authorization
    if !policies::can_index(auth_user) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    let query = parse_index_query!(req);

    // Get users
    let (total, users) = if query.query.is_empty() {
        let total = ctx
            .database
            .query_some::<i64>("SELECT COUNT(id) FROM users", ())?;
        let users = query_args!(
            User,
            ctx.database,
            formatcp!(
                "SELECT {} FROM users ORDER BY created_at DESC LIMIT :limit OFFSET :offset",
                User::columns()
            ),
            Args {
                limit: query.limit,
                offset: (query.page - 1) * query.limit
            }
        )?
        .map(|r| r.map(Into::into))
        .collect::<Result<Vec<_>, _>>()?;
        (total, users)
    } else {
        let fts_query = preprocess_fts_query(&query.query);
        let total = ctx
            .database
            .query_some::<i64>(
                "SELECT COUNT(id) FROM users WHERE id IN (SELECT id FROM users_fts WHERE users_fts MATCH ?)",
                fts_query.clone(),
            )?;
        let users = query_args!(
            User,
            ctx.database,
            formatcp!(
                "SELECT {} FROM users WHERE id IN (SELECT id FROM users_fts WHERE users_fts MATCH :fts_query)
                ORDER BY created_at DESC LIMIT :limit OFFSET :offset",
                User::columns()
            ),
            Args {
                fts_query: fts_query,
                limit: query.limit,
                offset: (query.page - 1) * query.limit
            }
        )
        ?
        .map(|r| r.map(Into::into))
        .collect::<Result<Vec<_>, _>>()?;
        (total, users)
    };

    // Return users
    Ok(Response::with_json(api::UserIndexResponse {
        pagination: api::Pagination {
            page: query.page,
            limit: query.limit,
            total,
        },
        data: users,
    }))
}

#[derive(Validate, FromStruct)]
#[from_struct(api::UserCreateBody)]
#[validate(context(Context))]
struct UserCreateBody {
    #[validate(length(min = 1, max = 128))]
    first_name: String,
    #[validate(length(min = 1, max = 128))]
    last_name: String,
    #[validate(email, custom(is_unique_email))]
    email: String,
    #[validate(ascii, length(min = 8, max = 128))]
    password: String,
    role: UserRole,
}

pub(crate) fn users_create(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = require_auth!(ctx);

    // Check authorization
    if !policies::can_create(auth_user) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    let body = parse_body_ctx!(req, api::UserCreateBody, UserCreateBody, ctx);

    // Hash password
    let hashed_password = pbkdf2::password_hash(&body.password);

    // Create user
    let user = User {
        first_name: body.first_name,
        last_name: body.last_name,
        email: body.email,
        password: hashed_password,
        role: body.role,
        ..Default::default()
    };
    ctx.database.insert_user(user.clone())?;

    // Return created user
    Ok(Response::with_json(api::User::from(user)))
}

pub(crate) fn users_show(_req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = require_auth!(ctx);

    // Get user
    let user = match get_user(_req, ctx)? {
        Some(user) => user,
        None => return not_found(_req, ctx),
    };

    // Check authorization
    if !policies::can_show(auth_user, &user) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    // Return user
    Ok(Response::with_json(api::User::from(user)))
}

#[derive(Validate, FromStruct)]
#[from_struct(api::UserUpdateBody)]
#[validate(context(Context))]
struct UserUpdateBody {
    #[validate(length(min = 1, max = 128))]
    first_name: String,
    #[validate(length(min = 1, max = 128))]
    last_name: String,
    #[validate(email, custom(is_unique_email_or_target_user_email))]
    email: String,
    #[validate(ascii, length(min = 8, max = 128))]
    password: Option<String>,
    theme: UserTheme,
    language: String,
    role: UserRole,
}

pub(crate) fn users_update(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = require_auth!(ctx);

    // Get user
    let mut user = match get_user(req, ctx)? {
        Some(user) => user,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_update(auth_user, &user) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    let body = parse_body_ctx!(
        req,
        api::UserUpdateBody,
        UserUpdateBody,
        &Context {
            update_target_user_id: Some(user.id),
            ..ctx.clone()
        }
    );

    // Update user
    user.first_name = body.first_name;
    user.last_name = body.last_name;
    user.email = body.email;
    user.theme = body.theme;
    user.language = body.language;
    if auth_user.role == UserRole::Admin {
        user.role = body.role;
    }
    if let Some(password) = body.password
        && auth_user.role == UserRole::Admin
    {
        user.password = pbkdf2::password_hash(&password);
        execute_args!(
            ctx.database,
            "UPDATE users SET password = :password WHERE id = :id",
            Args {
                password: user.password.clone(),
                id: user.id
            }
        )?;
    }
    user.updated_at = Utc::now();
    execute_args!(
        ctx.database,
        "UPDATE users SET first_name = :first_name, last_name = :last_name, email = :email, theme = :theme, language = :language, role = :role, updated_at = :updated_at WHERE id = :id",
        Args {
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            email: user.email.clone(),
            theme: user.theme,
            language: user.language.clone(),
            role: user.role,
            updated_at: user.updated_at,
            id: user.id
        }
    )?;

    // Return updated user
    Ok(Response::with_json(api::User::from(user)))
}

#[derive(Validate, FromStruct)]
#[from_struct(api::UserChangePasswordBody)]
struct UserChangePasswordBody {
    #[validate(ascii, length(min = 8, max = 128))]
    old_password: String,
    #[validate(ascii, length(min = 8, max = 128))]
    new_password: String,
}

pub(crate) fn users_change_password(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = require_auth!(ctx);

    // Get user
    let mut user = match get_user(req, ctx)? {
        Some(user) => user,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_update(auth_user, &user) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    let body = parse_body!(req, api::UserChangePasswordBody, UserChangePasswordBody);

    // Verify old password
    if let Some(err) = verify_password(&body.old_password, &user.password)? {
        return Ok(err);
    }

    // Update password
    user.password = pbkdf2::password_hash(&body.new_password);
    user.updated_at = Utc::now();
    execute_args!(
        ctx.database,
        "UPDATE users SET password = :password, updated_at = :updated_at WHERE id = :id",
        Args {
            password: user.password.clone(),
            updated_at: user.updated_at,
            id: user.id
        }
    )?;

    // Success response
    Ok(Response::new())
}

pub(crate) fn users_delete(_req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = require_auth!(ctx);

    // Get user
    let user = match get_user(_req, ctx)? {
        Some(user) => user,
        None => return not_found(_req, ctx),
    };

    // Check authorization
    if !policies::can_delete(auth_user, &user) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    // Delete user
    ctx.database
        .execute("DELETE FROM users WHERE id = ?", user.id)?;

    // Success response
    Ok(Response::new())
}

pub(crate) fn users_notes(req: &Request, ctx: &Context) -> Result<Response> {
    users_notes_filtered(req, ctx, FILTER_NORMAL)
}

pub(crate) fn users_notes_pinned(req: &Request, ctx: &Context) -> Result<Response> {
    users_notes_filtered(req, ctx, FILTER_PINNED)
}

pub(crate) fn users_notes_archived(req: &Request, ctx: &Context) -> Result<Response> {
    users_notes_filtered(req, ctx, FILTER_ARCHIVED)
}

pub(crate) fn users_notes_trashed(req: &Request, ctx: &Context) -> Result<Response> {
    users_notes_filtered(req, ctx, FILTER_TRASHED)
}

// MARK: Utils
pub(crate) fn get_user(req: &Request, ctx: &Context) -> Result<Option<User>> {
    // Parse user id from url
    let user_id = match req
        .params
        .get("user_id")
        .expect("Should be some")
        .parse::<Uuid>()
    {
        Ok(id) => id,
        Err(_) => return Ok(None),
    };

    // Get user
    Ok(ctx
        .database
        .query::<User>(
            formatcp!("SELECT {} FROM users WHERE id = ? LIMIT 1", User::columns()),
            user_id,
        )?
        .next()
        .transpose()?)
}

fn users_notes_filtered(req: &Request, ctx: &Context, filter: &str) -> Result<Response> {
    let auth_user = require_auth!(ctx);

    let user = match get_user(req, ctx)? {
        Some(user) => user,
        None => return not_found(req, ctx),
    };

    if !policies::can_show(auth_user, &user) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    let query = parse_index_query!(req);
    let (total, notes) =
        crate::controllers::notes::fetch_notes_page(ctx, filter, Some(user.id), &query)?;

    Ok(Response::with_json(api::NoteIndexResponse {
        pagination: api::Pagination {
            page: query.page,
            limit: query.limit,
            total,
        },
        data: notes,
    }))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::{Note, UserRole};
    use crate::router;
    use crate::test_utils::{
        create_test_user_with_session, create_test_user_with_session_and_role, insert_test_note,
        insert_test_user,
    };

    #[test]
    fn test_users_index() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Fetch /users — should contain just the test user
        let res = router.handle(
            &Request::get("http://localhost/api/users")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let users = serde_json::from_slice::<api::UserIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].first_name, "Test");
        assert_eq!(users[0].email, user.email);

        // Create another user
        insert_test_user(&ctx, "John", "Doe", "john.doe@example.com");

        // Fetch /users — should now contain both users
        let res = router.handle(
            &Request::get("http://localhost/api/users")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let users = serde_json::from_slice::<api::UserIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(users.len(), 2);
    }

    #[test]
    fn test_users_index_search() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        insert_test_user(&ctx, "Alice", "Smith", "alice@example.com");
        insert_test_user(&ctx, "Bob", "Jones", "bob@example.com");

        let res = router.handle(
            &Request::get("http://localhost/api/users?q=Alice")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::UserIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].first_name, "Alice");
    }

    #[test]
    fn test_users_index_fts5_search() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        insert_test_user(&ctx, "Alice", "Smith", "alice.smith@example.com");
        insert_test_user(&ctx, "Alice", "Johnson", "alice.johnson@example.com");
        insert_test_user(&ctx, "Bob", "Smith", "bob.smith@example.com");
        insert_test_user(&ctx, "Carol", "White", "carol@example.com");

        // Prefix search
        let res = router.handle(
            &Request::get("http://localhost/api/users?q=Al*")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::UserIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 2);

        // AND search
        let res = router.handle(
            &Request::get("http://localhost/api/users?q=Alice AND Smith")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::UserIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].first_name, "Alice");
        assert_eq!(response.data[0].last_name, "Smith");

        // OR search
        let res = router.handle(
            &Request::get("http://localhost/api/users?q=Alice OR Bob")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::UserIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 3);

        // NOT search
        let res = router.handle(
            &Request::get("http://localhost/api/users?q=Alice NOT Smith")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::UserIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].last_name, "Johnson");

        // Phrase search
        let res = router.handle(
            &Request::get(r#"http://localhost/api/users?q="Alice Smith""#)
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::UserIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].first_name, "Alice");
        assert_eq!(response.data[0].last_name, "Smith");

        // Column-scoped search (email field only)
        let res = router.handle(
            &Request::get("http://localhost/api/users?q=email:carol")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::UserIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].first_name, "Carol");
    }

    #[test]
    fn test_users_index_pagination() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Test user already exists, create 29 more for 30 total
        for i in 1..=29 {
            insert_test_user(
                &ctx,
                &format!("User{i}"),
                "Test",
                &format!("user{i}@example.com"),
            );
        }

        let res = router.handle(
            &Request::get("http://localhost/api/users?limit=10&page=1")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::UserIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 10);
        assert_eq!(response.pagination.page, 1);
        assert_eq!(response.pagination.limit, 10);
        assert_eq!(response.pagination.total, 30);

        let res = router.handle(
            &Request::get("http://localhost/api/users?limit=5&page=2")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::UserIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 5);
        assert_eq!(response.pagination.page, 2);
        assert_eq!(response.pagination.limit, 5);
        assert_eq!(response.pagination.total, 30);
    }

    #[test]
    fn test_users_create() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        let res = router.handle(
            &Request::post("http://localhost/api/users")
                .header("Authorization", format!("Bearer {token}"))
                .body(
                    "firstName=Jane&lastName=Smith&email=jane@example.com&password=securepass123&role=normal",
                ),
        );
        assert_eq!(res.status, Status::Ok);
        let user = serde_json::from_slice::<api::User>(&res.body).unwrap();
        assert_eq!(user.first_name, "Jane");
        assert_eq!(user.last_name, "Smith");
        assert_eq!(user.email, "jane@example.com");
    }

    #[test]
    fn test_users_create_duplicate_email() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        let res = router.handle(
            &Request::post("http://localhost/api/users")
                .header("Authorization", format!("Bearer {token}"))
                .body(
                    "firstName=Jane&lastName=Smith&email=jane@example.com&password=securepass123&role=normal",
                ),
        );
        assert_eq!(res.status, Status::Ok);

        // Same email again — should fail
        let res = router.handle(
            &Request::post("http://localhost/api/users")
                .header("Authorization", format!("Bearer {token}"))
                .body(
                    "firstName=John&lastName=Doe&email=jane@example.com&password=securepass123&role=normal",
                ),
        );
        assert_eq!(res.status, Status::BadRequest);
        let report = serde_json::from_slice::<api::Report>(&res.body).unwrap();
        assert!(report.0.contains_key("email"));
    }

    #[test]
    fn test_users_show() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);
        let user = insert_test_user(&ctx, "John", "Doe", "john@example.com");

        let res = router.handle(
            &Request::get(format!("http://localhost/api/users/{}", user.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let user = serde_json::from_slice::<api::User>(&res.body).unwrap();
        assert_eq!(user.first_name, "John");
    }

    #[test]
    fn test_users_show_not_found() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        let res = router.handle(
            &Request::get(format!("http://localhost/api/users/{}", Uuid::now_v7()))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::NotFound);
    }

    #[test]
    fn test_users_update() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);
        let user = insert_test_user(&ctx, "John", "Doe", "john@example.com");

        let res = router.handle(
            &Request::put(format!("http://localhost/api/users/{}", user.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("firstName=John&lastName=Smith&email=john.smith@example.com&theme=system&language=en&role=normal"),
        );
        assert_eq!(res.status, Status::Ok);
        let updated_user = serde_json::from_slice::<api::User>(&res.body).unwrap();
        assert_eq!(updated_user.last_name, "Smith");
        assert_eq!(updated_user.email, "john.smith@example.com");
    }

    #[test]
    fn test_users_update_duplicate_email() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);
        let user1 = insert_test_user(&ctx, "John", "Doe", "john@example.com");
        let user2 = insert_test_user(&ctx, "Jane", "Smith", "jane@example.com");

        let res = router.handle(
            &Request::put(format!("http://localhost/api/users/{}", user2.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("firstName=Jane&lastName=Smith&email=john@example.com&theme=system&language=en&role=normal"),
        );
        assert_eq!(res.status, Status::BadRequest);
        let report = serde_json::from_slice::<api::Report>(&res.body).unwrap();
        assert!(report.0.contains_key("email"));
        // Silence unused variable warning
        let _ = user1;
    }

    #[test]
    fn test_users_update_same_email() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);
        let user = insert_test_user(&ctx, "John", "Doe", "john@example.com");

        // Updating with the same email should succeed
        let res = router.handle(
            &Request::put(format!("http://localhost/api/users/{}", user.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("firstName=John&lastName=Doe&email=john@example.com&theme=system&language=en&role=normal"),
        );
        assert_eq!(res.status, Status::Ok);
        let updated_user = serde_json::from_slice::<api::User>(&res.body).unwrap();
        assert_eq!(updated_user.email, "john@example.com");
    }

    #[test]
    fn test_users_update_validation_error() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);
        let user = insert_test_user(&ctx, "John", "Doe", "john@example.com");

        let res = router.handle(
            &Request::put(format!("http://localhost/api/users/{}", user.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("firstName=&lastName=Smith&email=invalid-email"),
        );
        assert_eq!(res.status, Status::BadRequest);
    }

    #[test]
    fn test_users_change_password() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);
        let user = insert_test_user(&ctx, "John", "Doe", "john@example.com");

        let res = router.handle(
            &Request::post(format!(
                "http://localhost/api/users/{}/change-password",
                user.id
            ))
            .header("Authorization", format!("Bearer {token}"))
            .body("oldPassword=password123&newPassword=newpassword456"),
        );
        assert_eq!(res.status, Status::Ok);

        // Verify new password works
        let stored_user = ctx
            .database
            .query::<User>(
                formatcp!("SELECT {} FROM users WHERE id = ? LIMIT 1", User::columns()),
                user.id,
            )
            .unwrap()
            .next()
            .map(|r| r.unwrap())
            .unwrap();
        assert!(pbkdf2::password_verify("newpassword456", &stored_user.password).unwrap());
    }

    #[test]
    fn test_users_change_password_incorrect_old_password() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);
        let user = insert_test_user(&ctx, "John", "Doe", "john@example.com");

        let res = router.handle(
            &Request::post(format!(
                "http://localhost/api/users/{}/change-password",
                user.id
            ))
            .header("Authorization", format!("Bearer {token}"))
            .body("oldPassword=wrongpassword&newPassword=anotherpassword"),
        );
        assert_eq!(res.status, Status::Unauthorized);
    }

    #[test]
    fn test_users_change_password_validation_error() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);
        let user = insert_test_user(&ctx, "John", "Doe", "john@example.com");

        let res = router.handle(
            &Request::post(format!(
                "http://localhost/api/users/{}/change-password",
                user.id
            ))
            .header("Authorization", format!("Bearer {token}"))
            .body("oldPassword=password123&newPassword=short"),
        );
        assert_eq!(res.status, Status::BadRequest);
    }

    #[test]
    fn test_users_delete() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);
        let user = insert_test_user(&ctx, "John", "Doe", "john@example.com");

        let res = router.handle(
            &Request::delete(format!("http://localhost/api/users/{}", user.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);

        // Only the original test admin user should remain
        let res = router.handle(
            &Request::get("http://localhost/api/users")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let users = serde_json::from_slice::<api::UserIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(users.len(), 1);
    }

    #[test]
    fn test_password_hashing() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        let res = router.handle(
            &Request::post("http://localhost/api/users")
                .header("Authorization", format!("Bearer {token}"))
                .body("firstName=Test&lastName=User&email=test2@example.com&password=mypassword&role=normal"),
        );
        assert_eq!(res.status, Status::Ok);
        let user = serde_json::from_slice::<api::User>(&res.body).unwrap();

        let stored_user = ctx
            .database
            .query::<User>(
                formatcp!("SELECT {} FROM users WHERE id = ? LIMIT 1", User::columns()),
                user.id,
            )
            .unwrap()
            .next()
            .map(|r| r.unwrap())
            .unwrap();

        assert_ne!(stored_user.password, "mypassword");
        assert!(stored_user.password.starts_with("$pbkdf2-sha256$"));
        assert!(pbkdf2::password_verify("mypassword", &stored_user.password).unwrap());
        assert!(!pbkdf2::password_verify("wrongpassword", &stored_user.password).unwrap());
    }

    #[test]
    fn test_users_notes() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // User should have no notes initially
        let res = router.handle(
            &Request::get(format!("http://localhost/api/users/{}/notes", user.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        assert!(
            serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
                .unwrap()
                .data
                .is_empty()
        );

        let note = insert_test_note(&ctx, user.id, None, "My first note");

        let res = router.handle(
            &Request::get(format!("http://localhost/api/users/{}/notes", user.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let notes = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].body, "My first note");
        assert_eq!(notes[0].user_id, note.user_id);
    }

    #[test]
    fn test_users_notes_excludes_pinned_and_archived() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        insert_test_note(&ctx, user.id, None, "Regular");
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                body: "Pinned".to_string(),
                is_pinned: true,
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                body: "Archived".to_string(),
                is_archived: true,
                ..Default::default()
            })
            .unwrap();

        let res = router.handle(
            &Request::get(format!("http://localhost/api/users/{}/notes", user.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].body, "Regular");
    }

    #[test]
    fn test_users_notes_pagination() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        for i in 1..=30 {
            insert_test_note(&ctx, user.id, None, &format!("Note {i}"));
        }

        let res = router.handle(
            &Request::get(format!(
                "http://localhost/api/users/{}/notes?limit=10&page=1",
                user.id
            ))
            .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 10);
        assert_eq!(response.pagination.page, 1);
        assert_eq!(response.pagination.limit, 10);
        assert_eq!(response.pagination.total, 30);

        let res = router.handle(
            &Request::get(format!(
                "http://localhost/api/users/{}/notes?limit=10&page=2",
                user.id
            ))
            .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 10);
        assert_eq!(response.pagination.page, 2);
    }

    #[test]
    fn test_users_notes_search() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        insert_test_note(&ctx, user.id, None, "Meeting notes from today");
        insert_test_note(&ctx, user.id, None, "Shopping list for tomorrow");

        let res = router.handle(
            &Request::get(format!(
                "http://localhost/api/users/{}/notes?q=meeting",
                user.id
            ))
            .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].body, "Meeting notes from today");
    }

    #[test]
    fn test_users_notes_search_by_title() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        insert_test_note(&ctx, user.id, Some("ProjectBeta"), "Some content");
        insert_test_note(&ctx, user.id, None, "Other content");

        let res = router.handle(
            &Request::get(format!(
                "http://localhost/api/users/{}/notes?q=ProjectBeta",
                user.id
            ))
            .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].title, Some("ProjectBeta".to_string()));
    }

    #[test]
    fn test_users_notes_admin_can_see_any_user_notes() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_admin, admin_token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);
        let (user, _) = create_test_user_with_session(&ctx);

        insert_test_note(&ctx, user.id, None, "User's private note");

        let res = router.handle(
            &Request::get(format!("http://localhost/api/users/{}/notes", user.id))
                .header("Authorization", format!("Bearer {admin_token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let notes = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].body, "User's private note");
    }

    #[test]
    fn test_users_notes_normal_user_cannot_see_other_notes() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_user1, user1_token) = create_test_user_with_session(&ctx);
        let (user2, _) = create_test_user_with_session(&ctx);

        insert_test_note(&ctx, user2.id, None, "User2's private note");

        let res = router.handle(
            &Request::get(format!("http://localhost/api/users/{}/notes", user2.id))
                .header("Authorization", format!("Bearer {user1_token}")),
        );
        assert_eq!(res.status, Status::Forbidden);
    }

    #[test]
    fn test_users_notes_not_found() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        let res = router.handle(
            &Request::get(format!(
                "http://localhost/api/users/{}/notes",
                Uuid::now_v7()
            ))
            .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::NotFound);
    }

    #[test]
    fn test_users_notes_pinned() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("User Pinned Note".to_string()),
                body: "User's pinned note".to_string(),
                is_pinned: true,
                ..Default::default()
            })
            .unwrap();
        insert_test_note(
            &ctx,
            user.id,
            Some("User Unpinned Note"),
            "User's unpinned note",
        );

        let res = router.handle(
            &Request::get(format!(
                "http://localhost/api/users/{}/notes/pinned",
                user.id
            ))
            .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert!(response.data[0].is_pinned);
    }

    #[test]
    fn test_users_notes_archived() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("User Archived Note".to_string()),
                body: "User's archived note".to_string(),
                is_archived: true,
                ..Default::default()
            })
            .unwrap();
        insert_test_note(
            &ctx,
            user.id,
            Some("User Active Note"),
            "User's active note",
        );

        let res = router.handle(
            &Request::get(format!(
                "http://localhost/api/users/{}/notes/archived",
                user.id
            ))
            .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert!(response.data[0].is_archived);
    }

    #[test]
    fn test_users_notes_trashed() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("User Trashed Note".to_string()),
                body: "User's trashed note".to_string(),
                is_trashed: true,
                ..Default::default()
            })
            .unwrap();
        insert_test_note(&ctx, user.id, Some("User Kept Note"), "User's kept note");

        let res = router.handle(
            &Request::get(format!(
                "http://localhost/api/users/{}/notes/trashed",
                user.id
            ))
            .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert!(response.data[0].is_trashed);
    }

    #[test]
    fn test_users_update_with_password_admin() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);
        let user = insert_test_user(&ctx, "John", "Doe", "john@example.com");

        let res = router.handle(
            &Request::put(format!("http://localhost/api/users/{}", user.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("firstName=John&lastName=Doe&email=john.new@example.com&theme=system&language=en&role=normal&password=newpassword99"),
        );
        assert_eq!(res.status, Status::Ok);

        let stored = ctx
            .database
            .query::<User>(
                formatcp!("SELECT {} FROM users WHERE id = ? LIMIT 1", User::columns()),
                user.id,
            )
            .unwrap()
            .next()
            .map(|r| r.unwrap())
            .unwrap();
        assert!(pbkdf2::password_verify("newpassword99", &stored.password).unwrap());
    }

    #[test]
    fn test_users_update_with_password_non_admin_ignored() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session_and_role(&ctx, UserRole::Normal);

        let res = router.handle(
            &Request::put(format!("http://localhost/api/users/{}", user.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("firstName=Test&lastName=User&email=test@example.com&theme=system&language=en&role=normal&password=newpassword99"),
        );
        // Update is allowed but password change is silently ignored
        assert_eq!(res.status, Status::Ok);

        let stored = ctx
            .database
            .query::<User>(
                formatcp!("SELECT {} FROM users WHERE id = ? LIMIT 1", User::columns()),
                user.id,
            )
            .unwrap()
            .next()
            .map(|r| r.unwrap())
            .unwrap();
        assert!(pbkdf2::password_verify("password123", &stored.password).unwrap());
    }

    #[test]
    fn test_users_update_with_password_too_short() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);
        let user = insert_test_user(&ctx, "John", "Doe", "john@example.com");

        let res = router.handle(
            &Request::put(format!("http://localhost/api/users/{}", user.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("firstName=John&lastName=Doe&email=john@example.com&theme=system&language=en&role=normal&password=short"),
        );
        assert_eq!(res.status, Status::BadRequest);
        let report = serde_json::from_slice::<api::Report>(&res.body).unwrap();
        assert!(report.0.contains_key("password"));
    }
}
