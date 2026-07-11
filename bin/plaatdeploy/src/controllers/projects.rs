/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use chrono::Utc;
use const_format::formatcp;
use from_derive::FromStruct;
use small_http::{Request, Response, Status};
use uuid::Uuid;
use validate::Validate;

use super::{parse_body, parse_pagination};
use crate::context::{Context, DatabaseHelpers};
use crate::deploy::{DeployTask, runner};
use crate::models::{BuildType, Deployment, DeploymentStatus, Project, ProjectStatus, UserRole};
use crate::utils::{normalize_base_dir, validate_project_name};
use crate::{api, github};

fn has_project_access(ctx: &Context, project: &Project) -> Result<bool> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    if auth_user.role == UserRole::Admin {
        return Ok(true);
    }
    ctx.database
        .user_is_team_member(auth_user.id, project.team_id)
}

fn resolve_team_id(ctx: &Context, team_id: Option<Uuid>) -> Result<Option<Uuid>> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    let team_id = match team_id {
        Some(team_id) => team_id,
        None => match ctx.database.find_first_team_by_user_id(auth_user.id)? {
            Some(team) => team.id,
            None => return Ok(None),
        },
    };

    if auth_user.role != UserRole::Admin
        && !ctx.database.user_is_team_member(auth_user.id, team_id)?
    {
        return Ok(None);
    }
    Ok(Some(team_id))
}

fn validate_project_fields(
    name: &str,
    base_dir: &str,
    container_port: Option<i64>,
) -> Result<String, Response> {
    let mut errors = serde_json::Map::new();
    if !validate_project_name(name) {
        errors.insert(
            "name".to_string(),
            serde_json::json!([
                "Use a DNS-safe name: lowercase letters, numbers, hyphens, 1-63 chars, no leading or trailing hyphen"
            ]),
        );
    }
    let base_dir = match normalize_base_dir(base_dir) {
        Some(base_dir) => base_dir,
        None => {
            errors.insert(
                "baseDir".to_string(),
                serde_json::json!(["Use a relative path inside the repository"]),
            );
            String::new()
        }
    };
    if container_port.is_some_and(|port| !(1..=65535).contains(&port)) {
        errors.insert(
            "containerPort".to_string(),
            serde_json::json!(["Use a TCP port between 1 and 65535"]),
        );
    }

    if errors.is_empty() {
        Ok(base_dir)
    } else {
        Err(Response::new()
            .status(Status::BadRequest)
            .json(serde_json::Value::Object(errors)))
    }
}

fn validate_team_github_repo(
    ctx: &Context,
    team_id: Uuid,
    github_repo: &str,
    previous_team_id: Option<Uuid>,
    previous_github_repo: Option<&str>,
) -> Result<Option<Response>> {
    let allow_existing_repo =
        previous_team_id == Some(team_id) && previous_github_repo == Some(github_repo);
    if !github::team_is_connected(ctx, team_id)? {
        if allow_existing_repo {
            return Ok(None);
        }
        return Ok(Some(Response::new().status(Status::BadRequest).json(
            serde_json::json!({
                "githubRepo": ["Connect GitHub for the selected team first"]
            }),
        )));
    }
    let repositories = match github::list_repositories(ctx, team_id) {
        Some(repositories) => repositories,
        None => return Ok(Some(Response::new().status(Status::BadGateway))),
    };
    if repositories
        .iter()
        .any(|repository| repository.full_name == github_repo)
        || allow_existing_repo
    {
        return Ok(None);
    }
    Ok(Some(Response::new().status(Status::BadRequest).json(
        serde_json::json!({
            "githubRepo": ["Repository is not available for the selected team's GitHub connection"]
        }),
    )))
}

