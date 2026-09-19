/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::{Context as _, Result};
use base64::prelude::*;
use hmac::verify;
use serde::Deserialize;
use sha2::Sha256;
use small_http::{Request, Response, Status};
use uuid::Uuid;

use crate::api;
use crate::context::{Context, DatabaseHelpers};
use crate::controllers::require_auth;
use crate::deploy::DeployTask;
use crate::models::{Deployment, DeploymentMode, Project};

pub(crate) fn github_repositories(_: &Request, ctx: &Context) -> Result<Response> {
    let user = require_auth!(ctx);
    let Some(token) = &user.github_token else {
        return Ok(Response::with_status(Status::PreconditionFailed));
    };
    match crate::github::repositories(token) {
        Ok(repositories) => Ok(Response::with_json(repositories)),
        Err(error) => {
            log::warn!("Could not list GitHub repositories: {error}");
            Ok(Response::with_status(Status::BadGateway))
        }
    }
}

pub(crate) fn github_repository_scan(req: &Request, ctx: &Context) -> Result<Response> {
    let user = require_auth!(ctx);
    let Some(token) = &user.github_token else {
        return Ok(Response::with_status(Status::PreconditionFailed));
    };
    let body = match req.parse_body::<api::GitHubRepositoryScanBody>() {
        Ok(body) => body,
        Err(error) => return Ok(Response::with_status(error)),
    };
    if !is_valid_github_repo(&body.github_repo) {
        return Ok(Response::with_status(Status::BadRequest));
    }
    let repository = match crate::github::repository(token, &body.github_repo) {
        Ok(repository) => repository,
        Err(error) => {
            log::warn!("Could not read GitHub repository: {error}");
            return Ok(Response::with_status(Status::BadGateway));
        }
    };
    let dockerfile_path = match crate::github::find_dockerfile(
        token,
        &repository.full_name,
        &repository.default_branch,
    ) {
        Ok(path) => path,
        Err(error) => {
            log::warn!("Could not scan GitHub repository: {error}");
            return Ok(Response::with_status(Status::BadGateway));
        }
    };
    let deployment_mode = if dockerfile_path.is_some() {
        api::DeploymentMode::Dockerfile
    } else {
        api::DeploymentMode::Static
    };
    Ok(Response::with_json(api::GitHubRepositoryScan {
        deployment_mode,
        dockerfile_path,
    }))
}

