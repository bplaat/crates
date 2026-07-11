/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use small_http::{Request, Status};

use crate::api;
use crate::context::Context;
use crate::models::{BuildType, ProjectStatus, TeamUserRole, UserRole};
use crate::test_utils::*;

fn test_router(ctx: &Context) -> small_router::Router<Context> {
    crate::router(ctx.clone())
}

fn auth(token: &str) -> String {
    format!("Bearer {token}")
}

fn form(pairs: &[(&str, &str)]) -> String {
    serde_urlencoded::to_string(pairs).expect("encode form")
}

fn form_body(req: Request, pairs: &[(&str, &str)]) -> Request {
    req.header("Content-Type", "application/x-www-form-urlencoded")
        .body(form(pairs))
}

#[test]
fn home_and_cors_layers_work() {
    let ctx = Context::with_test_database().expect("test database");
    let router = test_router(&ctx);

    let res = router.handle(&Request::get("http://localhost/api"));
    assert_eq!(res.status, Status::Ok);
    assert_eq!(
        res.headers.get("Access-Control-Allow-Origin"),
        Some(ctx.server_origin.as_str())
    );
    assert!(
        String::from_utf8(res.body)
            .expect("body")
            .contains("plaatdeploy v")
    );

    let res = router.handle(
        &Request::options("http://localhost/api/auth/login")
            .header("Access-Control-Request-Method", "POST"),
    );
    assert_eq!(res.status, Status::NoContent);
    assert_eq!(
        res.headers.get("Access-Control-Allow-Origin"),
        Some(ctx.server_origin.as_str())
    );
    assert_eq!(
        res.headers.get("Access-Control-Allow-Methods"),
        Some("GET, POST, PUT, PATCH, DELETE, OPTIONS")
    );
}

#[test]
fn auth_login_success_and_validate() {
    let ctx = Context::with_test_database().expect("test database");
    let user = insert_user(&ctx, UserRole::Admin);
    let router = test_router(&ctx);

    let res = router.handle(&form_body(
        Request::post("http://localhost/api/auth/login"),
        &[("email", user.email.as_str()), ("password", "password123")],
    ));
    assert_eq!(res.status, Status::Ok);
    let login: api::LoginResponse = res.into_json().expect("login json");
    assert_eq!(login.user_id, user.id);

    let res = router.handle(
        &Request::get("http://localhost/api/auth/validate")
            .header("Authorization", auth(&login.token)),
    );
    assert_eq!(res.status, Status::Ok);
    let validate: api::AuthValidateResponse = res.into_json().expect("validate json");
    assert_eq!(validate.user.id, user.id);
}

#[test]
fn auth_login_rejects_bad_password_and_invalid_body() {
    let ctx = Context::with_test_database().expect("test database");
    let user = insert_user(&ctx, UserRole::Normal);
    let router = test_router(&ctx);

    let res = router.handle(&form_body(
        Request::post("http://localhost/api/auth/login"),
        &[
            ("email", user.email.as_str()),
            ("password", "wrong-password"),
        ],
    ));
    assert_eq!(res.status, Status::Unauthorized);

    let res = router.handle(&form_body(
        Request::post("http://localhost/api/auth/login"),
        &[("email", "not-an-email"), ("password", "password123")],
    ));
    assert_eq!(res.status, Status::BadRequest);
    let report: api::Report = res.into_json().expect("report json");
    assert!(report.0.contains_key("email"));
}

#[test]
fn auth_required_routes_reject_missing_and_invalid_tokens() {
    let ctx = Context::with_test_database().expect("test database");
    let router = test_router(&ctx);

    let res = router.handle(&Request::get("http://localhost/api/users"));
    assert_eq!(res.status, Status::Unauthorized);

    let res = router.handle(
        &Request::get("http://localhost/api/users").header("Authorization", "Bearer missing"),
    );
    assert_eq!(res.status, Status::Unauthorized);
}