pub(crate) fn projects_index(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");

    let q = match parse_pagination(req) {
        Ok(q) => q,
        Err(r) => return Ok(r),
    };
    let page = q.page;
    let limit = q.limit;
    let offset = (page - 1) * limit;

    let (total, projects) = if auth_user.role == UserRole::Admin {
        let total = ctx
            .database
            .query_some::<i64>("SELECT COUNT(id) FROM projects", ())?;
        let projects: Vec<api::Project> = ctx
            .database
            .query::<Project>(
                formatcp!(
                    "SELECT {} FROM projects ORDER BY created_at DESC LIMIT ? OFFSET ?",
                    Project::columns()
                ),
                (limit, offset),
            )?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Into::into)
            .collect();
        (total, projects)
    } else {
        let total = ctx.database.query_some::<i64>(
            "SELECT COUNT(id) FROM projects WHERE team_id IN (SELECT team_id FROM team_users WHERE user_id = ?)",
            auth_user.id,
        )?;
        let projects: Vec<api::Project> = ctx
            .database
            .query::<Project>(
                formatcp!(
                    "SELECT {} FROM projects WHERE team_id IN (SELECT team_id FROM team_users WHERE user_id = ?) ORDER BY created_at DESC LIMIT ? OFFSET ?",
                    Project::columns()
                ),
                (auth_user.id, limit, offset),
            )?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Into::into)
            .collect();
        (total, projects)
    };

    Ok(Response::with_json(api::ProjectIndexResponse {
        pagination: api::Pagination { page, limit, total },
        data: projects,
    }))
}

pub(crate) fn projects_create(req: &Request, ctx: &Context) -> Result<Response> {
    #[derive(Validate, FromStruct)]
    #[from_struct(api::ProjectCreateBody)]
    struct Body {
        #[validate(length(min = 1, max = 100))]
        name: String,
        #[validate(length(min = 1, max = 100))]
        github_repo: String,
        #[validate(length(max = 100))]
        github_branch: Option<String>,
        base_dir: Option<String>,
        team_id: Option<Uuid>,
    }

    let body = parse_body!(req, api::ProjectCreateBody, Body);
    let base_dir =
        match validate_project_fields(&body.name, body.base_dir.as_deref().unwrap_or(""), None) {
            Ok(base_dir) => base_dir,
            Err(response) => return Ok(response),
        };

    let count = ctx.database.query_some::<i64>(
        "SELECT COUNT(id) FROM projects WHERE name = ?",
        body.name.clone(),
    )?;
    if count != 0 {
        return Ok(Response::new()
            .status(Status::BadRequest)
            .json(serde_json::json!({ "name": ["Name is already taken"] })));
    }

    let team_id = match resolve_team_id(ctx, body.team_id)? {
        Some(team_id) => team_id,
        None => return Ok(Response::new().status(Status::Forbidden)),
    };
    if let Some(response) = validate_team_github_repo(ctx, team_id, &body.github_repo, None, None)?
    {
        return Ok(response);
    }
    if !github::ensure_repository_webhook(ctx, team_id, &body.github_repo) {
        return Ok(Response::new().status(Status::BadGateway).json(serde_json::json!({
            "githubRepo": ["Could not create the GitHub webhook. Ensure the token has Webhooks: Read and write."]
        })));
    }

    let github_branch = body.github_branch.unwrap_or_else(|| "master".to_string());
    let commit_sha =
        match github::latest_commit_sha(ctx, team_id, &body.github_repo, &github_branch) {
            Some(commit_sha) => commit_sha,
            None => {
                return Ok(Response::new()
                    .status(Status::BadGateway)
                    .json(serde_json::json!({
                        "githubBranch": ["Could not resolve the branch's latest commit from GitHub"]
                    })));
            }
        };

    let project = Project {
        team_id,
        name: body.name,
        github_repo: body.github_repo,
        github_branch,
        base_dir,
        ..Default::default()
    };
    ctx.database.insert_project(project.clone())?;

    let github_deployment_id =
        github::create_deployment(ctx, project.team_id, &project.github_repo, &commit_sha);
    let deployment = Deployment {
        project_id: project.id,
        commit_sha: commit_sha.clone(),
        commit_message: "Initial deployment".to_string(),
        github_deployment_id: github_deployment_id.map(|id| id as i64),
        status: DeploymentStatus::Pending,
        ..Default::default()
    };
    ctx.database.insert_deployment(deployment.clone())?;

    if let Some(github_deployment_id) = github_deployment_id {
        let log_url = ctx.deployment_log_url(deployment.id);
        github::update_deployment_status(
            ctx,
            project.team_id,
            &project.github_repo,
            github_deployment_id,
            "queued",
            "Initial deployment queued by PlaatDeploy",
            None,
            Some(&log_url),
        );
    }
    ctx.deploy_tx
        .send(DeployTask {
            deployment_id: deployment.id,
            project_id: project.id,
            github_deployment_id,
        })
        .ok();

    Ok(Response::with_json(api::Project::from(project)))
}