pub(crate) fn projects_index(_: &Request, ctx: &Context) -> Result<Response> {
    let user = require_auth!(ctx);
    let projects = ctx
        .database
        .query::<Project>(
            format!(
                "SELECT {} FROM projects WHERE user_id = ? ORDER BY created_at DESC",
                Project::columns()
            ),
            user.id,
        )?
        .map(|row| row.map(|project| project.into_api(&ctx.deployments_domain)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Response::with_json(projects))
}

pub(crate) fn projects_create(req: &Request, ctx: &Context) -> Result<Response> {
    let user = require_auth!(ctx);
    let body = match req.parse_body::<api::CreateProjectBody>() {
        Ok(body) => body,
        Err(error) => return Ok(Response::with_status(error)),
    };
    let Some(token) = &user.github_token else {
        return Ok(Response::with_status(Status::PreconditionFailed));
    };
    if !is_valid_github_repo(&body.github_repo) {
        return Ok(Response::with_status(Status::BadRequest));
    }
    let deployment_mode = match body.deployment_mode {
        api::DeploymentMode::Static => DeploymentMode::Static,
        api::DeploymentMode::Dockerfile => DeploymentMode::Dockerfile,
    };
    let dockerfile_path = if deployment_mode == DeploymentMode::Dockerfile {
        let Some(path) = body
            .dockerfile_path
            .as_deref()
            .and_then(normalize_repo_path)
        else {
            return Ok(Response::with_status(Status::BadRequest));
        };
        Some(path)
    } else {
        None
    };
    let repository = match crate::github::repository(token, &body.github_repo) {
        Ok(repository) => repository,
        Err(error) => {
            log::warn!("Could not read GitHub repository: {error}");
            return Ok(Response::with_status(Status::BadGateway));
        }
    };
    if ctx.database.query_some::<i64>(
        "SELECT COUNT(id) FROM projects WHERE user_id = ? AND github_repo = ?",
        (user.id, repository.full_name.clone()),
    )? > 0
    {
        return Ok(Response::with_status(Status::Conflict));
    }

    let id = Uuid::now_v7();
    let slug = unique_slug(ctx, &repository.name)?;
    let secret = webhook_secret()?;
    let callback = format!(
        "{}/api/webhooks/github/{id}",
        ctx.server_origin.trim_end_matches('/')
    );
    let webhook_id =
        match crate::github::create_webhook(token, &repository.full_name, &callback, &secret) {
            Ok(id) => id,
            Err(error) => {
                log::warn!("Could not create GitHub webhook: {error}");
                return Ok(Response::with_status(Status::BadGateway));
            }
        };
    let project = Project {
        id,
        user_id: user.id,
        name: repository.name,
        slug,
        github_repo: repository.full_name,
        github_branch: repository.default_branch,
        github_webhook_id: Some(webhook_id),
        webhook_secret: secret,
        deployment_mode,
        dockerfile_path,
        ..Default::default()
    };
    if let Err(error) = ctx.database.insert_project(project.clone()) {
        _ = crate::github::delete_webhook(token, &project.github_repo, webhook_id);
        return Err(error);
    }
    if let Err(error) =
        queue_deployment(ctx, &project, &project.github_branch, "Initial deployment")
    {
        _ = ctx
            .database
            .execute("DELETE FROM projects WHERE id = ?", project.id);
        _ = crate::github::delete_webhook(token, &project.github_repo, webhook_id);
        return Err(error);
    }
    Ok(Response::new()
        .status(Status::Created)
        .json(project.into_api(&ctx.deployments_domain)))
}

pub(crate) fn projects_show(req: &Request, ctx: &Context) -> Result<Response> {
    let user = require_auth!(ctx);
    let Some(project) = owned_project(req, ctx, user.id)? else {
        return Ok(Response::with_status(Status::NotFound));
    };
    Ok(Response::with_json(
        project.into_api(&ctx.deployments_domain),
    ))
}

pub(crate) fn projects_delete(req: &Request, ctx: &Context) -> Result<Response> {
    let user = require_auth!(ctx);
    let Some(project) = owned_project(req, ctx, user.id)? else {
        return Ok(Response::with_status(Status::NotFound));
    };
    if let (Some(token), Some(hook_id)) = (&user.github_token, project.github_webhook_id)
        && let Err(error) = crate::github::delete_webhook(token, &project.github_repo, hook_id)
    {
        log::warn!("Could not delete GitHub webhook: {error}");
    }
    crate::deploy::cleanup(ctx, &project)?;
    ctx.database
        .execute("DELETE FROM projects WHERE id = ?", project.id)?;
    Ok(Response::with_status(Status::NoContent))
}

pub(crate) fn project_deployments(req: &Request, ctx: &Context) -> Result<Response> {
    let user = require_auth!(ctx);
    let Some(project) = owned_project(req, ctx, user.id)? else {
        return Ok(Response::with_status(Status::NotFound));
    };
    let deployments = ctx
        .database
        .query::<Deployment>(
            format!(
                "SELECT {} FROM deployments WHERE project_id = ? ORDER BY created_at DESC LIMIT 50",
                Deployment::columns()
            ),
            project.id,
        )?
        .map(|row| row.map(api::Deployment::from))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Response::with_json(deployments))
}

pub(crate) fn projects_deploy(req: &Request, ctx: &Context) -> Result<Response> {
    let user = require_auth!(ctx);
    let Some(project) = owned_project(req, ctx, user.id)? else {
        return Ok(Response::with_status(Status::NotFound));
    };
    queue_deployment(ctx, &project, &project.github_branch, "Manual deployment")?;
    Ok(Response::with_status(Status::Accepted))
}