#[test]
fn users_admin_crud_and_validation() {
    let ctx = Context::with_test_database().expect("test database");
    let (_, token) = insert_user_with_session(&ctx, UserRole::Admin);
    let router = test_router(&ctx);

    let res = router.handle(&form_body(
        Request::post("http://localhost/api/users").header("Authorization", auth(&token)),
        &[
            ("first_name", "Alice"),
            ("last_name", "Admin"),
            ("email", "alice@example.com"),
            ("password", "password123"),
            ("role", "admin"),
        ],
    ));
    assert_eq!(res.status, Status::Ok);
    let created: api::User = res.into_json().expect("user json");
    assert_eq!(created.email, "alice@example.com");
    assert!(created.role == api::UserRole::Admin);

    let res = router.handle(&form_body(
        Request::post("http://localhost/api/users").header("Authorization", auth(&token)),
        &[
            ("first_name", "Bob"),
            ("last_name", "Duplicate"),
            ("email", "alice@example.com"),
            ("password", "password123"),
            ("role", "normal"),
        ],
    ));
    assert_eq!(res.status, Status::BadRequest);
    let report: api::Report = res.into_json().expect("report json");
    assert!(report.0.contains_key("email"));

    let res = router.handle(&form_body(
        Request::put(format!("http://localhost/api/users/{}", created.id))
            .header("Authorization", auth(&token)),
        &[
            ("first_name", "Alice"),
            ("last_name", "Updated"),
            ("email", "alice-updated@example.com"),
            ("password", "password456"),
            ("role", "normal"),
        ],
    ));
    assert_eq!(res.status, Status::Ok);
    let updated: api::User = res.into_json().expect("user json");
    assert_eq!(updated.last_name, "Updated");
    assert!(updated.role == api::UserRole::Normal);

    let res = router.handle(
        &Request::delete(format!("http://localhost/api/users/{}", created.id))
            .header("Authorization", auth(&token)),
    );
    assert_eq!(res.status, Status::BadRequest);
    let report: api::Report = res.into_json().expect("report json");
    assert!(report.0.contains_key("user"));

    let orphan = insert_user(&ctx, UserRole::Normal);
    let res = router.handle(
        &Request::delete(format!("http://localhost/api/users/{}", orphan.id))
            .header("Authorization", auth(&token)),
    );
    assert_eq!(res.status, Status::Ok);

    let res = router.handle(
        &Request::get(format!("http://localhost/api/users/{}", orphan.id))
            .header("Authorization", auth(&token)),
    );
    assert_eq!(res.status, Status::NotFound);
}

#[test]
fn normal_user_cannot_admin_user_index_or_create() {
    let ctx = Context::with_test_database().expect("test database");
    let (_, token) = insert_user_with_session(&ctx, UserRole::Normal);
    let router = test_router(&ctx);

    let res = router
        .handle(&Request::get("http://localhost/api/users").header("Authorization", auth(&token)));
    assert_eq!(res.status, Status::Forbidden);

    let res = router.handle(&form_body(
        Request::post("http://localhost/api/users").header("Authorization", auth(&token)),
        &[
            ("first_name", "No"),
            ("last_name", "Access"),
            ("email", "no@example.com"),
            ("password", "password123"),
            ("role", "normal"),
        ],
    ));
    assert_eq!(res.status, Status::Forbidden);
}

#[test]
fn teams_owner_can_manage_members_but_member_cannot_update_team() {
    let ctx = Context::with_test_database().expect("test database");
    let (owner, owner_token) = insert_user_with_session(&ctx, UserRole::Normal);
    let (member, member_token) = insert_user_with_session(&ctx, UserRole::Normal);
    let team = insert_team_with_member(&ctx, owner.id, TeamUserRole::Owner);
    let router = test_router(&ctx);

    let res = router.handle(&form_body(
        Request::post(format!("http://localhost/api/teams/{}/members", team.id))
            .header("Authorization", auth(&owner_token)),
        &[("email", member.email.as_str()), ("role", "member")],
    ));
    assert_eq!(res.status, Status::Ok);
    let team_user: api::TeamUser = res.into_json().expect("team user json");
    assert_eq!(team_user.user_id, member.id);
    assert!(team_user.role == api::TeamUserRole::Member);

    let res = router.handle(&form_body(
        Request::put(format!("http://localhost/api/teams/{}", team.id))
            .header("Authorization", auth(&member_token)),
        &[("name", "Renamed")],
    ));
    assert_eq!(res.status, Status::Forbidden);
}

#[test]
fn teams_index_show_update_and_delete_empty_team() {
    let ctx = Context::with_test_database().expect("test database");
    let (owner, token) = insert_user_with_session(&ctx, UserRole::Normal);
    let team = insert_team_with_member(&ctx, owner.id, TeamUserRole::Owner);
    let router = test_router(&ctx);

    let res = router
        .handle(&Request::get("http://localhost/api/teams").header("Authorization", auth(&token)));
    assert_eq!(res.status, Status::Ok);
    let teams: api::TeamIndexResponse = res.into_json().expect("teams json");
    assert_eq!(teams.pagination.total, 1);

    let res = router.handle(
        &Request::get(format!("http://localhost/api/teams/{}", team.id))
            .header("Authorization", auth(&token)),
    );
    assert_eq!(res.status, Status::Ok);
    let show: api::TeamShowResponse = res.into_json().expect("team show json");
    assert_eq!(show.team.id, team.id);
    assert_eq!(show.members.len(), 1);

    let res = router.handle(&form_body(
        Request::put(format!("http://localhost/api/teams/{}", team.id))
            .header("Authorization", auth(&token)),
        &[("name", "Renamed Team")],
    ));
    assert_eq!(res.status, Status::Ok);
    let updated: api::Team = res.into_json().expect("team json");
    assert_eq!(updated.name, "Renamed Team");

    let res = router.handle(
        &Request::delete(format!("http://localhost/api/teams/{}", team.id))
            .header("Authorization", auth(&token)),
    );
    assert_eq!(res.status, Status::Ok);
}

