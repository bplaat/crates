/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! GitHub API helpers via fine-grained personal access tokens.

use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use small_http::{Request, Status};
use uuid::Uuid;

use crate::context::{Context, DatabaseHelpers};
use crate::models::TeamGitHubToken;

#[derive(Clone)]
pub(crate) struct Repository {
    pub full_name: String,
}

#[derive(Clone)]
pub(crate) struct Branch {
    pub name: String,
}

#[derive(Deserialize)]
struct WebhookQuery {
    team_id: Uuid,
}

fn is_team_webhook(hook: &serde_json::Value, team_id: Uuid) -> bool {
    let Some(url) = hook["config"]["url"].as_str() else {
        return false;
    };
    let Some((path, query)) = url.split_once('?') else {
        return false;
    };
    path.ends_with("/api/webhook/github")
        && serde_urlencoded::from_str::<WebhookQuery>(query)
            .map(|query| query.team_id == team_id)
            .unwrap_or(false)
}

fn is_managed_webhook(ctx: &Context, hook: &serde_json::Value, team_id: Uuid) -> bool {
    if is_team_webhook(hook, team_id) {
        return true;
    }
    let legacy_url = format!(
        "{}/api/webhook/github",
        ctx.server_origin.trim_end_matches('/')
    );
    hook["config"]["url"].as_str() == Some(legacy_url.as_str())
}

fn github_request(
    token: &str,
    request: Request,
    method: &str,
    url: &str,
) -> Option<serde_json::Value> {
    let res = request
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "plaatdeploy")
        .fetch()
        .ok()?;
    let val: serde_json::Value = serde_json::from_slice(&res.body).ok()?;
    if let Some(msg) = val["message"].as_str() {
        let details = val["errors"]
            .as_array()
            .filter(|errors| !errors.is_empty())
            .map(|errors| serde_json::Value::Array(errors.clone()).to_string())
            .unwrap_or_default();
        if details.is_empty() {
            log::warn!(
                "GitHub API {method} {url} error (HTTP {}): {msg}",
                res.status as u16
            );
        } else {
            log::warn!(
                "GitHub API {method} {url} error (HTTP {}): {msg}; details: {details}",
                res.status as u16
            );
        }
        return None;
    }
    Some(val)
}

fn github_get(token: &str, url: &str) -> Option<serde_json::Value> {
    github_request(token, Request::get(url), "GET", url)
}

fn github_post(token: &str, url: &str, body: impl serde::Serialize) -> Option<serde_json::Value> {
    github_request(token, Request::post(url).json(body), "POST", url)
}

fn github_patch(token: &str, url: &str, body: impl serde::Serialize) -> Option<serde_json::Value> {
    github_request(token, Request::patch(url).json(body), "PATCH", url)
}

fn github_delete(token: &str, url: &str) -> bool {
    match Request::delete(url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "plaatdeploy")
        .fetch()
    {
        Ok(response) if response.status == Status::NoContent => true,
        Ok(response) => {
            let message = serde_json::from_slice::<serde_json::Value>(&response.body)
                .ok()
                .and_then(|body| body["message"].as_str().map(str::to_string))
                .unwrap_or_else(|| format!("HTTP {}", response.status as u16));
            log::warn!("GitHub API DELETE {url} error: {message}");
            false
        }
        Err(error) => {
            log::warn!("GitHub API DELETE {url} request failed: {error}");
            false
        }
    }
}

pub(crate) fn team_connection(ctx: &Context, team_id: Uuid) -> Result<Option<TeamGitHubToken>> {
    ctx.database.find_team_github_token_by_team_id(team_id)
}

pub(crate) fn team_token(ctx: &Context, team_id: Uuid) -> Option<String> {
    let token = team_connection(ctx, team_id).ok().flatten()?;
    (!token.access_token.is_empty()).then_some(token.access_token)
}

pub(crate) fn team_is_connected(ctx: &Context, team_id: Uuid) -> Result<bool> {
    Ok(team_token(ctx, team_id).is_some())
}

pub(crate) fn account_login(token: &str) -> Option<String> {
    github_get(token, "https://api.github.com/user")?["login"]
        .as_str()
        .map(str::to_string)
}

