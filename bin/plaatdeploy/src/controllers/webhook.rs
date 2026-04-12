/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! GitHub App webhook handler

use anyhow::Result;
use const_format::formatcp;
use hmac::hmac;
use serde::Deserialize;
use sha2::Sha256;
use small_http::{Request, Response, Status};
use subtle::ConstantTimeEq as _;

use crate::context::{Context, DatabaseHelpers};
use crate::deploy::DeployTask;
use crate::github;
use crate::models::{Deployment, DeploymentStatus, Project};

// MARK: Event types
#[derive(Deserialize)]
struct PushEvent {
    #[serde(rename = "ref")]
    git_ref: String,
    after: String,
    head_commit: Option<PushCommit>,
    installation: Installation,
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
    installation: Installation,
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
struct Installation {
    id: i64,
}

#[derive(Deserialize)]
struct InstallationOnlyEvent {
    installation: Installation,
}

// MARK: Handler
pub(crate) fn webhook_github(req: &Request, ctx: &Context) -> Result<Response> {
    // Verify HMAC-SHA256 signature
    let body = req.body.as_deref().unwrap_or(&[]);
    let installation_id = match serde_json::from_slice::<InstallationOnlyEvent>(body) {
        Ok(event) => event.installation.id,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };
    let connection = match ctx
        .database
        .find_team_github_connection_by_installation_id(installation_id)?
    {
        Some(connection) => connection,
        None => return Ok(Response::new().status(Status::Unauthorized)),
    };
    let webhook_secret = connection.webhook_secret.unwrap_or_default();
    if !webhook_secret.is_empty() {
        let sig_header = match req
            .headers
            .get("X-Hub-Signature-256")
            .or(req.headers.get("x-hub-signature-256"))
        {
            Some(h) => h,
            None => return Ok(Response::new().status(Status::Unauthorized)),
        };
        let sig_hex = match sig_header.strip_prefix("sha256=") {
            Some(h) => h,
            None => return Ok(Response::new().status(Status::Unauthorized)),
        };
        let expected = hmac::<Sha256>(webhook_secret.as_bytes(), body);
        let expected_hex = expected
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        if !bool::from(sig_hex.as_bytes().ct_eq(expected_hex.as_bytes())) {
            return Ok(Response::new().status(Status::Unauthorized));
        }
    }

    let event_type = match req
        .headers
        .get("X-GitHub-Event")
        .or(req.headers.get("x-github-event"))
    {
        Some(e) => e.to_string(),
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    match event_type.as_str() {
        "push" => handle_push(ctx, body),
        "workflow_run" => handle_workflow_run(ctx, body),
        _ => Ok(Response::new()),
    }
}

fn handle_push(ctx: &Context, body: &[u8]) -> Result<Response> {
    let event: PushEvent = match serde_json::from_slice(body) {
        Ok(e) => e,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };

    // Ignore branch deletions (after is all zeros)
    if event.after.chars().all(|c| c == '0') {
        return Ok(Response::new());
    }

    let branch = match event.git_ref.strip_prefix("refs/heads/") {
        Some(b) => b.to_string(),
        None => return Ok(Response::new()),
    };

    let head_commit = match event.head_commit {
        Some(c) => c,
        None => return Ok(Response::new()),
    };

    let commit_sha = head_commit.id;
    let commit_message = head_commit.message;
    let repo_full_name = event.repository.full_name;

    // If this commit already has queued/in-progress CI workflow runs, skip push-based deploy —
    // the workflow_run event will handle it once CI passes.
    if github::commit_has_pending_runs(ctx, event.installation.id, &repo_full_name, &commit_sha) {
        log::info!(
            "Skipping push deploy for {repo_full_name}: CI workflow runs pending for {}",
            &commit_sha[..7.min(commit_sha.len())]
        );
        return Ok(Response::new());
    }

    trigger_deploys(
        ctx,
        event.installation.id,
        &repo_full_name,
        &branch,
        &commit_sha,
        &commit_message,
    )
}

fn handle_workflow_run(ctx: &Context, body: &[u8]) -> Result<Response> {
    let event: WorkflowRunEvent = match serde_json::from_slice(body) {
        Ok(e) => e,
        Err(_) => return Ok(Response::new().status(Status::BadRequest)),
    };

    // Only deploy on completed + success
    if event.action != "completed" || event.workflow_run.conclusion.as_deref() != Some("success") {
        return Ok(Response::new());
    }

    trigger_deploys(
        ctx,
        event.installation.id,
        &event.repository.full_name,
        &event.workflow_run.head_branch,
        &event.workflow_run.head_sha,
        &event.workflow_run.head_commit.message,
    )
}

fn trigger_deploys(
    ctx: &Context,
    installation_id: i64,
    repo_full_name: &str,
    branch: &str,
    commit_sha: &str,
    commit_message: &str,
) -> Result<Response> {
    let projects: Vec<Project> = ctx
        .database
        .query::<Project>(
            formatcp!(
                "SELECT {} FROM projects WHERE github_repo = ?",
                Project::columns()
            ),
            repo_full_name.to_string(),
        )?
        .collect::<Result<Vec<_>, _>>()?;

    for project in projects {
        if project.github_branch != branch {
            continue;
        }
        if !github::team_uses_installation(ctx, project.team_id, installation_id)? {
            continue;
        }

        let github_deployment_id =
            github::create_deployment(ctx, installation_id, &project.github_repo, commit_sha);

        let deployment = Deployment {
            project_id: project.id,
            commit_sha: commit_sha.to_string(),
            commit_message: commit_message.lines().next().unwrap_or("").to_string(),
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

        log::info!(
            "Queued deploy for project '{}' on commit {}",
            project.name,
            &commit_sha[..7.min(commit_sha.len())]
        );
    }

    Ok(Response::new())
}
