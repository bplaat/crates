/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::LazyLock;
use std::time::Duration;

use chrono::Utc;

use crate::consts::SESSION_EXPIRY_SECONDS;
use crate::context::{Context, DatabaseHelpers};
use crate::models::{Project, Session, User};
use crate::utils::password_hash;

pub(crate) static TEST_PASSWORD_HASH: LazyLock<String> =
    LazyLock::new(|| password_hash("password123"));

pub(crate) fn insert_test_user(ctx: &Context) -> User {
    let user = User {
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        email: format!("test-{}@example.com", uuid::Uuid::now_v7()),
        password: TEST_PASSWORD_HASH.to_string(),
        ..Default::default()
    };
    ctx.database
        .insert_user(user.clone())
        .expect("insert test user");
    user
}

pub(crate) fn insert_test_session(ctx: &Context, user_id: uuid::Uuid, token: &str) -> Session {
    let session = Session {
        user_id,
        token: token.to_string(),
        expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
        ..Default::default()
    };
    ctx.database
        .insert_session(session.clone())
        .expect("insert test session");
    session
}

pub(crate) fn authenticated_user(ctx: &Context) -> (User, String) {
    let user = insert_test_user(ctx);
    let token = format!("test-token-{}", user.id);
    insert_test_session(ctx, user.id, &token);
    (user, token)
}

pub(crate) fn insert_test_project(ctx: &Context, user_id: uuid::Uuid) -> Project {
    let project = Project {
        user_id,
        name: "Example Project".to_string(),
        slug: format!("example-{}", uuid::Uuid::now_v7()),
        github_repo: "example/project".to_string(),
        github_branch: "main".to_string(),
        webhook_secret: "secret".to_string(),
        ..Default::default()
    };
    ctx.database
        .insert_project(project.clone())
        .expect("insert test project");
    project
}