#[test]
fn team_delete_rejects_team_with_projects() {
    let ctx = Context::with_test_database().expect("test database");
    let (owner, token) = insert_user_with_session(&ctx, UserRole::Normal);
    let team = insert_team_with_member(&ctx, owner.id, TeamUserRole::Owner);
    insert_project(&ctx, team.id);
    let router = test_router(&ctx);

    let res = router.handle(
        &Request::delete(format!("http://localhost/api/teams/{}", team.id))
            .header("Authorization", auth(&token)),
    );
    assert_eq!(res.status, Status::BadRequest);
    let report: api::Report = res.into_json().expect("report json");
    assert!(report.0.contains_key("team"));
}

#[test]
fn projects_are_scoped_to_team_membership() {
    let ctx = Context::with_test_database().expect("test database");
    let (owner, owner_token) = insert_user_with_session(&ctx, UserRole::Normal);
    let (_, outsider_token) = insert_user_with_session(&ctx, UserRole::Normal);
    let team = insert_team_with_member(&ctx, owner.id, TeamUserRole::Owner);
    let project = insert_project(&ctx, team.id);
    let router = test_router(&ctx);

    let res = router.handle(
        &Request::get(format!("http://localhost/api/projects/{}", project.id))
            .header("Authorization", auth(&owner_token)),
    );
    assert_eq!(res.status, Status::Ok);

    let res = router.handle(
        &Request::get(format!("http://localhost/api/projects/{}", project.id))
            .header("Authorization", auth(&outsider_token)),
    );
    assert_eq!(res.status, Status::Forbidden);
}

#[test]
fn projects_reject_unsafe_name_and_base_dir() {
    let ctx = Context::with_test_database().expect("test database");
    let (owner, token) = insert_user_with_session(&ctx, UserRole::Normal);
    let team = insert_team_with_member(&ctx, owner.id, TeamUserRole::Owner);
    let project = insert_project(&ctx, team.id);
    let router = test_router(&ctx);
    let team_id = team.id.to_string();

    let res = router.handle(&form_body(
        Request::post("http://localhost/api/projects").header("Authorization", auth(&token)),
        &[
            ("name", "../escape"),
            ("github_repo", "owner/repo"),
            ("github_branch", "main"),
            ("base_dir", ""),
            ("team_id", team_id.as_str()),
        ],
    ));
    assert_eq!(res.status, Status::BadRequest);
    let report: api::Report = res.into_json().expect("report json");
    assert!(report.0.contains_key("name"));

    let res = router.handle(&form_body(
        Request::put(format!("http://localhost/api/projects/{}", project.id))
            .header("Authorization", auth(&token)),
        &[
            ("name", "safe-project"),
            ("github_repo", "owner/repo"),
            ("github_branch", "main"),
            ("base_dir", "../outside"),
            ("team_id", team_id.as_str()),
        ],
    ));
    assert_eq!(res.status, Status::BadRequest);
    let report: api::Report = res.into_json().expect("report json");
    assert!(report.0.contains_key("baseDir"));

    let res = router.handle(&form_body(
        Request::put(format!("http://localhost/api/projects/{}", project.id))
            .header("Authorization", auth(&token)),
        &[
            ("name", "safe-project"),
            ("github_repo", "owner/repo"),
            ("github_branch", "main"),
            ("base_dir", "/absolute"),
            ("team_id", team_id.as_str()),
        ],
    ));
    assert_eq!(res.status, Status::BadRequest);
    let report: api::Report = res.into_json().expect("report json");
    assert!(report.0.contains_key("baseDir"));

    let res = router.handle(&form_body(
        Request::put(format!("http://localhost/api/projects/{}", project.id))
            .header("Authorization", auth(&token)),
        &[
            ("name", "safe-project"),
            ("github_repo", "owner/repo"),
            ("github_branch", "main"),
            ("base_dir", ""),
            ("container_port", "70000"),
            ("team_id", team_id.as_str()),
        ],
    ));
    assert_eq!(res.status, Status::BadRequest);
    let report: api::Report = res.into_json().expect("report json");
    assert!(report.0.contains_key("containerPort"));
}

