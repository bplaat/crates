/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::{Context as _, Result, bail};
use serde::{Deserialize, Serialize};
use small_http::{Request, Status};

use crate::api;

const API: &str = "https://api.github.com";

#[derive(Deserialize)]
struct GitHubUser {
    login: String,
}

#[derive(Deserialize)]
struct Repository {
    full_name: String,
    name: String,
    default_branch: String,
    private: bool,
    html_url: String,
}

#[derive(Deserialize)]
struct Hook {
    id: i64,
}

#[derive(Deserialize)]
pub(crate) struct Commit {
    pub sha: String,
    pub commit: CommitDetails,
}

#[derive(Deserialize)]
pub(crate) struct CommitDetails {
    pub message: String,
    tree: GitTreeReference,
}

#[derive(Deserialize)]
struct GitTreeReference {
    sha: String,
}

#[derive(Deserialize)]
struct GitHubDeployment {
    id: i64,
}

#[derive(Deserialize)]
struct GitTree {
    tree: Vec<GitTreeEntry>,
    truncated: bool,
}

#[derive(Deserialize)]
struct GitTreeEntry {
    path: String,
    #[serde(rename = "type")]
    kind: String,
}

fn send<T: for<'de> Deserialize<'de>>(token: &str, request: Request) -> Result<T> {
    let response = request
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "plaatdeploy")
        .fetch()
        .context("GitHub request failed")?;
    if (response.status as u16) >= 400 {
        let message = serde_json::from_slice::<serde_json::Value>(&response.body)
            .ok()
            .and_then(|body| body["message"].as_str().map(str::to_string))
            .unwrap_or_else(|| format!("HTTP {}", response.status as u16));
        bail!("GitHub API: {message}");
    }
    serde_json::from_slice(&response.body).context("invalid GitHub response")
}

pub(crate) fn account_login(token: &str) -> Result<String> {
    Ok(send::<GitHubUser>(token, Request::get(format!("{API}/user")))?.login)
}

pub(crate) fn repositories(token: &str) -> Result<Vec<api::GitHubRepository>> {
    let mut repositories = Vec::new();
    let mut page = 1;
    loop {
        let repos = send::<Vec<Repository>>(
            token,
            Request::get(format!(
                "{API}/user/repos?affiliation=owner,collaborator&sort=updated&per_page=100&page={page}"
            )),
        )?;
        let is_last_page = repos.len() < 100;
        repositories.extend(repos.into_iter().map(|repo| api::GitHubRepository {
            full_name: repo.full_name,
            name: repo.name,
            default_branch: repo.default_branch,
            private: repo.private,
            html_url: repo.html_url,
        }));
        if is_last_page {
            break;
        }
        page += 1;
    }
    Ok(repositories)
}

pub(crate) fn repository(token: &str, full_name: &str) -> Result<api::GitHubRepository> {
    let repo = send::<Repository>(token, Request::get(format!("{API}/repos/{full_name}")))?;
    Ok(api::GitHubRepository {
        full_name: repo.full_name,
        name: repo.name,
        default_branch: repo.default_branch,
        private: repo.private,
        html_url: repo.html_url,
    })
}

pub(crate) fn find_dockerfile(token: &str, repo: &str, branch: &str) -> Result<Option<String>> {
    let tree_sha = commit(token, repo, branch)?.commit.tree.sha;
    let tree = send::<GitTree>(
        token,
        Request::get(format!(
            "{API}/repos/{repo}/git/trees/{tree_sha}?recursive=1"
        )),
    )?;
    let dockerfile = find_dockerfile_in_tree(tree.tree);
    if dockerfile.is_none() && tree.truncated {
        bail!("GitHub repository tree is truncated");
    }
    Ok(dockerfile)
}

fn find_dockerfile_in_tree(entries: Vec<GitTreeEntry>) -> Option<String> {
    entries
        .into_iter()
        .filter(|entry| {
            entry.kind == "blob"
                && entry
                    .path
                    .rsplit('/')
                    .next()
                    .is_some_and(|name| name == "Dockerfile")
        })
        .min_by_key(|entry| (entry.path.matches('/').count(), entry.path.len()))
        .map(|entry| entry.path)
}

