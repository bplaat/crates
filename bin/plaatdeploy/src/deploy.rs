/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashSet;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::mpsc::Receiver;

use anyhow::{Context as _, Result, bail};
use chrono::Utc;
use uuid::Uuid;

use crate::context::Context;
use crate::github;
use crate::models::{Deployment, DeploymentMode, DeploymentStatus, Project, ProjectStatus, User};

#[derive(Clone, Copy)]
pub(crate) struct DeployTask {
    pub project_id: Uuid,
    pub deployment_id: Uuid,
}

pub(crate) fn start(ctx: Context, receiver: Receiver<DeployTask>) {
    let recovered = match recover_deployments(&ctx) {
        Ok(tasks) => tasks,
        Err(error) => {
            log::error!("Could not recover deployments: {error:#}");
            Vec::new()
        }
    };
    std::thread::spawn(move || {
        for task in recovered {
            run_deploy_task(&ctx, task);
        }
        for task in receiver {
            run_deploy_task(&ctx, task);
        }
    });
}

fn run_deploy_task(ctx: &Context, task: DeployTask) {
    if let Err(error) = deploy(ctx, task) {
        log::error!("Deployment {} failed: {error:#}", task.deployment_id);
        let now = Utc::now();
        _ = ctx.database.execute(
            "UPDATE deployments SET status = ?, log = ?, updated_at = ? WHERE id = ?",
            (
                DeploymentStatus::Failed as i64,
                format!("{error:#}"),
                now,
                task.deployment_id,
            ),
        );
        _ = ctx.database.execute(
            "UPDATE projects SET status = ?, updated_at = ? WHERE id = ?",
            (ProjectStatus::Failed as i64, now, task.project_id),
        );
    }
}

