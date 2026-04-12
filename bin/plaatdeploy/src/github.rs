/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! GitHub API helpers via GitHub App installation tokens

use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::Result;
use base64::prelude::*;
use uuid::Uuid;

use crate::context::{Context, DatabaseHelpers};
use crate::models::TeamGitHubConnection;

// MARK: Token cache
struct TokenCache {
    token: String,
    expires_at: Instant,
}

static TOKEN_CACHE: Mutex<Option<HashMap<i64, TokenCache>>> = Mutex::new(None);

pub(crate) fn clear_token_cache() {
    if let Ok(mut cache) = TOKEN_CACHE.lock() {
        cache.take();
    }
}

#[derive(Clone)]
pub(crate) struct Repository {
    pub full_name: String,
}

#[derive(Clone)]
pub(crate) struct Branch {
    pub name: String,
}

#[derive(Clone)]
pub(crate) struct Installation {
    pub id: i64,
    pub account_login: String,
    pub account_type: Option<String>,
}

impl TryFrom<TeamGitHubConnection> for Installation {
    type Error = ();

    fn try_from(connection: TeamGitHubConnection) -> std::result::Result<Self, Self::Error> {
        let installation_id = connection.installation_id.ok_or(())?;
        let account_login = connection.account_login.ok_or(())?;
        Ok(Self {
            id: installation_id,
            account_login,
            account_type: connection.account_type,
        })
    }
}

fn connection_credentials(connection: &TeamGitHubConnection) -> Option<(String, String)> {
    let app_id = connection.app_id.clone()?;
    let private_key = connection.app_private_key.clone()?;
    if app_id.is_empty() || private_key.is_empty() {
        return None;
    }
    Some((app_id, private_key))
}

fn app_credentials_for_team(ctx: &Context, team_id: Uuid) -> Result<Option<(String, String)>> {
    Ok(team_connection(ctx, team_id)?
        .as_ref()
        .and_then(connection_credentials))
}

fn app_credentials_for_installation(
    ctx: &Context,
    installation_id: i64,
) -> Result<Option<(String, String)>> {
    Ok(ctx
        .database
        .find_team_github_connection_by_installation_id(installation_id)?
        .as_ref()
        .and_then(connection_credentials))
}

// MARK: JWT
fn base64url_encode(data: &[u8]) -> String {
    BASE64_URL_SAFE_NO_PAD.encode(data)
}

fn generate_jwt(app_id: &str, private_key_pem: &str) -> Option<String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

    let header = base64url_encode(b"{\"alg\":\"RS256\",\"typ\":\"JWT\"}");
    let payload = base64url_encode(
        format!(
            "{{\"iat\":{},\"exp\":{},\"iss\":\"{}\"}}",
            now - 60,
            now + 540,
            app_id
        )
        .as_bytes(),
    );
    let message = format!("{header}.{payload}");

    // Write private key to temp file (openssl needs a file for the signing key)
    let key_path = std::env::temp_dir().join("pd-gh-app.pem");
    std::fs::write(&key_path, private_key_pem).ok()?;

    let mut child = Command::new("openssl")
        .args(["dgst", "-sha256", "-sign", key_path.to_str()?])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(message.as_bytes()).ok()?;
    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        log::warn!("openssl dgst failed: could not sign JWT");
        return None;
    }

    let signature = base64url_encode(&output.stdout);
    Some(format!("{message}.{signature}"))
}

// MARK: Installation token
fn fetch_installation_token(
    app_id: &str,
    private_key_pem: &str,
    installation_id: i64,
) -> Option<String> {
    let jwt = generate_jwt(app_id, private_key_pem)?;

    // Exchange for an installation access token
    let url = format!("https://api.github.com/app/installations/{installation_id}/access_tokens");
    let output = Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            "-H",
            &format!("Authorization: Bearer {jwt}"),
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "X-GitHub-Api-Version: 2022-11-28",
            "-H",
            "User-Agent: plaatdeploy",
            &url,
        ])
        .output()
        .ok()?;
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    if let Some(msg) = val["message"].as_str() {
        log::warn!("GitHub installation token error: {msg}");
        return None;
    }
    val["token"].as_str().map(str::to_owned)
}

