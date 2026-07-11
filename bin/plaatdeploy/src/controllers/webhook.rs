/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! GitHub repository webhook handler for fine-grained PAT connections.

use anyhow::Result;
use const_format::formatcp;
use hmac::hmac;
use serde::Deserialize;
use sha2::Sha256;
use small_http::{Request, Response, Status};
use subtle::ConstantTimeEq as _;
use uuid::Uuid;

use crate::context::{Context, DatabaseHelpers};
use crate::deploy::DeployTask;
use crate::github;
use crate::models::{Deployment, DeploymentStatus, Project};

#[derive(Deserialize)]
struct PushEvent {
    #[serde(rename = "ref")]
    git_ref: String,
    after: String,
    head_commit: Option<PushCommit>,
    repository: Repository,
}

#[derive(Deserialize)]
struct PushCommit {
    id: String,
    message: String,
}

#[derive(Deserialize)]
struct WorkflowRunEvent {
    action: String,
    workflow_run: WorkflowRun,
    repository: Repository,
}

#[derive(Deserialize)]
struct WorkflowRun {
    conclusion: Option<String>,
    head_branch: String,
    head_sha: String,
    head_commit: HeadCommit,
}

#[derive(Deserialize)]
struct HeadCommit {
    message: String,
}

#[derive(Deserialize)]
struct Repository {
    full_name: String,
}

#[derive(Deserialize)]
struct RepositoryEvent {
    repository: Repository,
}

pub(crate) fn webhook_github(req: &Request, ctx: &Context) -> Result<Response> {
    let body = req.body.as_deref().unwrap_or(&[]);
    let event = match serde_json::from_slice::<RepositoryEvent>(body) {
        Ok(event) => event,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };
    let sig_hex = match req
        .headers
        .get("X-Hub-Signature-256")
        .or(req.headers.get("x-hub-signature-256"))
        .and_then(|header| header.strip_prefix("sha256="))
    {
        Some(signature) => signature,
        None => return Ok(Response::new().status(Status::Unauthorized)),
    };
    let requested_team_id = req
        .url
        .query()
        .and_then(|query| serde_urlencoded::from_str::<WebhookQuery>(query).ok())
        .and_then(|query| query.team_id);
    let team_id = match verified_team_for_repository(
        ctx,
        &event.repository.full_name,
        body,
        sig_hex,
        requested_team_id,
    )? {
        Some(team_id) => team_id,
        None => return Ok(Response::new().status(Status::Unauthorized)),
    };
    match req
        .headers
        .get("X-GitHub-Event")
        .or(req.headers.get("x-github-event"))
    {
        Some("push") => handle_push(ctx, body, team_id),
        Some("workflow_run") => handle_workflow_run(ctx, body, team_id),
        Some(_) => Ok(Response::new()),
        None => Ok(Response::new().status(Status::BadRequest)),
    }
}

#[derive(Deserialize)]
struct WebhookQuery {
    team_id: Option<Uuid>,
}

fn verified_team_for_repository(
    ctx: &Context,
    repo: &str,
    body: &[u8],
    signature: &str,
    requested_team_id: Option<Uuid>,
) -> Result<Option<Uuid>> {
    let projects = ctx
        .database
        .query::<Project>(
            formatcp!(
                "SELECT {} FROM projects WHERE github_repo = ?",
                Project::columns()
            ),
            repo.to_string(),
        )?
        .collect::<Result<Vec<_>, _>>()?;
    for project in projects {
        if requested_team_id.is_some_and(|team_id| team_id != project.team_id) {
            continue;
        }
        let Some(connection) = github::team_connection(ctx, project.team_id)? else {
            continue;
        };
        if connection.webhook_secret.is_empty() {
            continue;
        }
        let secret = connection.webhook_secret;
        let expected = hmac::<Sha256>(secret.as_bytes(), body);
        let expected_hex = expected
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        if bool::from(signature.as_bytes().ct_eq(expected_hex.as_bytes())) {
            return Ok(Some(project.team_id));
        }
    }
    Ok(None)
}

fn handle_push(ctx: &Context, body: &[u8], team_id: Uuid) -> Result<Response> {
    let event: PushEvent = match serde_json::from_slice(body) {
        Ok(event) => event,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };
    if event.after.chars().all(|character| character == '0') {
        return Ok(Response::new());
    }
    let Some(branch) = event
        .git_ref
        .strip_prefix("refs/heads/")
        .map(str::to_string)
    else {
        return Ok(Response::new());
    };
    let Some(commit) = event.head_commit else {
        return Ok(Response::new());
    };
    let repo = event.repository.full_name;
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        if github::commit_has_pending_runs(&ctx, team_id, &repo, &commit.id) {
            return;
        }
        if let Err(error) =
            trigger_deploys(&ctx, team_id, &repo, &branch, &commit.id, &commit.message)
        {
            log::error!("Failed to trigger deploys for {repo}: {error}");
        }
    });
    Ok(Response::new())
}

fn handle_workflow_run(ctx: &Context, body: &[u8], team_id: Uuid) -> Result<Response> {
    let event: WorkflowRunEvent = match serde_json::from_slice(body) {
        Ok(event) => event,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };
    if event.action == "completed" && event.workflow_run.conclusion.as_deref() == Some("success") {
        trigger_deploys(
            ctx,
            team_id,
            &event.repository.full_name,
            &event.workflow_run.head_branch,
            &event.workflow_run.head_sha,
            &event.workflow_run.head_commit.message,
        )?;
    }
    Ok(Response::new())
}

fn trigger_deploys(
    ctx: &Context,
    team_id: Uuid,
    repo: &str,
    branch: &str,
    sha: &str,
    message: &str,
) -> Result<()> {
    let projects = ctx
        .database
        .query::<Project>(
            formatcp!(
                "SELECT {} FROM projects WHERE team_id = ? AND github_repo = ?",
                Project::columns()
            ),
            (team_id, repo.to_string()),
        )?
        .collect::<Result<Vec<_>, _>>()?;
    for project in projects
        .into_iter()
        .filter(|project| project.github_branch == branch)
    {
        let github_deployment_id =
            github::create_deployment(ctx, team_id, &project.github_repo, sha);
        let deployment = Deployment {
            project_id: project.id,
            commit_sha: sha.to_string(),
            commit_message: message.lines().next().unwrap_or("").to_string(),
            github_deployment_id: github_deployment_id.map(|id| id as i64),
            status: DeploymentStatus::Pending,
            ..Default::default()
        };
        ctx.database.insert_deployment(deployment.clone())?;

        if let Some(github_deployment_id) = github_deployment_id {
            let log_url = ctx.deployment_log_url(deployment.id);
            github::update_deployment_status(
                ctx,
                team_id,
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
    }
    Ok(())
}