fn recover_deployments(ctx: &Context) -> Result<Vec<DeployTask>> {
    let interrupted = ctx
        .database
        .query::<(Uuid, Uuid)>(
            "SELECT project_id, id FROM deployments WHERE status = ?",
            DeploymentStatus::Deploying as i64,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    let now = Utc::now();
    for (project_id, deployment_id) in interrupted {
        ctx.database.execute(
            "UPDATE deployments SET status = ?, log = ?, updated_at = ? WHERE id = ?",
            (
                DeploymentStatus::Failed as i64,
                "Deployment interrupted by server restart".to_string(),
                now,
                deployment_id,
            ),
        )?;
        ctx.database.execute(
            "UPDATE projects SET status = ?, updated_at = ? WHERE id = ?",
            (ProjectStatus::Failed as i64, now, project_id),
        )?;
    }
    ctx.database
        .query::<(Uuid, Uuid)>(
            "SELECT project_id, id FROM deployments WHERE status = ? ORDER BY created_at",
            DeploymentStatus::Pending as i64,
        )?
        .map(|row| {
            row.map(|(project_id, deployment_id)| DeployTask {
                project_id,
                deployment_id,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

fn deploy(ctx: &Context, task: DeployTask) -> Result<()> {
    let mut project = find_project(ctx, task.project_id)?;
    let mut deployment = find_deployment(ctx, task.deployment_id)?;
    let user = find_user(ctx, project.user_id)?;
    let token = user.github_token.context("GitHub is not connected")?;
    let commit = github::commit(&token, &project.github_repo, &deployment.commit_sha)?;
    deployment.commit_sha.clone_from(&commit.sha);
    deployment.commit_message = commit
        .commit
        .message
        .lines()
        .next()
        .unwrap_or_default()
        .to_string();
    let deployment_url = format!(
        "{}/projects/{}",
        ctx.server_origin.trim_end_matches('/'),
        project.id
    );
    let site_url = format!("https://{}.{}", project.slug, ctx.deployments_domain);
    let github_deployment_id =
        github::create_deployment(&token, &project.github_repo, &commit.sha, &project.slug).ok();

    let now = Utc::now();
    ctx.database.execute(
        "UPDATE projects SET status = ?, updated_at = ? WHERE id = ?",
        (ProjectStatus::Deploying as i64, now, project.id),
    )?;
    ctx.database.execute(
        "UPDATE deployments SET commit_sha = ?, commit_message = ?, github_deployment_id = ?, status = ?, updated_at = ? WHERE id = ?",
        (deployment.commit_sha.clone(), deployment.commit_message.clone(), github_deployment_id, DeploymentStatus::Deploying as i64, now, deployment.id),
    )?;
    if let Some(id) = github_deployment_id {
        _ = github::deployment_status(
            &token,
            &project.github_repo,
            id,
            "in_progress",
            "Deployment is in progress",
            None,
            &deployment_url,
        );
    }

    let result = deploy_source(ctx, &mut project, &token, &commit.sha);
    let (project_status, deployment_status, github_state, description, environment_url, log) =
        match result {
            Ok(log) => (
                ProjectStatus::Running,
                DeploymentStatus::Succeeded,
                "success",
                "Deployment completed",
                Some(site_url.as_str()),
                log,
            ),
            Err(error) => (
                ProjectStatus::Failed,
                DeploymentStatus::Failed,
                "failure",
                "Deployment failed",
                None,
                format!("{error:#}"),
            ),
        };
    let log = truncate_log(log);
    let now = Utc::now();
    ctx.database.execute(
        "UPDATE projects SET deployment_mode = ?, container_port = ?, status = ?, last_deployed_at = ?, updated_at = ? WHERE id = ?",
        (project.deployment_mode as i64, project.container_port, project_status as i64, now, now, project.id),
    )?;
    ctx.database.execute(
        "UPDATE deployments SET status = ?, log = ?, updated_at = ? WHERE id = ?",
        (deployment_status as i64, log, now, deployment.id),
    )?;
    if let Some(id) = github_deployment_id {
        _ = github::deployment_status(
            &token,
            &project.github_repo,
            id,
            github_state,
            description,
            environment_url,
            &deployment_url,
        );
    }
    Ok(())
}

fn deploy_source(ctx: &Context, project: &mut Project, token: &str, sha: &str) -> Result<String> {
    let project_dir = ctx.data_path.join("projects").join(project.id.to_string());
    let repo_dir = project_dir.join("repo");
    std::fs::create_dir_all(&project_dir)?;
    let mut log = String::new();
    if repo_dir.join(".git").is_dir() {
        append_output(
            &mut log,
            run_git(&repo_dir, token, &["fetch", "--prune", "origin"])?,
        );
    } else {
        let repo_url = format!("https://github.com/{}.git", project.github_repo);
        let repo_path = repo_dir.to_string_lossy().into_owned();
        append_output(
            &mut log,
            run_git(
                &project_dir,
                token,
                &["clone", "--no-checkout", &repo_url, &repo_path],
            )?,
        );
    }
    append_output(
        &mut log,
        run_git(&repo_dir, token, &["reset", "--hard", sha])?,
    );
    append_output(&mut log, run_git(&repo_dir, token, &["clean", "-fdx"])?);

    match project.deployment_mode {
        DeploymentMode::Static => {
            project.container_port = Some(80);
            append_output(&mut log, deploy_static(ctx, project)?);
        }
        DeploymentMode::Dockerfile => {
            let dockerfile_path = project
                .dockerfile_path
                .as_deref()
                .context("Dockerfile path is not configured")?;
            let dockerfile = repo_dir.join(dockerfile_path);
            if !dockerfile.is_file() {
                bail!("Dockerfile does not exist: {dockerfile_path}");
            }
            let config = parse_dockerfile(&dockerfile)?;
            let port = config
                .exposed_port
                .context("Dockerfile must contain a numeric EXPOSE instruction")?;
            project.container_port = Some(i64::from(port));
            append_output(
                &mut log,
                deploy_docker(
                    ctx,
                    project,
                    &repo_dir,
                    dockerfile_path,
                    port,
                    &config.volumes,
                )?,
            );
        }
    }
    Ok(log)
}

fn run_git(directory: &Path, token: &str, args: &[&str]) -> Result<Output> {
    let header = format!("Authorization: Bearer {token}");
    checked_output(
        Command::new("git")
            .current_dir(directory)
            .env("GIT_CONFIG_COUNT", "1")
            .env("GIT_CONFIG_KEY_0", "http.extraHeader")
            .env("GIT_CONFIG_VALUE_0", header)
            .args(args),
    )
}

fn deploy_static(ctx: &Context, project: &Project) -> Result<Output> {
    remove_container(project);
    let subpath = format!("projects/{}/repo", project.id);
    let volume = format!(
        "type=volume,source=plaatdeploy-data,target=/usr/share/nginx/html,volume-subpath={subpath},readonly"
    );
    checked_output(container_command(ctx, project, 80).args(["--mount", &volume, "nginx:alpine"]))
}

fn deploy_docker(
    ctx: &Context,
    project: &Project,
    repo_dir: &Path,
    dockerfile_path: &str,
    port: u16,
    volume_paths: &[String],
) -> Result<Output> {
    let image = image_name(project);
    let build = checked_output(
        Command::new("docker")
            .args(["build", "--tag", &image, "--file", dockerfile_path, "."])
            .current_dir(repo_dir),
    )?;
    let mut output = build;
    let mounts = ensure_project_volumes(project, volume_paths)?;
    remove_container(project);
    let mut command = container_command(ctx, project, port);
    for mount in mounts {
        command.args(["--mount", &mount]);
    }
    let run = checked_output(command.arg(&image))?;
    output.stdout.extend(run.stdout);
    output.stderr.extend(run.stderr);
    Ok(output)
}

fn container_command(ctx: &Context, project: &Project, port: u16) -> Command {
    let name = container_name(project);
    let router = format!("plaatdeploy-{}", compact_id(project.id));
    let host = format!("{}.{}", project.slug, ctx.deployments_domain);
    let rule = format!("traefik.http.routers.{router}.rule=Host(`{host}`)");
    let entrypoints = format!("traefik.http.routers.{router}.entrypoints=websecure");
    let tls = format!("traefik.http.routers.{router}.tls.certresolver=letsencrypt");
    let service = format!("traefik.http.services.{router}.loadbalancer.server.port={port}");
    let mut command = Command::new("docker");
    command.args([
        "run",
        "--detach",
        "--name",
        &name,
        "--restart",
        "unless-stopped",
        "--network",
        "plaatdeploy",
        "--label",
        "traefik.enable=true",
        "--label",
        &rule,
        "--label",
        &entrypoints,
        "--label",
        &tls,
        "--label",
        &service,
    ]);
    command
}

fn checked_output(command: &mut Command) -> Result<Output> {
    let output = command.output().context("failed to start command")?;
    if !output.status.success() {
        bail!(
            "command failed with status {:?}: {}{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(output)
}

fn append_output(log: &mut String, output: Output) {
    log.push_str(&String::from_utf8_lossy(&output.stdout));
    log.push_str(&String::from_utf8_lossy(&output.stderr));
}

fn truncate_log(mut log: String) -> String {
    const MAX: usize = 256 * 1024;
    if log.len() > MAX {
        let mut split = log.len() - MAX;
        while !log.is_char_boundary(split) {
            split += 1;
        }
        log.replace_range(..split, "[earlier output truncated]\n");
    }
    log
}

#[derive(Default)]
struct DockerfileConfig {
    exposed_port: Option<u16>,
    volumes: Vec<String>,
}

fn parse_dockerfile(path: &Path) -> Result<DockerfileConfig> {
    let contents = std::fs::read_to_string(path)?;
    parse_dockerfile_contents(&contents)
}

fn parse_dockerfile_contents(contents: &str) -> Result<DockerfileConfig> {
    let mut config = DockerfileConfig::default();
    let mut seen_volumes = HashSet::new();
    for line in contents.lines() {
        let line = line.trim();
        let mut parts = line.splitn(2, char::is_whitespace);
        let instruction = parts.next().unwrap_or_default();
        let value = parts.next().unwrap_or_default().trim();
        if config.exposed_port.is_none() && instruction.eq_ignore_ascii_case("EXPOSE") {
            config.exposed_port = value
                .split('#')
                .next()
                .unwrap_or_default()
                .split_whitespace()
                .next()
                .and_then(|value| value.split('/').next())
                .and_then(|value| value.parse::<u16>().ok())
                .filter(|port| *port > 0);
        } else if instruction.eq_ignore_ascii_case("VOLUME") {
            for volume in parse_volume_instruction(value)? {
                validate_volume_path(&volume)?;
                if seen_volumes.insert(volume.clone()) {
                    config.volumes.push(volume);
                }
            }
        }
    }
    Ok(config)
}

fn parse_volume_instruction(value: &str) -> Result<Vec<String>> {
    if value.starts_with('[') {
        return serde_json::from_str(value).context("invalid JSON VOLUME instruction");
    }
    Ok(value
        .split_whitespace()
        .map(|path| path.trim_matches(['\'', '"']).to_string())
        .filter(|path| !path.is_empty())
        .collect())
}

fn validate_volume_path(path: &str) -> Result<()> {
    if !path.starts_with('/') || path.contains(',') || path.contains('\0') {
        bail!("Dockerfile VOLUME path must be an absolute container path without commas: {path}");
    }
    Ok(())
}

fn ensure_project_volumes(project: &Project, paths: &[String]) -> Result<Vec<String>> {
    let mut mounts = Vec::with_capacity(paths.len());
    for path in paths {
        let name = volume_name(project.id, path);
        let label = format!("plaatdeploy.project={}", project.id);
        checked_output(
            Command::new("docker").args(["volume", "create", "--label", &label, &name]),
        )?;
        mounts.push(format!("type=volume,source={name},target={path}"));
    }
    Ok(mounts)
}

fn volume_name(project_id: Uuid, path: &str) -> String {
    use sha2::Sha256;

    let hash = Sha256::digest(path.as_bytes());
    let suffix = hash[..6]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("plaatdeploy-{}-{suffix}", compact_id(project_id))
}

fn remove_container(project: &Project) {
    _ = Command::new("docker")
        .args(["rm", "--force", &container_name(project)])
        .output();
}

pub(crate) fn cleanup(ctx: &Context, project: &Project) -> Result<()> {
    remove_container(project);
    _ = Command::new("docker")
        .args(["image", "rm", &image_name(project)])
        .output();
    remove_project_volumes(project);
    let directory = ctx.data_path.join("projects").join(project.id.to_string());
    if directory.is_dir() {
        std::fs::remove_dir_all(directory)?;
    }
    Ok(())
}

fn remove_project_volumes(project: &Project) {
    let filter = format!("label=plaatdeploy.project={}", project.id);
    let Ok(output) = Command::new("docker")
        .args(["volume", "ls", "--quiet", "--filter", &filter])
        .output()
    else {
        return;
    };
    for name in String::from_utf8_lossy(&output.stdout).lines() {
        if !name.is_empty() {
            _ = Command::new("docker").args(["volume", "rm", name]).output();
        }
    }
}

fn container_name(project: &Project) -> String {
    format!("plaatdeploy-{}", compact_id(project.id))
}
fn image_name(project: &Project) -> String {
    format!("plaatdeploy-project-{}", compact_id(project.id))
}

fn compact_id(id: Uuid) -> String {
    id.to_string().replace('-', "")
}

fn find_project(ctx: &Context, id: Uuid) -> Result<Project> {
    ctx.database
        .query::<Project>(
            format!(
                "SELECT {} FROM projects WHERE id = ? LIMIT 1",
                Project::columns()
            ),
            id,
        )?
        .next()
        .transpose()?
        .context("project not found")
}

fn find_deployment(ctx: &Context, id: Uuid) -> Result<Deployment> {
    ctx.database
        .query::<Deployment>(
            format!(
                "SELECT {} FROM deployments WHERE id = ? LIMIT 1",
                Deployment::columns()
            ),
            id,
        )?
        .next()
        .transpose()?
        .context("deployment not found")
}

fn find_user(ctx: &Context, id: Uuid) -> Result<User> {
    ctx.database
        .query::<User>(
            format!("SELECT {} FROM users WHERE id = ? LIMIT 1", User::columns()),
            id,
        )?
        .next()
        .transpose()?
        .context("user not found")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::DatabaseHelpers;
    use crate::test_utils::{insert_test_project, insert_test_user};

    #[test]
    fn parses_dockerfile_configuration() {
        let config = parse_dockerfile_contents(
            "FROM scratch\nEXPOSE 8080/tcp\nVOLUME [\"/data\", \"/cache\"]\nVOLUME /uploads\n",
        )
        .expect("parse Dockerfile");

        assert_eq!(config.exposed_port, Some(8080));
        assert_eq!(config.volumes, ["/data", "/cache", "/uploads"]);
    }

    #[test]
    fn handles_dockerfile_instruction_edge_cases() {
        let config = parse_dockerfile_contents(
            "expose invalid\nEXPOSE 3000/udp # comment\nVOLUME /data /data '/uploads'\n",
        )
        .expect("parse Dockerfile");

        assert_eq!(config.exposed_port, Some(3000));
        assert_eq!(config.volumes, ["/data", "/uploads"]);
        assert!(parse_dockerfile_contents("VOLUME relative\n").is_err());
        assert!(parse_dockerfile_contents("VOLUME [not-json]\n").is_err());
        assert!(parse_dockerfile_contents("VOLUME [\"/data,cache\"]\n").is_err());
    }

    #[test]
    fn truncates_large_unicode_logs_on_a_character_boundary() {
        let log = truncate_log("é".repeat(256 * 1024));

        assert!(log.starts_with("[earlier output truncated]\n"));
        assert!(log.ends_with('é'));
        assert!(log.len() <= 256 * 1024 + "[earlier output truncated]\n".len());
    }

    #[test]
    fn keeps_small_logs_unchanged() {
        assert_eq!(truncate_log("build complete".to_string()), "build complete");
    }

    #[test]
    fn creates_stable_project_resource_names() {
        let id = Uuid::from_bytes([
            0x01, 0x8f, 0x00, 0x00, 0x00, 0x00, 0x70, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01,
        ]);
        let project = Project {
            id,
            ..Default::default()
        };

        assert_eq!(compact_id(id), "018f0000000070008000000000000001");
        assert_eq!(
            container_name(&project),
            "plaatdeploy-018f0000000070008000000000000001"
        );
        assert_eq!(
            image_name(&project),
            "plaatdeploy-project-018f0000000070008000000000000001"
        );
        assert_eq!(volume_name(id, "/data"), volume_name(id, "/data"));
        assert_ne!(volume_name(id, "/data"), volume_name(id, "/cache"));
    }

    #[test]
    fn recovers_pending_and_marks_interrupted_deployments_failed() {
        let ctx = Context::with_test_database().expect("create test database");
        let user = insert_test_user(&ctx);
        let project = insert_test_project(&ctx, user.id);
        let pending = Deployment {
            project_id: project.id,
            ..Default::default()
        };
        let interrupted = Deployment {
            project_id: project.id,
            status: DeploymentStatus::Deploying,
            ..Default::default()
        };
        ctx.database
            .insert_deployment(pending.clone())
            .expect("insert pending deployment");
        ctx.database
            .insert_deployment(interrupted.clone())
            .expect("insert interrupted deployment");

        let tasks = recover_deployments(&ctx).expect("recover deployments");
        let (status, log) = ctx
            .database
            .query_some::<(DeploymentStatus, String)>(
                "SELECT status, log FROM deployments WHERE id = ?",
                interrupted.id,
            )
            .expect("query interrupted deployment");

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].project_id, project.id);
        assert_eq!(tasks[0].deployment_id, pending.id);
        assert_eq!(status, DeploymentStatus::Failed);
        assert_eq!(log, "Deployment interrupted by server restart");
    }
}
