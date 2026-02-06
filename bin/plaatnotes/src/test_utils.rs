/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::time::Duration;

use chrono::Utc;

use crate::consts::SESSION_EXPIRY_SECONDS;
use crate::context::{Context, DatabaseHelpers};
use crate::models::{Session, User, UserRole};

// Precomputed hash for "password123" to avoid computing it in every test
pub(crate) const TEST_PASSWORD_HASH: &str =
    "$pbkdf2-sha256$t=100000$ehJ7aeA7FWwbPkeY4a2pJA$mqqvN/B9sR2BW6bk9wPUWjcsz03E6bJXWIkCis7GvJk";

// Creates a test user and returns (user, token)
pub(crate) fn create_test_user_with_session(ctx: &Context) -> (User, String) {
    create_test_user_with_session_and_role(ctx, UserRole::Normal)
}

// Creates a test user with specific role and returns (user, token)
pub(crate) fn create_test_user_with_session_and_role(
    ctx: &Context,
    role: UserRole,
) -> (User, String) {
    // Create test user with specified role
    let user = User {
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        email: format!("test-{}@example.com", uuid::Uuid::now_v7()),
        password: TEST_PASSWORD_HASH.to_string(),
        role,
        ..Default::default()
    };
    ctx.database.insert_user(user.clone());

    // Create session with token
    let token = format!("test-token-{}", user.id);
    let session = Session {
        user_id: user.id,
        token: token.clone(),
        expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
        ..Default::default()
    };
    ctx.database.insert_session(session);

    (user, token)
}