/// Resolve a branch to the SHA of its current tip. Deployments must use an immutable
/// commit ref so the GitHub deployment and the checked-out source cannot drift apart.
pub(crate) fn latest_commit_sha(
    ctx: &Context,
    team_id: Uuid,
    repo: &str,
    branch: &str,
) -> Option<String> {
    let token = team_token(ctx, team_id)?;
    let query = serde_urlencoded::to_string([("sha", branch), ("per_page", "1")]).ok()?;
    let url = format!("https://api.github.com/repos/{repo}/commits?{query}");
    github_get(&token, &url)?.as_array()?.first()?["sha"]
        .as_str()
        .map(str::to_string)
}

pub(crate) fn commit_has_pending_runs(ctx: &Context, team_id: Uuid, repo: &str, sha: &str) -> bool {
    std::thread::sleep(Duration::from_secs(3));
    let token = match team_token(ctx, team_id) {
        Some(token) => token,
        None => return false,
    };
    for status in &["queued", "in_progress"] {
        let url = format!(
            "https://api.github.com/repos/{repo}/actions/runs?head_sha={sha}&status={status}&per_page=1"
        );
        if github_get(&token, &url)
            .and_then(|value| value["total_count"].as_u64())
            .unwrap_or(0)
            > 0
        {
            return true;
        }
    }
    false
}

pub(crate) fn create_deployment(
    ctx: &Context,
    team_id: Uuid,
    repo: &str,
    sha: &str,
) -> Option<u64> {
    let token = team_token(ctx, team_id)?;
    let url = format!("https://api.github.com/repos/{repo}/deployments");
    let body = serde_json::json!({
        "ref": sha,
        "environment": ctx.server_host,
        "auto_merge": false,
        "required_contexts": [],
        "description": "plaatdeploy",
    });
    let id = github_post(&token, &url, body)?["id"].as_u64();
    if let Some(id) = id {
        log::info!("Created GitHub deployment {id} for {repo}@{sha}");
    }
    id
}

pub(crate) fn update_deployment_status(
    ctx: &Context,
    team_id: Uuid,
    repo: &str,
    deployment_id: u64,
    state: &str,
    description: &str,
    environment_url: Option<&str>,
    log_url: Option<&str>,
) {
    let Some(token) = team_token(ctx, team_id) else {
        return;
    };
    let url = format!("https://api.github.com/repos/{repo}/deployments/{deployment_id}/statuses");
    let mut body = serde_json::json!({
        "state": state,
        "environment": ctx.server_host,
        "description": description,
    });
    if let Some(environment_url) = environment_url {
        body["environment_url"] = serde_json::Value::String(environment_url.to_string());
    }
    if let Some(log_url) = log_url {
        body["log_url"] = serde_json::Value::String(log_url.to_string());
    }
    github_post(&token, &url, body);
}

pub(crate) fn deactivate_deployment(
    ctx: &Context,
    team_id: Uuid,
    repo: &str,
    deployment_id: i64,
    log_url: &str,
) -> bool {
    let Some(token) = team_token(ctx, team_id) else {
        return false;
    };
    let url = format!("https://api.github.com/repos/{repo}/deployments/{deployment_id}/statuses");
    github_post(
        &token,
        &url,
        serde_json::json!({
            "state": "inactive",
            "environment": ctx.server_host,
            "description": "Project deleted from PlaatDeploy",
            "log_url": log_url,
        }),
    )
    .is_some()
}

pub(crate) fn list_repositories(ctx: &Context, team_id: Uuid) -> Option<Vec<Repository>> {
    let token = team_token(ctx, team_id)?;
    let mut repositories = Vec::new();
    for page in 1..=10 {
        let url = format!(
            "https://api.github.com/user/repos?affiliation=owner,collaborator,organization_member&per_page=100&page={page}"
        );
        let response = github_get(&token, &url)?;
        let page_repositories = response.as_array()?;
        if page_repositories.is_empty() {
            break;
        }
        repositories.extend(page_repositories.iter().filter_map(|repo| {
            Some(Repository {
                full_name: repo["full_name"].as_str()?.to_string(),
            })
        }));
        if page_repositories.len() < 100 {
            break;
        }
    }
    Some(repositories)
}