#[cfg(unix)]
#[test]
fn static_spa_fallback_rejects_symlink_escape() {
    let mut ctx = Context::with_test_database().expect("test database");
    ctx.deployments_origin = "https://*.projects.example.test".to_string();
    ctx.deployments_host = "projects.example.test".to_string();
    let user = insert_user(&ctx, UserRole::Normal);
    let team = insert_team_with_member(&ctx, user.id, TeamUserRole::Owner);
    let mut project = insert_project(&ctx, team.id);
    let deploy_path = std::env::temp_dir().join(format!("plaatdeploy-test-{}", project.id));
    let repo_dir = deploy_path.join(&project.name).join("repo");
    let secret = deploy_path.join("secret.txt");
    std::fs::create_dir_all(&repo_dir).expect("create repo dir");
    std::fs::write(&secret, "secret").expect("write secret");
    std::os::unix::fs::symlink(&secret, repo_dir.join("index.html")).expect("symlink index");

    project.build_type = BuildType::Static;
    project.status = ProjectStatus::Running;
    ctx.database
        .execute(
            "UPDATE projects SET build_type = ?, status = ? WHERE id = ?",
            (project.build_type as i64, project.status as i64, project.id),
        )
        .expect("update project");

    let router = test_router(&ctx);
    let res = crate::proxy::dispatch(
        &Request::get(format!(
            "https://{}.projects.example.test/missing",
            project.name
        ))
        .header("Host", format!("{}.projects.example.test", project.name)),
        &ctx,
        &router,
        &ctx.server_host,
        deploy_path.to_str().expect("deploy path"),
    );
    assert_eq!(res.status, Status::Forbidden);
    assert_ne!(res.body, b"secret");

    std::fs::remove_dir_all(&deploy_path).ok();
}

#[test]
fn projects_index_update_deployments_and_delete() {
    let ctx = Context::with_test_database().expect("test database");
    let (owner, token) = insert_user_with_session(&ctx, UserRole::Normal);
    let team = insert_team_with_member(&ctx, owner.id, TeamUserRole::Owner);
    let project = insert_project(&ctx, team.id);
    let deployment = insert_deployment(&ctx, project.id);
    let router = test_router(&ctx);

    let res = router.handle(
        &Request::get("http://localhost/api/projects").header("Authorization", auth(&token)),
    );
    assert_eq!(res.status, Status::Ok);
    let projects: api::ProjectIndexResponse = res.into_json().expect("projects json");
    assert_eq!(projects.pagination.total, 1);

    let team_id = team.id.to_string();
    let res = router.handle(&form_body(
        Request::put(format!("http://localhost/api/projects/{}", project.id))
            .header("Authorization", auth(&token)),
        &[
            ("name", "updated-project"),
            ("github_repo", "owner/repo"),
            ("github_branch", "main"),
            ("base_dir", "web"),
            ("container_port", "8080"),
            ("team_id", team_id.as_str()),
        ],
    ));
    assert_eq!(res.status, Status::Ok);
    let updated: api::Project = res.into_json().expect("project json");
    assert_eq!(updated.name, "updated-project");
    assert_eq!(updated.base_dir, "web");
    assert_eq!(updated.container_port, Some(8080));

    let res = router.handle(
        &Request::get(format!(
            "http://localhost/api/projects/{}/deployments",
            project.id
        ))
        .header("Authorization", auth(&token)),
    );
    assert_eq!(res.status, Status::Ok);
    let deployments: api::DeploymentIndexResponse = res.into_json().expect("deployments json");
    assert_eq!(deployments.pagination.total, 1);
    assert_eq!(deployments.data[0].id, deployment.id);

    let res = router.handle(
        &Request::post(format!(
            "http://localhost/api/projects/{}/deploy",
            project.id
        ))
        .header("Authorization", auth(&token)),
    );
    assert_eq!(res.status, Status::Ok);
    let manual_deploy: api::Deployment = res.into_json().expect("deployment json");
    assert_eq!(manual_deploy.project_id, project.id);
    assert_eq!(manual_deploy.commit_sha, "manual");

    let res = router.handle(
        &Request::delete(format!("http://localhost/api/projects/{}", project.id))
            .header("Authorization", auth(&token)),
    );
    assert_eq!(res.status, Status::Ok);

    let res = router.handle(
        &Request::get(format!("http://localhost/api/projects/{}", project.id))
            .header("Authorization", auth(&token)),
    );
    assert_eq!(res.status, Status::NotFound);
}