pub(crate) fn list_installations(ctx: &Context, team_id: Uuid) -> Option<Vec<Installation>> {
    let (app_id, private_key) = app_credentials_for_team(ctx, team_id).ok().flatten()?;
    let jwt = generate_jwt(&app_id, &private_key)?;
    let mut installations = Vec::new();

    for page in 1..=10 {
        let url = format!("https://api.github.com/app/installations?per_page=100&page={page}");
        let output = Command::new("curl")
            .args([
                "-s",
                "-H",
                &format!("Authorization: Bearer {jwt}"),
                "-H",
                "Accept: application/vnd.github+json",
                "-H",
                "X-GitHub-Api-Version: 2022-11-28",
                "-H",
                "User-Agent: plaatdeploy",
                &url,
            ])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let val: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
        let page_installations = val.as_array()?;
        if page_installations.is_empty() {
            break;
        }

        installations.extend(page_installations.iter().filter_map(|installation| {
            Some(Installation {
                id: installation["id"].as_i64()?,
                account_login: installation["account"]["login"].as_str()?.to_string(),
                account_type: installation["account"]["type"].as_str().map(str::to_string),
            })
        }));

        if page_installations.len() < 100 {
            break;
        }
    }

    Some(installations)
}

pub(crate) fn team_connection(
    ctx: &Context,
    team_id: Uuid,
) -> Result<Option<TeamGitHubConnection>> {
    ctx.database.find_team_github_connection_by_team_id(team_id)
}

pub(crate) fn team_installation(ctx: &Context, team_id: Uuid) -> Result<Option<Installation>> {
    if let Some(connection) = team_connection(ctx, team_id)? {
        return Ok(Installation::try_from(connection).ok());
    }
    Ok(None)
}

pub(crate) fn team_uses_installation(
    ctx: &Context,
    team_id: Uuid,
    installation_id: i64,
) -> Result<bool> {
    Ok(team_installation(ctx, team_id)?
        .map(|installation| installation.id == installation_id)
        .unwrap_or(false))
}

fn get_token(ctx: &Context, installation_id: i64) -> Option<String> {
    let (app_id, private_key) = app_credentials_for_installation(ctx, installation_id)
        .ok()
        .flatten()?;

    let mut cache = TOKEN_CACHE.lock().ok()?;
    let cache_map = cache.get_or_insert_with(HashMap::new);
    // Reuse cached token if it has more than 5 minutes left
    if let Some(entry) = cache_map.get(&installation_id)
        && entry.expires_at > Instant::now() + Duration::from_secs(300)
    {
        return Some(entry.token.clone());
    }
    // Fetch a new token
    let token = fetch_installation_token(&app_id, &private_key, installation_id)?;
    cache_map.insert(
        installation_id,
        TokenCache {
            token: token.clone(),
            // Installation tokens are valid for 1 hour
            expires_at: Instant::now() + Duration::from_secs(3600),
        },
    );
    log::info!("Refreshed GitHub installation token for installation {installation_id}");
    Some(token)
}

// MARK: API helpers
fn curl_get(token: &str, url: &str) -> Option<serde_json::Value> {
    let output = Command::new("curl")
        .args([
            "-s",
            "-H",
            &format!("Authorization: Bearer {token}"),
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "X-GitHub-Api-Version: 2022-11-28",
            "-H",
            "User-Agent: plaatdeploy",
            url,
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        log::warn!("GitHub API GET {url} curl failed");
        return None;
    }
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    if let Some(msg) = val["message"].as_str() {
        log::warn!("GitHub API GET {url} error: {msg}");
        return None;
    }
    Some(val)
}

fn curl_post(token: &str, url: &str, body: &str) -> Option<serde_json::Value> {
    let output = Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            "-H",
            &format!("Authorization: Bearer {token}"),
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "X-GitHub-Api-Version: 2022-11-28",
            "-H",
            "User-Agent: plaatdeploy",
            "-H",
            "Content-Type: application/json",
            "-d",
            body,
            url,
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        log::warn!("GitHub API POST {url} curl failed");
        return None;
    }
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    if let Some(msg) = val["message"].as_str() {
        log::warn!("GitHub API POST {url} error: {msg}");
        return None;
    }
    Some(val)
}