pub(crate) fn list_branches(ctx: &Context, team_id: Uuid, repo: &str) -> Option<Vec<Branch>> {
    let token = team_token(ctx, team_id)?;
    let mut branches = Vec::new();
    for page in 1..=10 {
        let url = format!("https://api.github.com/repos/{repo}/branches?per_page=100&page={page}");
        let response = github_get(&token, &url)?;
        let page_branches = response.as_array()?;
        if page_branches.is_empty() {
            break;
        }
        branches.extend(page_branches.iter().filter_map(|branch| {
            Some(Branch {
                name: branch["name"].as_str()?.to_string(),
            })
        }));
        if page_branches.len() < 100 {
            break;
        }
    }
    Some(branches)
}

pub(crate) fn ensure_repository_webhook(ctx: &Context, team_id: Uuid, repo: &str) -> bool {
    let Some(token) = team_token(ctx, team_id) else {
        return false;
    };
    let Some(connection) = team_connection(ctx, team_id).ok().flatten() else {
        return false;
    };
    if connection.webhook_secret.is_empty() {
        return false;
    }
    let secret = connection.webhook_secret;
    let url = format!("{}/api/webhook/github?team_id={team_id}", ctx.server_origin);
    let config = serde_json::json!({
        "url": url,
        "content_type": "json",
        "secret": secret,
        "insecure_ssl": "0"
    });
    let hooks_url = format!("https://api.github.com/repos/{repo}/hooks?per_page=100");
    let existing = github_get(&token, &hooks_url)
        .and_then(|hooks| hooks.as_array().cloned())
        .and_then(|hooks| {
            hooks
                .into_iter()
                .find(|hook| is_managed_webhook(ctx, hook, team_id))
        });
    let body = serde_json::json!({ "name": "web", "active": true, "events": ["push", "workflow_run"], "config": config });
    if let Some(hook) = existing {
        let Some(id) = hook["id"].as_u64() else {
            return false;
        };
        github_patch(
            &token,
            &format!("https://api.github.com/repos/{repo}/hooks/{id}"),
            body,
        )
        .is_some()
    } else {
        github_post(
            &token,
            &format!("https://api.github.com/repos/{repo}/hooks"),
            body,
        )
        .is_some()
    }
}

/// Remove PlaatDeploy's webhook for one team from a repository.
pub(crate) fn remove_repository_webhook(ctx: &Context, team_id: Uuid, repo: &str) -> bool {
    let Some(token) = team_token(ctx, team_id) else {
        return false;
    };
    let hooks_url = format!("https://api.github.com/repos/{repo}/hooks?per_page=100");
    let Some(hooks) = github_get(&token, &hooks_url).and_then(|hooks| hooks.as_array().cloned())
    else {
        return false;
    };

    let hook_ids = hooks
        .iter()
        .filter(|hook| is_managed_webhook(ctx, hook, team_id))
        .filter_map(|hook| hook["id"].as_u64())
        .collect::<Vec<_>>();
    if hook_ids.is_empty() {
        log::warn!("No managed PlaatDeploy webhook found for {repo} (team {team_id})");
    }
    for id in hook_ids {
        if !github_delete(
            &token,
            &format!("https://api.github.com/repos/{repo}/hooks/{id}"),
        ) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{is_managed_webhook, is_team_webhook};
    use crate::context::Context;
    use uuid::Uuid;

    #[test]
    fn team_webhook_match_ignores_callback_origin() {
        let team_id = Uuid::now_v7();
        let hook = serde_json::json!({
            "config": { "url": format!("https://old.example.test/api/webhook/github?team_id={team_id}") }
        });
        assert!(is_team_webhook(&hook, team_id));
    }

    #[test]
    fn legacy_webhook_match_uses_current_server_origin() {
        let ctx = Context::with_test_database().expect("test database");
        let hook = serde_json::json!({
            "config": { "url": "http://localhost/api/webhook/github" }
        });
        assert!(is_managed_webhook(&ctx, &hook, Uuid::now_v7()));
    }
}