pub(crate) fn projects_show(req: &Request, ctx: &Context) -> Result<Response> {
    let project_id = match req.params.get("project_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let project = match ctx.database.find_project_by_id(project_id)? {
        Some(p) => p,
        None => return Ok(Response::new().status(Status::NotFound)),
    };

    if !has_project_access(ctx, &project)? {
        return Ok(Response::new().status(Status::Forbidden));
    }

    Ok(Response::with_json(api::Project::from(project)))
}

pub(crate) fn projects_update(req: &Request, ctx: &Context) -> Result<Response> {
    let project_id = match req.params.get("project_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let mut project = match ctx.database.find_project_by_id(project_id)? {
        Some(p) => p,
        None => return Ok(Response::new().status(Status::NotFound)),
    };

    if !has_project_access(ctx, &project)? {
        return Ok(Response::new().status(Status::Forbidden));
    }

    #[derive(Validate, FromStruct)]
    #[from_struct(api::ProjectUpdateBody)]
    struct Body {
        #[validate(length(min = 1, max = 100))]
        name: String,
        #[validate(length(min = 1, max = 100))]
        github_repo: String,
        #[validate(length(max = 100))]
        github_branch: Option<String>,
        base_dir: Option<String>,
        team_id: Option<Uuid>,
        container_port: Option<i64>,
    }

    let body = parse_body!(req, api::ProjectUpdateBody, Body);
    let base_dir = match validate_project_fields(
        &body.name,
        body.base_dir.as_deref().unwrap_or(&project.base_dir),
        body.container_port,
    ) {
        Ok(base_dir) => base_dir,
        Err(response) => return Ok(response),
    };
    let count = ctx.database.query_some::<i64>(
        "SELECT COUNT(id) FROM projects WHERE name = ? AND id != ?",
        (body.name.clone(), project.id),
    )?;
    if count != 0 {
        return Ok(Response::new()
            .status(Status::BadRequest)
            .json(serde_json::json!({ "name": ["Name is already taken"] })));
    }

    let team_id = match resolve_team_id(ctx, body.team_id)? {
        Some(team_id) => team_id,
        None => return Ok(Response::new().status(Status::Forbidden)),
    };
    if let Some(response) = validate_team_github_repo(
        ctx,
        team_id,
        &body.github_repo,
        Some(project.team_id),
        Some(&project.github_repo),
    )? {
        return Ok(response);
    }
    if (team_id != project.team_id || body.github_repo != project.github_repo)
        && !github::ensure_repository_webhook(ctx, team_id, &body.github_repo)
    {
        return Ok(Response::new().status(Status::BadGateway).json(serde_json::json!({
            "githubRepo": ["Could not create the GitHub webhook. Ensure the token has Webhooks: Read and write."]
        })));
    }

    let old_name = project.name.clone();
    project.name = body.name;
    project.team_id = team_id;
    project.github_repo = body.github_repo;
    if let Some(branch) = body.github_branch {
        project.github_branch = branch;
    }
    project.base_dir = base_dir;
    project.container_port = body.container_port;
    project.updated_at = Utc::now();
    if old_name != project.name {
        runner::cleanup_project_artifacts(&ctx.deploy_path, &old_name)?;
        project.build_type = BuildType::Unknown;
        project.status = ProjectStatus::Idle;
        project.container_ip = None;
        project.last_deployed_at = None;
    }

    ctx.database.execute(
        "UPDATE projects SET team_id = ?, name = ?, github_repo = ?, github_branch = ?, base_dir = ?, container_port = ?, container_ip = ?, build_type = ?, status = ?, last_deployed_at = ?, updated_at = ? WHERE id = ?",
        (
            project.team_id,
            project.name.clone(),
            project.github_repo.clone(),
            project.github_branch.clone(),
            project.base_dir.clone(),
            project.container_port,
            project.container_ip.clone(),
            project.build_type as i64,
            project.status as i64,
            project.last_deployed_at,
            project.updated_at,
            project.id,
        ),
    )?;

    Ok(Response::with_json(api::Project::from(project)))
}