pub(crate) fn commit(token: &str, repo: &str, git_ref: &str) -> Result<Commit> {
    let git_ref = encode_path_segment(git_ref);
    send(
        token,
        Request::get(format!("{API}/repos/{repo}/commits/{git_ref}")),
    )
}

fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

pub(crate) fn create_webhook(
    token: &str,
    repo: &str,
    callback_url: &str,
    secret: &str,
) -> Result<i64> {
    #[derive(Serialize)]
    struct Config<'a> {
        url: &'a str,
        content_type: &'static str,
        secret: &'a str,
        insecure_ssl: &'static str,
    }
    #[derive(Serialize)]
    struct Body<'a> {
        name: &'static str,
        active: bool,
        events: [&'static str; 1],
        config: Config<'a>,
    }
    let body = Body {
        name: "web",
        active: true,
        events: ["push"],
        config: Config {
            url: callback_url,
            content_type: "json",
            secret,
            insecure_ssl: "0",
        },
    };
    Ok(send::<Hook>(
        token,
        Request::post(format!("{API}/repos/{repo}/hooks")).json(body),
    )?
    .id)
}

pub(crate) fn delete_webhook(token: &str, repo: &str, hook_id: i64) -> Result<()> {
    let response = Request::delete(format!("{API}/repos/{repo}/hooks/{hook_id}"))
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "plaatdeploy")
        .fetch()?;
    if response.status != Status::NoContent && response.status != Status::NotFound {
        bail!(
            "GitHub webhook deletion failed with HTTP {}",
            response.status as u16
        );
    }
    Ok(())
}

pub(crate) fn create_deployment(
    token: &str,
    repo: &str,
    sha: &str,
    environment: &str,
) -> Result<i64> {
    let body = serde_json::json!({
        "ref": sha, "environment": environment, "auto_merge": false,
        "required_contexts": [], "description": "Plaat Deploy"
    });
    Ok(send::<GitHubDeployment>(
        token,
        Request::post(format!("{API}/repos/{repo}/deployments")).json(body),
    )?
    .id)
}

pub(crate) fn deployment_status(
    token: &str,
    repo: &str,
    deployment_id: i64,
    state: &str,
    description: &str,
    environment_url: Option<&str>,
    log_url: &str,
) -> Result<()> {
    let mut body = serde_json::json!({
        "state": state, "description": description, "log_url": log_url, "auto_inactive": true
    });
    if let Some(url) = environment_url {
        body["environment_url"] = serde_json::Value::String(url.to_string());
    }
    let _: serde_json::Value = send(
        token,
        Request::post(format!(
            "{API}/repos/{repo}/deployments/{deployment_id}/statuses"
        ))
        .json(body),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, kind: &str) -> GitTreeEntry {
        GitTreeEntry {
            path: path.to_string(),
            kind: kind.to_string(),
        }
    }

    #[test]
    fn prefers_the_shallowest_dockerfile() {
        let path = find_dockerfile_in_tree(vec![
            entry("apps/web/Dockerfile", "blob"),
            entry("docker/Dockerfile", "blob"),
            entry("Dockerfile", "blob"),
        ]);

        assert_eq!(path.as_deref(), Some("Dockerfile"));
    }

    #[test]
    fn ignores_directories_and_different_file_names() {
        let path = find_dockerfile_in_tree(vec![
            entry("Dockerfile", "tree"),
            entry("dockerfile", "blob"),
            entry("Dockerfile.dev", "blob"),
            entry("services/api/Dockerfile", "blob"),
        ]);

        assert_eq!(path.as_deref(), Some("services/api/Dockerfile"));
        assert_eq!(find_dockerfile_in_tree(Vec::new()), None);
    }

    #[test]
    fn percent_encodes_git_references_as_path_segments() {
        assert_eq!(
            encode_path_segment("feature/deploy #1"),
            "feature%2Fdeploy%20%231"
        );
        assert_eq!(encode_path_segment("abc-_.~123"), "abc-_.~123");
    }
}