pub(crate) fn github_webhook(req: &Request, ctx: &Context) -> Result<Response> {
    let Some(id) = project_id(req) else {
        return Ok(Response::with_status(Status::BadRequest));
    };
    let Some(project) = ctx
        .database
        .query::<Project>(
            format!(
                "SELECT {} FROM projects WHERE id = ? LIMIT 1",
                Project::columns()
            ),
            id,
        )?
        .next()
        .transpose()?
    else {
        return Ok(Response::with_status(Status::NotFound));
    };
    let body = req.body.as_deref().unwrap_or_default();
    let signature = req.headers.get("X-Hub-Signature-256").unwrap_or_default();
    if !verify_signature(&project.webhook_secret, body, signature) {
        return Ok(Response::with_status(Status::Unauthorized));
    }
    if req.headers.get("X-GitHub-Event") != Some("push") {
        return Ok(Response::with_status(Status::NoContent));
    }
    let payload = match serde_json::from_slice::<PushPayload>(body) {
        Ok(payload) => payload,
        Err(_) => return Ok(Response::with_status(Status::BadRequest)),
    };
    if payload.git_ref != format!("refs/heads/{}", project.github_branch) {
        return Ok(Response::with_status(Status::NoContent));
    }
    if payload.deleted {
        return Ok(Response::with_status(Status::NoContent));
    }
    if ctx.database.query_some::<i64>(
        "SELECT COUNT(id) FROM deployments WHERE project_id = ? AND commit_sha = ? AND status IN (0, 1)",
        (project.id, payload.after.clone()),
    )? == 0 {
        queue_deployment(ctx, &project, &payload.after, &payload.head_commit.map(|commit| commit.message).unwrap_or_default())?;
    }
    Ok(Response::with_status(Status::Accepted))
}

fn owned_project(req: &Request, ctx: &Context, user_id: Uuid) -> Result<Option<Project>> {
    let Some(id) = project_id(req) else {
        return Ok(None);
    };
    ctx.database
        .query::<Project>(
            format!(
                "SELECT {} FROM projects WHERE id = ? AND user_id = ? LIMIT 1",
                Project::columns()
            ),
            (id, user_id),
        )?
        .next()
        .transpose()
        .map_err(Into::into)
}

fn project_id(req: &Request) -> Option<Uuid> {
    req.params.get("project_id").and_then(|id| id.parse().ok())
}

fn queue_deployment(ctx: &Context, project: &Project, git_ref: &str, message: &str) -> Result<()> {
    let deployment = Deployment {
        project_id: project.id,
        commit_sha: git_ref.to_string(),
        commit_message: message.to_string(),
        ..Default::default()
    };
    let deployment_id = deployment.id;
    ctx.database.insert_deployment(deployment)?;
    if let Err(error) = ctx.deploy_tx.send(DeployTask {
        project_id: project.id,
        deployment_id,
    }) {
        let message = "deployment runner stopped";
        ctx.database.execute(
            "UPDATE deployments SET status = ?, log = ?, updated_at = ? WHERE id = ?",
            (
                crate::models::DeploymentStatus::Failed as i64,
                message.to_string(),
                chrono::Utc::now(),
                deployment_id,
            ),
        )?;
        return Err(error).context(message);
    }
    Ok(())
}

fn unique_slug(ctx: &Context, name: &str) -> Result<String> {
    let base = slug_base(name);
    for suffix in 1..=10_000 {
        let slug = if suffix == 1 {
            base.clone()
        } else {
            let suffix = format!("-{suffix}");
            let base = base[..base.len().min(63 - suffix.len())].trim_end_matches('-');
            format!("{base}{suffix}")
        };
        if ctx.database.query_some::<i64>(
            "SELECT COUNT(id) FROM projects WHERE slug = ?",
            slug.clone(),
        )? == 0
        {
            return Ok(slug);
        }
    }
    anyhow::bail!("could not allocate project slug")
}

fn slug_base(name: &str) -> String {
    let mut slug = String::with_capacity(name.len().min(63));
    let mut separator = false;
    for character in name.to_ascii_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            if separator && !slug.is_empty() && slug.len() < 63 {
                slug.push('-');
            }
            separator = false;
            if slug.len() < 63 {
                slug.push(character);
            }
        } else {
            separator = true;
        }
    }
    let slug = slug.trim_end_matches('-');
    if slug.is_empty() {
        "project".to_string()
    } else {
        slug.to_string()
    }
}