pub(crate) fn projects_delete(req: &Request, ctx: &Context) -> Result<Response> {
    let project_id = match req.params.get("project_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let project = match ctx.database.find_project_by_id(project_id)? {
        Some(p) => p,
        None => return Ok(Response::new().status(Status::NotFound)),
    };

    if !has_project_access(ctx, &project)? {
        return Ok(Response::new().status(Status::Forbidden));
    }

    if github::team_is_connected(ctx, project.team_id)? {
        let deployments = ctx
            .database
            .query::<Deployment>(
                formatcp!(
                    "SELECT {} FROM deployments WHERE project_id = ? AND github_deployment_id IS NOT NULL",
                    Deployment::columns()
                ),
                project.id,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        for deployment in deployments
            .into_iter()
            .filter(|deployment| deployment.status == DeploymentStatus::Succeeded)
        {
            let Some(github_deployment_id) = deployment.github_deployment_id else {
                continue;
            };
            let log_url = ctx.deployment_log_url(deployment.id);
            if !github::deactivate_deployment(
                ctx,
                project.team_id,
                &project.github_repo,
                github_deployment_id,
                &log_url,
            ) {
                return Ok(Response::new().status(Status::BadGateway).json(serde_json::json!({
                    "githubRepo": ["Could not mark the GitHub deployment inactive. Check the token's Deployments permission and try again."]
                })));
            }
        }
    }

    let repository_project_count = ctx.database.query_some::<i64>(
        "SELECT COUNT(id) FROM projects WHERE team_id = ? AND github_repo = ?",
        (project.team_id, project.github_repo.clone()),
    )?;
    if repository_project_count == 1
        && github::team_is_connected(ctx, project.team_id)?
        && !github::remove_repository_webhook(ctx, project.team_id, &project.github_repo)
    {
        return Ok(Response::new().status(Status::BadGateway).json(serde_json::json!({
            "githubRepo": ["Could not remove the GitHub webhook. Check the token's Webhooks permission and try again."]
        })));
    }

    runner::cleanup_project_artifacts(&ctx.deploy_path, &project.name)?;

    ctx.database
        .execute("DELETE FROM deployments WHERE project_id = ?", project.id)?;
    ctx.database
        .execute("DELETE FROM projects WHERE id = ?", project.id)?;
    Ok(Response::new())
}

pub(crate) fn projects_deploy(req: &Request, ctx: &Context) -> Result<Response> {
    let project_id = match req.params.get("project_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let project = match ctx.database.find_project_by_id(project_id)? {
        Some(p) => p,
        None => return Ok(Response::new().status(Status::NotFound)),
    };

    if !has_project_access(ctx, &project)? {
        return Ok(Response::new().status(Status::Forbidden));
    }

    let github_deployment_id = github::create_deployment(
        ctx,
        project.team_id,
        &project.github_repo,
        &project.github_branch,
    );

    let deployment = Deployment {
        project_id: project.id,
        commit_sha: "manual".to_string(),
        commit_message: "Manual deploy".to_string(),
        github_deployment_id: github_deployment_id.map(|id| id as i64),
        status: DeploymentStatus::Pending,
        ..Default::default()
    };
    ctx.database.insert_deployment(deployment.clone())?;

    if let Some(github_deployment_id) = github_deployment_id {
        let log_url = ctx.deployment_log_url(deployment.id);
        github::update_deployment_status(
            ctx,
            project.team_id,
            &project.github_repo,
            github_deployment_id,
            "queued",
            "Queued by PlaatDeploy",
            None,
            Some(&log_url),
        );
    }

    ctx.deploy_tx
        .send(DeployTask {
            deployment_id: deployment.id,
            project_id: project.id,
            github_deployment_id,
        })
        .ok();

    Ok(Response::with_json(api::Deployment::from(deployment)))
}

pub(crate) fn projects_deployments(req: &Request, ctx: &Context) -> Result<Response> {
    let project_id = match req.params.get("project_id").and_then(|id| id.parse().ok()) {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let project = match ctx.database.find_project_by_id(project_id)? {
        Some(p) => p,
        None => return Ok(Response::new().status(Status::NotFound)),
    };

    if !has_project_access(ctx, &project)? {
        return Ok(Response::new().status(Status::Forbidden));
    }

    let q = match parse_pagination(req) {
        Ok(q) => q,
        Err(r) => return Ok(r),
    };
    let page = q.page;
    let limit = q.limit;
    let offset = (page - 1) * limit;

    let total = ctx.database.query_some::<i64>(
        "SELECT COUNT(id) FROM deployments WHERE project_id = ?",
        project_id,
    )?;
    let deployments: Vec<api::Deployment> = ctx
        .database
        .query::<Deployment>(
            formatcp!(
                "SELECT {} FROM deployments WHERE project_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
                Deployment::columns()
            ),
            (project_id, limit, offset),
        )?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Response::with_json(api::DeploymentIndexResponse {
        pagination: api::Pagination { page, limit, total },
        data: deployments,
    }))
}
