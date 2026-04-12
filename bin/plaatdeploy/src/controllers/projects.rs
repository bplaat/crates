/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use chrono::Utc;
use const_format::formatcp;
use serde::Deserialize;
use small_http::{Request, Response, Status};
use uuid::Uuid;

use super::parse_pagination;
use crate::context::{Context, DatabaseHelpers};
use crate::deploy::DeployTask;
use crate::models::{Deployment, DeploymentStatus, Project, UserRole};
use crate::{api, github};

fn has_project_access(ctx: &Context, project: &Project) -> Result<bool> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    if auth_user.role == UserRole::Admin {
        return Ok(true);
    }
    ctx.database
        .user_is_team_member(auth_user.id, project.team_id)
}

fn resolve_team_id(ctx: &Context, team_id: Option<&str>) -> Result<Option<Uuid>> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    let team_id = match team_id {
        Some(team_id) => match team_id.parse() {
            Ok(team_id) => team_id,
            Err(_) => return Ok(None),
        },
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

fn validate_team_github_repo(
    ctx: &Context,
    team_id: Uuid,
    github_repo: &str,
    previous_team_id: Option<Uuid>,
    previous_github_repo: Option<&str>,
) -> Result<Option<Response>> {
    let allow_existing_repo =
        previous_team_id == Some(team_id) && previous_github_repo == Some(github_repo);
    let installation = match github::team_installation(ctx, team_id)? {
        Some(installation) => installation,
        None => {
            if allow_existing_repo {
                return Ok(None);
            }
            return Ok(Some(Response::new().status(Status::BadRequest).json(
                serde_json::json!({
                    "githubRepo": ["Connect GitHub for the selected team first"]
                }),
            )));
        }
    };
    let repositories = match github::list_repositories(ctx, installation.id) {
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
    #[derive(Deserialize)]
    struct Body {
        name: String,
        github_repo: String,
        github_branch: Option<String>,
        base_dir: Option<String>,
        team_id: Option<String>,
    }

    let body = match serde_urlencoded::from_bytes::<Body>(req.body.as_deref().unwrap_or(&[])) {
        Ok(b) => b,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };
    if body.name.is_empty() || body.github_repo.is_empty() {
        return Ok(Response::new().status(Status::BadRequest));
    }

    let count = ctx.database.query_some::<i64>(
        "SELECT COUNT(id) FROM projects WHERE name = ?",
        body.name.clone(),
    )?;
    if count != 0 {
        return Ok(Response::new()
            .status(Status::BadRequest)
            .json(serde_json::json!({ "name": ["Name is already taken"] })));
    }

    let team_id = match resolve_team_id(ctx, body.team_id.as_deref())? {
        Some(team_id) => team_id,
        None => return Ok(Response::new().status(Status::Forbidden)),
    };
    if let Some(response) = validate_team_github_repo(ctx, team_id, &body.github_repo, None, None)?
    {
        return Ok(response);
    }

    let project = Project {
        team_id,
        name: body.name,
        github_repo: body.github_repo,
        github_branch: body.github_branch.unwrap_or_else(|| "master".to_string()),
        base_dir: body.base_dir.unwrap_or_default(),
        ..Default::default()
    };
    ctx.database.insert_project(project.clone())?;

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

    #[derive(Deserialize)]
    struct Body {
        name: String,
        github_repo: String,
        github_branch: Option<String>,
        base_dir: Option<String>,
        container_port: Option<i64>,
        team_id: Option<String>,
    }

    let body = match serde_urlencoded::from_bytes::<Body>(req.body.as_deref().unwrap_or(&[])) {
        Ok(b) => b,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };

    let team_id = match resolve_team_id(ctx, body.team_id.as_deref())? {
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

    project.name = body.name;
    project.team_id = team_id;
    project.github_repo = body.github_repo;
    if let Some(branch) = body.github_branch {
        project.github_branch = branch;
    }
    if let Some(dir) = body.base_dir {
        project.base_dir = dir;
    }
    project.container_port = body.container_port;
    project.updated_at = Utc::now();

    ctx.database.execute(
        "UPDATE projects SET team_id = ?, name = ?, github_repo = ?, github_branch = ?, base_dir = ?, container_port = ?, updated_at = ? WHERE id = ?",
        (
            project.team_id,
            project.name.clone(),
            project.github_repo.clone(),
            project.github_branch.clone(),
            project.base_dir.clone(),
            project.container_port,
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

    let github_deployment_id =
        github::team_installation(ctx, project.team_id)?.and_then(|installation| {
            github::create_deployment(
                ctx,
                installation.id,
                &project.github_repo,
                &project.github_branch,
            )
        });

    let deployment = Deployment {
        project_id: project.id,
        commit_sha: "manual".to_string(),
        commit_message: "Manual deploy".to_string(),
        status: DeploymentStatus::Pending,
        ..Default::default()
    };
    ctx.database.insert_deployment(deployment.clone())?;

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
