/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::{execute_args, query_args};
use chrono::Utc;
use const_format::formatcp;
use small_http::{Request, Response, Status};
use uuid::Uuid;
use validate::Validate;

use crate::api;
use crate::context::{Context, DatabaseHelpers};
use crate::controllers::{get_auth_user, not_found};
use crate::models::user::{UserRole, policies};
use crate::models::{IndexQuery, User};

pub(crate) fn users_index(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let auth_user = match get_auth_user(req, ctx) {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Check authorization
    if !policies::can_index(&auth_user) {
        return Response::with_status(Status::Forbidden);
    }

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

    // Get users
    let search_query = format!("%{}%", query.query.replace("%", "\\%"));
    let total = ctx.database.query_some::<i64>(
        "SELECT COUNT(id) FROM users WHERE first_name LIKE ? OR last_name LIKE ? OR email LIKE ?",
        (
            search_query.clone(),
            search_query.clone(),
            search_query.clone(),
        ),
    );
    let users = query_args!(
        User,
        ctx.database,
        formatcp!(
            "SELECT {} FROM users WHERE first_name LIKE :search_query OR last_name LIKE :search_query OR email LIKE :search_query ORDER BY created_at DESC LIMIT :limit OFFSET :offset",
            User::columns()
        ),
        Args {
            search_query: search_query,
            limit: query.limit,
            offset: (query.page - 1) * query.limit
        }
    )
    .map(Into::<api::User>::into)
    .collect::<Vec<_>>();

    // Return users
    Response::with_json(api::UserIndexResponse {
        pagination: api::Pagination {
            page: query.page,
            limit: query.limit,
            total,
        },
        data: users,
    })
}

#[derive(Validate)]
struct UserCreateBody {
    #[validate(ascii, length(min = 1, max = 128))]
    first_name: String,
    #[validate(ascii, length(min = 1, max = 128))]
    last_name: String,
    #[validate(email)]
    email: String,
    #[validate(ascii, length(min = 8, max = 128))]
    password: String,
    role: UserRole,
}

impl From<api::UserCreateBody> for UserCreateBody {
    fn from(body: api::UserCreateBody) -> Self {
        Self {
            first_name: body.first_name,
            last_name: body.last_name,
            email: body.email,
            password: body.password,
            role: body.role.into(),
        }
    }
}

pub(crate) fn users_create(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let auth_user = match get_auth_user(req, ctx) {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Check authorization
    if !policies::can_create(&auth_user) {
        return Response::with_status(Status::Forbidden);
    }

    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::UserCreateBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => Into::<UserCreateBody>::into(body),
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(report) = body.validate() {
        return Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report));
    }

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
    ctx.database.insert_user(user.clone());

    // Return created user
    Response::with_json(Into::<api::User>::into(user))
}

pub(crate) fn get_user(req: &Request, ctx: &Context) -> Option<User> {
    // Parse user id from url
    let user_id = match req
        .params
        .get("user_id")
        .expect("user_id param should be present")
        .parse::<Uuid>()
    {
        Ok(id) => id,
        Err(_) => return None,
    };

    // Get user
    ctx.database
        .query::<User>(
            formatcp!("SELECT {} FROM users WHERE id = ? LIMIT 1", User::columns()),
            user_id,
        )
        .next()
}