#[test]
fn github_team_status_and_empty_repo_branch_lists() {
    let ctx = Context::with_test_database().expect("test database");
    let (owner, owner_token) = insert_user_with_session(&ctx, UserRole::Normal);
    let team = insert_team_with_member(&ctx, owner.id, TeamUserRole::Owner);
    let router = test_router(&ctx);

    let res = router.handle(
        &Request::get(format!("http://localhost/api/teams/{}/github", team.id))
            .header("Authorization", auth(&owner_token)),
    );
    assert_eq!(res.status, Status::Ok);
    let status: api::TeamGithubStatusResponse = res.into_json().expect("github status json");
    assert!(!status.connected);

    let res = router.handle(
        &Request::get(format!(
            "http://localhost/api/teams/{}/github/repositories",
            team.id
        ))
        .header("Authorization", auth(&owner_token)),
    );
    assert_eq!(res.status, Status::Ok);
    let repos: api::GithubRepositoryIndexResponse = res.into_json().expect("repos json");
    assert!(repos.data.is_empty());

    let res = router.handle(
        &Request::get(format!(
            "http://localhost/api/teams/{}/github/branches?repository=owner/repo",
            team.id
        ))
        .header("Authorization", auth(&owner_token)),
    );
    assert_eq!(res.status, Status::Ok);
    let branches: api::GithubBranchIndexResponse = res.into_json().expect("branches json");
    assert!(branches.data.is_empty());
}

#[test]
fn sessions_index_and_delete_are_owner_scoped() {
    let ctx = Context::with_test_database().expect("test database");
    let (user, token) = insert_user_with_session(&ctx, UserRole::Normal);
    let (_, other_token) = insert_user_with_session(&ctx, UserRole::Normal);
    let extra_session = insert_session(&ctx, user.id, "extra-session");
    let router = test_router(&ctx);

    let res = router.handle(
        &Request::get("http://localhost/api/sessions").header("Authorization", auth(&token)),
    );
    assert_eq!(res.status, Status::Ok);
    let sessions: api::SessionIndexResponse = res.into_json().expect("sessions json");
    assert_eq!(sessions.pagination.total, 2);

    let res = router.handle(
        &Request::delete(format!(
            "http://localhost/api/sessions/{}",
            extra_session.id
        ))
        .header("Authorization", auth(&other_token)),
    );
    assert_eq!(res.status, Status::Forbidden);

    let res = router.handle(
        &Request::delete(format!(
            "http://localhost/api/sessions/{}",
            extra_session.id
        ))
        .header("Authorization", auth(&token)),
    );
    assert_eq!(res.status, Status::Ok);
}

#[test]
fn deployments_show_respects_project_access() {
    let ctx = Context::with_test_database().expect("test database");
    let (owner, owner_token) = insert_user_with_session(&ctx, UserRole::Normal);
    let (_, outsider_token) = insert_user_with_session(&ctx, UserRole::Normal);
    let team = insert_team_with_member(&ctx, owner.id, TeamUserRole::Owner);
    let project = insert_project(&ctx, team.id);
    let deployment = insert_deployment(&ctx, project.id);
    let router = test_router(&ctx);

    let res = router.handle(
        &Request::get(format!(
            "http://localhost/api/deployments/{}",
            deployment.id
        ))
        .header("Authorization", auth(&owner_token)),
    );
    assert_eq!(res.status, Status::Ok);

    let res = router.handle(
        &Request::get(format!(
            "http://localhost/api/deployments/{}",
            deployment.id
        ))
        .header("Authorization", auth(&outsider_token)),
    );
    assert_eq!(res.status, Status::Forbidden);
}

#[test]
fn webhook_rejects_missing_or_invalid_signature() {
    let ctx = Context::with_test_database().expect("test database");
    let user = insert_user(&ctx, UserRole::Normal);
    let team = insert_team_with_member(&ctx, user.id, TeamUserRole::Owner);
    insert_github_token(&ctx, team.id, "secret");
    let router = test_router(&ctx);
    insert_project(&ctx, team.id);
    let payload = r#"{"repository":{"full_name":"owner/repo"}}"#;

    let res = router.handle(&Request::post("http://localhost/api/webhook/github").body(payload));
    assert_eq!(res.status, Status::Unauthorized);

    let res = router.handle(
        &Request::post("http://localhost/api/webhook/github")
            .header("X-Hub-Signature-256", "sha256=invalid")
            .body(payload),
    );
    assert_eq!(res.status, Status::Unauthorized);
}
