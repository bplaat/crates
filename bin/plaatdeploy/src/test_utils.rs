/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::LazyLock;
use std::time::Duration;

use chrono::Utc;
use uuid::Uuid;

use crate::consts::SESSION_EXPIRY_SECONDS;
use crate::context::{Context, DatabaseHelpers};
use crate::models::{
    Deployment, DeploymentStatus, Project, Session, Team, TeamGitHubToken, TeamUser, TeamUserRole,
    User, UserRole,
};
use crate::utils::password_hash;

pub(crate) static TEST_PASSWORD_HASH: LazyLock<String> =
    LazyLock::new(|| password_hash("password123"));

pub(crate) fn insert_user(ctx: &Context, role: UserRole) -> User {
    let user = User {
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        email: format!("test-{}@example.com", Uuid::now_v7()),
        password: TEST_PASSWORD_HASH.to_string(),
        role,
        ..Default::default()
    };
    ctx.database.insert_user(user.clone()).expect("insert user");
    user
}

pub(crate) fn insert_session(ctx: &Context, user_id: Uuid, token: &str) -> Session {
    let session = Session {
        user_id,
        token: token.to_string(),
        ip_address: "127.0.0.1".to_string(),
        expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
        ..Default::default()
    };
    ctx.database
        .insert_session(session.clone())
        .expect("insert session");
    session
}

pub(crate) fn insert_user_with_session(ctx: &Context, role: UserRole) -> (User, String) {
    let user = insert_user(ctx, role);
    let token = format!("test-token-{}", user.id);
    insert_session(ctx, user.id, &token);
    (user, token)
}

pub(crate) fn insert_team_with_member(ctx: &Context, user_id: Uuid, role: TeamUserRole) -> Team {
    let team = Team {
        name: format!("Team {}", Uuid::now_v7()),
        ..Default::default()
    };
    ctx.database.insert_team(team.clone()).expect("insert team");
    ctx.database
        .insert_team_user(TeamUser {
            team_id: team.id,
            user_id,
            role,
            ..Default::default()
        })
        .expect("insert team user");
    team
}

pub(crate) fn insert_project(ctx: &Context, team_id: Uuid) -> Project {
    let project = Project {
        team_id,
        name: format!("project-{}", Uuid::now_v7()),
        github_repo: "owner/repo".to_string(),
        github_branch: "main".to_string(),
        base_dir: String::new(),
        ..Default::default()
    };
    ctx.database
        .insert_project(project.clone())
        .expect("insert project");
    project
}

pub(crate) fn insert_deployment(ctx: &Context, project_id: Uuid) -> Deployment {
    let deployment = Deployment {
        project_id,
        commit_sha: "abc123".to_string(),
        commit_message: "Test commit".to_string(),
        status: DeploymentStatus::Succeeded,
        log: Some("ok".to_string()),
        ..Default::default()
    };
    ctx.database
        .insert_deployment(deployment.clone())
        .expect("insert deployment");
    deployment
}

pub(crate) fn insert_github_token(
    ctx: &Context,
    team_id: Uuid,
    webhook_secret: &str,
) -> TeamGitHubToken {
    let token = TeamGitHubToken {
        team_id,
        access_token: "github_pat_test".to_string(),
        webhook_secret: webhook_secret.to_string(),
        account_login: "test-user".to_string(),
        ..Default::default()
    };
    ctx.database
        .execute(
            const_format::formatcp!(
                "INSERT INTO team_github_tokens ({}) VALUES ({})",
                TeamGitHubToken::columns(),
                TeamGitHubToken::values()
            ),
            token.clone(),
        )
        .expect("insert github connection");
    token
}