pub(crate) fn users_show(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let auth_user = match get_auth_user(req, ctx) {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Get user
    let user = match get_user(req, ctx) {
        Some(user) => user,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_show(&auth_user, &user) {
        return Response::with_status(Status::Forbidden);
    }

    // Return user
    Response::with_json(Into::<api::User>::into(user))
}

#[derive(Validate)]
struct UserUpdateBody {
    #[validate(ascii, length(min = 1, max = 128))]
    first_name: String,
    #[validate(ascii, length(min = 1, max = 128))]
    last_name: String,
    #[validate(email)]
    email: String,
    role: UserRole,
}

impl From<api::UserUpdateBody> for UserUpdateBody {
    fn from(body: api::UserUpdateBody) -> Self {
        Self {
            first_name: body.first_name,
            last_name: body.last_name,
            email: body.email,
            role: body.role.into(),
        }
    }
}

pub(crate) fn users_update(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let auth_user = match get_auth_user(req, ctx) {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Get user
    let mut user = match get_user(req, ctx) {
        Some(user) => user,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_update(&auth_user, &user) {
        return Response::with_status(Status::Forbidden);
    }

    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::UserUpdateBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => Into::<UserUpdateBody>::into(body),
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(report) = body.validate() {
        return Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report));
    }

    // Update user
    user.first_name = body.first_name;
    user.last_name = body.last_name;
    user.email = body.email;
    user.role = body.role;
    user.updated_at = Utc::now();
    execute_args!(
        ctx.database,
        "UPDATE users SET first_name = :first_name, last_name = :last_name, email = :email, role = :role, updated_at = :updated_at WHERE id = :id",
        Args {
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            email: user.email.clone(),
            role: user.role,
            updated_at: user.updated_at,
            id: user.id
        }
    );

    // Return updated user
    Response::with_json(Into::<api::User>::into(user))
}

#[derive(Validate)]
struct UserChangePasswordBody {
    #[validate(ascii, length(min = 8, max = 128))]
    old_password: String,
    #[validate(ascii, length(min = 8, max = 128))]
    new_password: String,
}

impl From<api::UserChangePasswordBody> for UserChangePasswordBody {
    fn from(body: api::UserChangePasswordBody) -> Self {
        Self {
            old_password: body.old_password,
            new_password: body.new_password,
        }
    }
}

pub(crate) fn users_change_password(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let auth_user = match get_auth_user(req, ctx) {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Get user
    let mut user = match get_user(req, ctx) {
        Some(user) => user,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_update(&auth_user, &user) {
        return Response::with_status(Status::Forbidden);
    }

    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::UserChangePasswordBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => Into::<UserChangePasswordBody>::into(body),
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(report) = body.validate() {
        return Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report));
    }

    // Verify old password
    match pbkdf2::password_verify(&body.old_password, &user.password) {
        Ok(true) => {}
        Ok(false) => return Response::with_status(Status::Unauthorized),
        Err(_) => return Response::with_status(Status::InternalServerError),
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
    );

    // Success response
    Response::new()
}

pub(crate) fn users_delete(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let auth_user = match get_auth_user(req, ctx) {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Get user
    let user = match get_user(req, ctx) {
        Some(user) => user,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_delete(&auth_user, &user) {
        return Response::with_status(Status::Forbidden);
    }

    // Delete user
    ctx.database
        .execute("DELETE FROM users WHERE id = ?", user.id);

    // Success response
    Response::new()
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;
    use crate::context::test_helpers::create_test_user_with_session_and_role;
    use crate::models::user::UserRole;
    use crate::router;

    #[test]
    fn test_users_index() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Fetch /users check if user is there (the test user)
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
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john.doe@example.com".to_string(),
            password: pbkdf2::password_hash("password123"),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        // Fetch /users check if both users are there
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
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create multiple users
        ctx.database.insert_user(User {
            first_name: "Alice".to_string(),
            last_name: "Smith".to_string(),
            email: "alice@example.com".to_string(),
            password: pbkdf2::password_hash("password123"),
            ..Default::default()
        });
        ctx.database.insert_user(User {
            first_name: "Bob".to_string(),
            last_name: "Jones".to_string(),
            email: "bob@example.com".to_string(),
            password: pbkdf2::password_hash("password123"),
            ..Default::default()
        });

        // Search for "Alice"
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
    fn test_users_index_pagination() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create multiple users (test user already exists, so create 29 more for 30 total)
        for i in 1..=29 {
            ctx.database.insert_user(User {
                first_name: format!("User{i}"),
                last_name: "Test".to_string(),
                email: format!("user{i}@example.com"),
                password: pbkdf2::password_hash("password123"),
                ..Default::default()
            });
        }

        // Fetch /users with limit 10 and page 1
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

        // Fetch /users with limit 5 and page 2
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
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create user
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
    fn test_users_show() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create user
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            password: pbkdf2::password_hash("password123"),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        // Fetch /users/:user_id check if user is there
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
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Fetch user by random id should be 404 Not Found
        let res = router.handle(
            &Request::get(format!("http://localhost/api/users/{}", Uuid::now_v7()))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::NotFound);
    }

    #[test]
    fn test_users_update() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create user
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            password: pbkdf2::password_hash("password123"),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        // Update user
        let res = router.handle(
            &Request::put(format!("http://localhost/api/users/{}", user.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("firstName=John&lastName=Smith&email=john.smith@example.com&role=normal"),
        );
        assert_eq!(res.status, Status::Ok);
        let updated_user = serde_json::from_slice::<api::User>(&res.body).unwrap();
        assert_eq!(updated_user.last_name, "Smith");
        assert_eq!(updated_user.email, "john.smith@example.com");
    }

    #[test]
    fn test_users_update_validation_error() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create user
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            password: pbkdf2::password_hash("password123"),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        // Update user with validation errors
        let res = router.handle(
            &Request::put(format!("http://localhost/api/users/{}", user.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("firstName=&lastName=Smith&email=invalid-email"),
        );
        assert_eq!(res.status, Status::BadRequest);
    }

    #[test]
    fn test_users_change_password() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create user
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            password: pbkdf2::password_hash("password123"),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        // Change password with correct old password
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
            .next()
            .unwrap();
        assert!(pbkdf2::password_verify("newpassword456", &stored_user.password).unwrap());
    }

    #[test]
    fn test_users_change_password_incorrect_old_password() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create user
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            password: pbkdf2::password_hash("password123"),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        // Try to change password with incorrect old password
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
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create user
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            password: pbkdf2::password_hash("password123"),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        // Try to change password with validation errors (short password)
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
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create user
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            password: pbkdf2::password_hash("password123"),
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        // Delete user
        let res = router.handle(
            &Request::delete(format!("http://localhost/api/users/{}", user.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);

        // Fetch /users check if only test user remains
        let res = router.handle(
            &Request::get("http://localhost/api/users")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let users = serde_json::from_slice::<api::UserIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(users.len(), 1); // Only test user remains
    }

    #[test]
    fn test_password_hashing() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create user with password
        let res = router.handle(
            &Request::post("http://localhost/api/users")
                .header("Authorization", format!("Bearer {token}"))
                .body("firstName=Test&lastName=User&email=test2@example.com&password=mypassword&role=normal"),
        );
        assert_eq!(res.status, Status::Ok);
        let user = serde_json::from_slice::<api::User>(&res.body).unwrap();

        // Fetch user from database and verify password is hashed
        let stored_user = ctx
            .database
            .query::<User>(
                formatcp!("SELECT {} FROM users WHERE id = ? LIMIT 1", User::columns()),
                user.id,
            )
            .next()
            .unwrap();

        // Password should be hashed (not plain text)
        assert_ne!(stored_user.password, "mypassword");
        assert!(stored_user.password.starts_with("$pbkdf2-sha256$"));

        // Verify password can be verified
        assert!(pbkdf2::password_verify("mypassword", &stored_user.password).unwrap());
        assert!(!pbkdf2::password_verify("wrongpassword", &stored_user.password).unwrap());
    }
}