fn is_valid_github_repo(value: &str) -> bool {
    let Some((owner, repository)) = value.split_once('/') else {
        return false;
    };
    !owner.is_empty()
        && owner.len() <= 39
        && !owner.starts_with('-')
        && !owner.ends_with('-')
        && owner
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        && !repository.is_empty()
        && repository.len() <= 100
        && !repository.contains('/')
        && repository != "."
        && repository != ".."
        && repository
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn webhook_secret() -> Result<String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes)?;
    Ok(BASE64_URL_SAFE_NO_PAD.encode(bytes))
}

fn normalize_repo_path(value: &str) -> Option<String> {
    use std::path::{Component, Path};

    if value.trim().is_empty() || value.len() > 255 || value.contains('\0') {
        return None;
    }
    let mut parts = Vec::new();
    for component in Path::new(value.trim()).components() {
        match component {
            Component::Normal(part) => parts.push(part.to_str()?.to_string()),
            Component::CurDir => {}
            _ => return None,
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

fn verify_signature(secret: &str, body: &[u8], signature: &str) -> bool {
    let Some(hex) = signature.strip_prefix("sha256=") else {
        return false;
    };
    let Some(expected) = decode_hex(hex) else {
        return false;
    };
    verify::<Sha256>(secret.as_bytes(), body, &expected)
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(text, 16).ok()
        })
        .collect()
}

#[derive(Deserialize)]
struct PushPayload {
    #[serde(rename = "ref")]
    git_ref: String,
    after: String,
    #[serde(default)]
    deleted: bool,
    head_commit: Option<PushCommit>,
}

#[derive(Deserialize)]
struct PushCommit {
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::DatabaseHelpers;
    use crate::models::Deployment;
    use crate::router;
    use crate::test_utils::{authenticated_user, insert_test_project, insert_test_user};

    #[test]
    fn verifies_github_signature() {
        assert!(verify_signature(
            "secret",
            b"payload",
            "sha256=b82fcb791acec57859b989b430a826488ce2e479fdf92326bd0a2e8375a42ba4",
        ));
        assert!(!verify_signature(
            "secret",
            b"changed",
            "sha256=b82fcb791acec57859b989b430a826488ce2e479fdf92326bd0a2e8375a42ba4"
        ));
        assert!(!verify_signature("secret", b"payload", "missing-prefix"));
        assert!(!verify_signature("secret", b"payload", "sha256=xyz"));
    }

    #[test]
    fn normalizes_safe_dockerfile_paths() {
        assert_eq!(
            normalize_repo_path("./apps/web/Dockerfile").as_deref(),
            Some("apps/web/Dockerfile")
        );
        assert_eq!(normalize_repo_path("../Dockerfile"), None);
        assert_eq!(normalize_repo_path("/Dockerfile"), None);
        assert_eq!(normalize_repo_path("."), None);
        assert_eq!(
            normalize_repo_path("app\\Dockerfile").as_deref(),
            Some("app\\Dockerfile")
        );
        assert_eq!(
            normalize_repo_path("Docker,file").as_deref(),
            Some("Docker,file")
        );
        assert_eq!(normalize_repo_path("Dockerfile\0"), None);
    }

    #[test]
    fn creates_dns_safe_slug_bases() {
        assert_eq!(slug_base("Plaat World"), "plaat-world");
        assert_eq!(
            slug_base("  Multiple---Separators  "),
            "multiple-separators"
        );
        assert_eq!(slug_base("!@#$"), "project");
        assert_eq!(slug_base("Already-Safe"), "already-safe");
        assert_eq!(slug_base(&"A".repeat(100)).len(), 63);
    }

    #[test]
    fn validates_github_repository_names() {
        for repository in ["bplaat/crates", "owner/repo.name", "owner/repo_name"] {
            assert!(is_valid_github_repo(repository), "{repository}");
        }
        for repository in [
            "repo",
            "/repo",
            "owner/",
            "owner/repo/extra",
            "owner name/repo",
            "owner/repo?ref=main",
        ] {
            assert!(!is_valid_github_repo(repository), "{repository}");
        }
    }

    #[test]
    fn recognizes_deleted_branch_pushes() {
        let payload = serde_json::from_str::<PushPayload>(
            r#"{"ref":"refs/heads/main","after":"0000000000000000000000000000000000000000","deleted":true,"head_commit":null}"#,
        )
        .expect("parse push payload");

        assert!(payload.deleted);
    }

    #[test]
    fn lists_and_reads_owned_projects() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (user, token) = authenticated_user(&ctx);
        let project = insert_test_project(&ctx, user.id);
        let other = insert_test_user(&ctx);
        insert_test_project(&ctx, other.id);

        let list = router.handle(
            &Request::get("http://localhost/api/projects")
                .header("Authorization", format!("Bearer {token}")),
        );
        let projects =
            serde_json::from_slice::<Vec<api::Project>>(&list.body).expect("projects response");
        let show = router.handle(
            &Request::get(format!("http://localhost/api/projects/{}", project.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        let shown = serde_json::from_slice::<api::Project>(&show.body).expect("project response");

        assert_eq!(list.status, Status::Ok);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].id, project.id);
        assert_eq!(
            projects[0].deployment_url,
            format!("https://{}.apps.example.com", project.slug)
        );
        assert_eq!(show.status, Status::Ok);
        assert_eq!(shown.id, project.id);
    }

    #[test]
    fn hides_projects_owned_by_other_users() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (_, token) = authenticated_user(&ctx);
        let other = insert_test_user(&ctx);
        let project = insert_test_project(&ctx, other.id);

        let response = router.handle(
            &Request::get(format!("http://localhost/api/projects/{}", project.id))
                .header("Authorization", format!("Bearer {token}")),
        );

        assert_eq!(response.status, Status::NotFound);
    }

    #[test]
    fn lists_project_deployments() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (user, token) = authenticated_user(&ctx);
        let project = insert_test_project(&ctx, user.id);
        ctx.database
            .insert_deployment(Deployment {
                project_id: project.id,
                commit_sha: "abc123".to_string(),
                commit_message: "Initial deployment".to_string(),
                ..Default::default()
            })
            .expect("insert deployment");

        let response = router.handle(
            &Request::get(format!(
                "http://localhost/api/projects/{}/deployments",
                project.id
            ))
            .header("Authorization", format!("Bearer {token}")),
        );
        let deployments = serde_json::from_slice::<Vec<api::Deployment>>(&response.body)
            .expect("deployments response");

        assert_eq!(response.status, Status::Ok);
        assert_eq!(deployments.len(), 1);
        assert_eq!(deployments[0].commit_sha, "abc123");
    }

    #[test]
    fn marks_a_deployment_failed_when_the_runner_has_stopped() {
        let ctx = Context::with_test_database().expect("create test database");
        let user = insert_test_user(&ctx);
        let project = insert_test_project(&ctx, user.id);

        assert!(queue_deployment(&ctx, &project, "main", "Manual deployment").is_err());
        let (status, log) = ctx
            .database
            .query_some::<(crate::models::DeploymentStatus, String)>(
                "SELECT status, log FROM deployments WHERE project_id = ?",
                project.id,
            )
            .expect("query deployment");

        assert_eq!(status, crate::models::DeploymentStatus::Failed);
        assert_eq!(log, "deployment runner stopped");
    }

    #[test]
    fn github_project_endpoints_require_a_connected_token() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx.clone());
        let (_, token) = authenticated_user(&ctx);
        let authorization = format!("Bearer {token}");

        let repositories = router.handle(
            &Request::get("http://localhost/api/github/repositories")
                .header("Authorization", authorization.clone()),
        );
        let scan = router.handle(
            &Request::post("http://localhost/api/github/repositories/scan")
                .header("Authorization", authorization.clone())
                .header("Content-Type", "application/json")
                .body(r#"{"githubRepo":"example/project"}"#),
        );
        let create = router.handle(
            &Request::post("http://localhost/api/projects")
                .header("Authorization", authorization)
                .header("Content-Type", "application/json")
                .body(r#"{"githubRepo":"example/project","deploymentMode":"static"}"#),
        );

        assert_eq!(repositories.status, Status::PreconditionFailed);
        assert_eq!(scan.status, Status::PreconditionFailed);
        assert_eq!(create.status, Status::PreconditionFailed);
    }
}