// MARK: Public API
/// Returns true if the specific commit SHA has any queued or in-progress workflow runs.
/// Sleeps briefly first to allow GitHub to queue runs after a push event.
pub(crate) fn commit_has_pending_runs(
    ctx: &Context,
    installation_id: i64,
    repo: &str,
    sha: &str,
) -> bool {
    std::thread::sleep(Duration::from_secs(3));
    let token = match get_token(ctx, installation_id) {
        Some(t) => t,
        None => return false,
    };
    for status in &["queued", "in_progress"] {
        let url = format!(
            "https://api.github.com/repos/{repo}/actions/runs?head_sha={sha}&status={status}&per_page=1"
        );
        if curl_get(&token, &url)
            .and_then(|v| v["total_count"].as_u64())
            .unwrap_or(0)
            > 0
        {
            return true;
        }
    }
    false
}

/// Create a GitHub Deployment and return its numeric ID.
pub(crate) fn create_deployment(
    ctx: &Context,
    installation_id: i64,
    repo: &str,
    sha: &str,
) -> Option<u64> {
    let token = get_token(ctx, installation_id)?;
    let url = format!("https://api.github.com/repos/{repo}/deployments");
    let host = &ctx.server_host;
    let body = format!(
        r#"{{"ref":"{sha}","environment":"{host}","auto_merge":false,"required_contexts":[],"description":"plaatdeploy"}}"#
    );
    let id = curl_post(&token, &url, &body)?["id"].as_u64();
    if let Some(id) = id {
        log::info!("Created GitHub deployment {id} for {repo}@{sha}");
    }
    id
}

/// Update a GitHub Deployment status.
/// `state`: "queued" | "in_progress" | "success" | "failure" | "error"
pub(crate) fn update_deployment_status(
    ctx: &Context,
    installation_id: i64,
    repo: &str,
    deployment_id: u64,
    state: &str,
    environment_url: Option<&str>,
) {
    let token = match get_token(ctx, installation_id) {
        Some(t) => t,
        None => return,
    };
    let url = format!("https://api.github.com/repos/{repo}/deployments/{deployment_id}/statuses");
    let host = &ctx.server_host;
    let env_url_field = environment_url
        .map(|u| format!(r#","environment_url":"{u}""#))
        .unwrap_or_default();
    let body = format!(r#"{{"state":"{state}","environment":"{host}"{env_url_field}}}"#);
    curl_post(&token, &url, &body);
    log::info!("Updated GitHub deployment {deployment_id} status to {state}");
}

/// List repositories available to the configured GitHub App installation.
pub(crate) fn list_repositories(ctx: &Context, installation_id: i64) -> Option<Vec<Repository>> {
    let token = get_token(ctx, installation_id)?;
    let mut repositories = Vec::new();

    for page in 1..=10 {
        let url =
            format!("https://api.github.com/installation/repositories?per_page=100&page={page}");
        let response = curl_get(&token, &url)?;
        let page_repositories = response["repositories"].as_array()?;
        if page_repositories.is_empty() {
            break;
        }

        repositories.extend(page_repositories.iter().filter_map(|repository| {
            Some(Repository {
                full_name: repository["full_name"].as_str()?.to_string(),
            })
        }));

        if page_repositories.len() < 100 {
            break;
        }
    }

    Some(repositories)
}

/// List branches available for a repository installed on the configured GitHub App.
pub(crate) fn list_branches(
    ctx: &Context,
    installation_id: i64,
    repo: &str,
) -> Option<Vec<Branch>> {
    let token = get_token(ctx, installation_id)?;
    let mut branches = Vec::new();

    for page in 1..=10 {
        let url = format!("https://api.github.com/repos/{repo}/branches?per_page=100&page={page}");
        let response = curl_get(&token, &url)?;
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
