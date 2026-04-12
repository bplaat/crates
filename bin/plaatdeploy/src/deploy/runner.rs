/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Background deploy runner thread

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::Receiver;
use std::thread;

use chrono::Utc;

use crate::context::{Context, DatabaseHelpers};
use crate::deploy::DeployTask;
use crate::github;
use crate::models::{BuildType, Deployment, DeploymentStatus, Project, ProjectStatus};

/// Start the deploy network, restart previously-running containers, then spawn the runner thread.
pub(crate) fn start(ctx: Context, rx: Receiver<DeployTask>, deploy_path: String) {
    // Ensure shared Docker network exists for container-to-container communication
    let _ = Command::new("docker")
        .args(["network", "create", "plaatdeploy"])
        .output();

    restart_running_projects(&ctx, &deploy_path);

    thread::spawn(move || {
        for task in rx {
            log::info!("Starting deploy task {:?}", task.deployment_id);
            if let Err(e) = run_deploy(&ctx, &task, &deploy_path) {
                log::error!("Deploy task failed: {e}");
            }
        }
    });
}

/// Stop all plaatdeploy-managed child containers. Called on shutdown.
pub(crate) fn stop_all_containers() {
    let Ok(output) = Command::new("docker")
        .args([
            "ps",
            "--filter",
            "name=plaatdeploy-",
            "--format",
            "{{.Names}}",
        ])
        .output()
    else {
        return;
    };
    for name in String::from_utf8_lossy(&output.stdout).lines() {
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        log::info!("Stopping container {name}");
        let _ = Command::new("docker").args(["stop", name]).output();
    }
}

/// On startup, reconcile the DB with running Docker containers.
/// - Container already running  → refresh IP in DB.
/// - Image exists but container stopped → restart container, refresh IP.
/// - Image missing              → enqueue a full deploy.
fn restart_running_projects(ctx: &Context, deploy_path: &str) {
    let columns = Project::columns();
    let projects = match ctx.database.query::<Project>(
        &format!("SELECT {columns} FROM projects WHERE build_type = ? AND status IN (?, ?)"),
        (
            BuildType::Docker as i64,
            ProjectStatus::Running as i64,
            ProjectStatus::Building as i64,
        ),
    ) {
        Ok(iter) => iter.filter_map(|r| r.ok()).collect::<Vec<_>>(),
        Err(e) => {
            log::error!("Failed to query running projects on startup: {e}");
            return;
        }
    };

    for project in projects {
        let container = format!("plaatdeploy-{}", project.name);

        // Is the container already up?
        let running = Command::new("docker")
            .args(["inspect", "--format", "{{.State.Running}}", &container])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "true")
            .unwrap_or(false);

        if running {
            let _ = Command::new("docker").args(["stop", &container]).output();
            let _ = Command::new("docker").args(["rm", &container]).output();
        }

        // Does the image still exist?
        let image_exists = Command::new("docker")
            .args(["image", "inspect", &container])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if image_exists {
            let work_dir = project_work_dir(&project, deploy_path);
            let volume_paths = parse_dockerfile_config(&format!("{work_dir}/Dockerfile")).volumes;
            let ok = start_container(&project, deploy_path, &volume_paths, None).is_ok();

            if ok {
                refresh_container_ip(ctx, &project, &container);
                log::info!("Project '{}': restarted from existing image", project.name);
            } else {
                log::warn!(
                    "Project '{}': failed to restart container, queuing redeploy",
                    project.name
                );
                enqueue_redeploy(ctx, &project);
            }
        } else {
            log::info!(
                "Project '{}': image not found, queuing full redeploy",
                project.name
            );
            enqueue_redeploy(ctx, &project);
        }
    }
}

fn refresh_container_ip(ctx: &Context, project: &Project, container: &str) {
    let Ok(ip_output) = Command::new("docker")
        .args([
            "inspect",
            "-f",
            r#"{{(index .NetworkSettings.Networks "plaatdeploy").IPAddress}}"#,
            container,
        ])
        .output()
    else {
        return;
    };
    let ip = String::from_utf8_lossy(&ip_output.stdout)
        .trim()
        .to_string();
    if !ip.is_empty() {
        let _ = ctx.database.execute(
            "UPDATE projects SET container_ip = ?, status = ?, updated_at = ? WHERE id = ?",
            (ip, ProjectStatus::Running as i64, Utc::now(), project.id),
        );
    }
}

fn enqueue_redeploy(ctx: &Context, project: &Project) {
    let deployment = Deployment {
        project_id: project.id,
        commit_sha: "auto-restart".to_string(),
        commit_message: "Automatic redeploy on startup".to_string(),
        ..Default::default()
    };
    let deployment_id = deployment.id;
    if ctx.database.insert_deployment(deployment).is_ok() {
        let _ = ctx.deploy_tx.send(DeployTask {
            project_id: project.id,
            deployment_id,
            github_deployment_id: None,
        });
    }
}

fn run_deploy(ctx: &Context, task: &DeployTask, deploy_path: &str) -> anyhow::Result<()> {
    let project = match ctx.database.find_project_by_id(task.project_id)? {
        Some(p) => p,
        None => {
            log::warn!("Project {:?} not found for deploy task", task.project_id);
            return Ok(());
        }
    };

    // Mark deployment as building
    ctx.database.execute(
        "UPDATE deployments SET status = ?, updated_at = ? WHERE id = ?",
        (
            DeploymentStatus::Building as i64,
            Utc::now(),
            task.deployment_id,
        ),
    )?;
    ctx.database.execute(
        "UPDATE projects SET status = ?, updated_at = ? WHERE id = ?",
        (ProjectStatus::Building as i64, Utc::now(), project.id),
    )?;
    let github_installation_id =
        github::team_installation(ctx, project.team_id)?.map(|installation| installation.id);
    if let (Some(gd_id), Some(installation_id)) =
        (task.github_deployment_id, github_installation_id)
    {
        github::update_deployment_status(
            ctx,
            installation_id,
            &project.github_repo,
            gd_id,
            "in_progress",
            None,
        );
    }

    let mut log_buf = String::new();

    let result = do_deploy(ctx, &project, task, deploy_path, &mut log_buf);

    let (final_status, project_status) = match result {
        Ok(()) => (DeploymentStatus::Succeeded, ProjectStatus::Running),
        Err(ref e) => {
            log_buf.push_str(&format!("\nError: {e}"));
            (DeploymentStatus::Failed, ProjectStatus::Failed)
        }
    };

    ctx.database.execute(
        "UPDATE deployments SET status = ?, log = ?, updated_at = ? WHERE id = ?",
        (
            final_status as i64,
            log_buf.clone(),
            Utc::now(),
            task.deployment_id,
        ),
    )?;
    ctx.database.execute(
        "UPDATE projects SET status = ?, last_deployed_at = ?, updated_at = ? WHERE id = ?",
        (project_status as i64, Utc::now(), Utc::now(), project.id),
    )?;
    if let (Some(gd_id), Some(installation_id)) =
        (task.github_deployment_id, github_installation_id)
    {
        let gh_state = if final_status == DeploymentStatus::Succeeded {
            "success"
        } else {
            "failure"
        };
        let env_url = (final_status == DeploymentStatus::Succeeded)
            .then(|| format!("https://{}.{}", project.name, ctx.server_host));
        github::update_deployment_status(
            ctx,
            installation_id,
            &project.github_repo,
            gd_id,
            gh_state,
            env_url.as_deref(),
        );
    }

    log::info!(
        "Deploy task {:?} finished: {:?}",
        task.deployment_id,
        final_status
    );

    result
}

fn do_deploy(
    ctx: &Context,
    project: &Project,
    task: &DeployTask,
    deploy_path: &str,
    log_buf: &mut String,
) -> anyhow::Result<()> {
    let repo_dir = project_repo_dir(project, deploy_path);

    // Clone or pull repo
    if Path::new(&repo_dir).exists() {
        run_cmd(
            Command::new("git")
                .args(["-C", &repo_dir, "fetch", "--depth=1", "origin"])
                .arg(&project.github_branch),
            log_buf,
        )?;
        run_cmd(
            Command::new("git")
                .args(["-C", &repo_dir, "reset", "--hard"])
                .arg(format!("origin/{}", project.github_branch)),
            log_buf,
        )?;
    } else {
        std::fs::create_dir_all(&repo_dir)?;
        run_cmd(
            Command::new("git").args([
                "clone",
                "--depth=1",
                "--branch",
                &project.github_branch,
                &format!("https://github.com/{}", project.github_repo),
                &repo_dir,
            ]),
            log_buf,
        )?;
    }

    // Detect build type from base_dir
    let work_dir = project_work_dir(project, deploy_path);

    let static_index = format!("{work_dir}/index.html");
    let dockerfile_path = format!("{work_dir}/Dockerfile");

    let build_type = if Path::new(&static_index).exists() {
        BuildType::Static
    } else if Path::new(&dockerfile_path).exists() {
        BuildType::Docker
    } else {
        return Err(anyhow::anyhow!(
            "Neither index.html nor Dockerfile found in {work_dir}"
        ));
    };

    // Persist detected build type
    ctx.database.execute(
        "UPDATE projects SET build_type = ?, updated_at = ? WHERE id = ?",
        (build_type as i64, Utc::now(), project.id),
    )?;

    match build_type {
        BuildType::Static => {
            log_buf.push_str(&format!("Detected static site. Serving from {work_dir}\n"));
        }
        BuildType::Docker => {
            let container = format!("plaatdeploy-{}", project.name);
            let dockerfile_config = parse_dockerfile_config(&dockerfile_path);

            // Use manual container_port override if set, else auto-detect from EXPOSE
            let container_port: u16 = if let Some(p) = project.container_port {
                p as u16
            } else {
                dockerfile_config.exposed_port.unwrap_or(3000)
            };

            // Persist detected internal port
            ctx.database.execute(
                "UPDATE projects SET container_port = ?, updated_at = ? WHERE id = ?",
                (container_port as i64, Utc::now(), project.id),
            )?;

            log_buf.push_str(&format!("Building Docker image {container}...\n"));
            run_cmd(
                Command::new("docker").args(["build", "-t", &container, &work_dir]),
                log_buf,
            )?;

            // Stop and remove old container (ignore errors)
            let _ = Command::new("docker").args(["stop", &container]).output();
            let _ = Command::new("docker").args(["rm", &container]).output();

            log_buf.push_str(&format!(
                "Starting container {container} on internal port {container_port}...\n"
            ));
            start_container(
                project,
                deploy_path,
                &dockerfile_config.volumes,
                Some(log_buf),
            )?;

            // Get container IP on the plaatdeploy network for reverse proxy.
            // Using the named network index avoids IP concatenation if the container
            // were ever on multiple networks.
            let ip_output = Command::new("docker")
                .args([
                    "inspect",
                    "-f",
                    r#"{{(index .NetworkSettings.Networks "plaatdeploy").IPAddress}}"#,
                    &container,
                ])
                .output()?;
            let container_ip = String::from_utf8_lossy(&ip_output.stdout)
                .trim()
                .to_string();
            log_buf.push_str(&format!("Container IP: {container_ip}\n"));
            ctx.database.execute(
                "UPDATE projects SET container_ip = ?, updated_at = ? WHERE id = ?",
                (container_ip, Utc::now(), project.id),
            )?;
        }
        BuildType::Unknown => unreachable!(),
    }

    // Update deployment with commit sha from git log
    let sha_output = Command::new("git")
        .args(["-C", &repo_dir, "rev-parse", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| String::new());
    if !sha_output.is_empty() {
        ctx.database.execute(
            "UPDATE deployments SET commit_sha = ? WHERE id = ? AND commit_sha IN ('manual', 'auto-restart')",
            (sha_output, task.deployment_id),
        )?;
    }

    log_buf.push_str("Deploy succeeded.\n");
    Ok(())
}

fn run_cmd(cmd: &mut Command, log_buf: &mut String) -> anyhow::Result<()> {
    let output = cmd.output()?;
    log_buf.push_str(&format!(
        "$ {:?}\n{}{}\n",
        cmd.get_program(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ));
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Command failed with exit code {:?}",
            output.status.code()
        ));
    }
    Ok(())
}

fn project_repo_dir(project: &Project, deploy_path: &str) -> String {
    format!("{deploy_path}/{}/repo", project.name)
}

fn project_work_dir(project: &Project, deploy_path: &str) -> String {
    let repo_dir = project_repo_dir(project, deploy_path);
    if project.base_dir.is_empty() {
        repo_dir
    } else {
        format!("{repo_dir}/{}", project.base_dir)
    }
}

#[derive(Default)]
struct DockerfileConfig {
    exposed_port: Option<u16>,
    volumes: Vec<String>,
}

fn parse_dockerfile_config(dockerfile_path: &str) -> DockerfileConfig {
    let dockerfile_content = std::fs::read_to_string(dockerfile_path).unwrap_or_default();
    let mut config = DockerfileConfig::default();
    let mut volumes = HashSet::new();

    for line in dockerfile_content.lines() {
        let trimmed = line.trim();
        let uppercase = trimmed.to_uppercase();

        if config.exposed_port.is_none() && uppercase.starts_with("EXPOSE ") {
            config.exposed_port = trimmed[7..]
                .trim()
                .split('/')
                .next()
                .and_then(|port| port.parse().ok());
        }

        if uppercase.starts_with("VOLUME ") {
            for volume in parse_volume_instruction(&trimmed[7..]) {
                if volumes.insert(volume.clone()) {
                    config.volumes.push(volume);
                }
            }
        }
    }

    config
}

fn parse_volume_instruction(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    if trimmed.starts_with('[') {
        return serde_json::from_str::<Vec<String>>(trimmed).unwrap_or_default();
    }

    trimmed
        .split_whitespace()
        .map(|part| part.trim_matches('"').trim_matches('\'').to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn volume_mounts(
    project: &Project,
    deploy_path: &str,
    volume_paths: &[String],
) -> anyhow::Result<Vec<String>> {
    let mut mounts = Vec::new();

    for volume_path in volume_paths {
        let sanitized = sanitize_volume_path(volume_path);
        let host_dir = format!("{deploy_path}/{}/volumes/{sanitized}", project.name);
        std::fs::create_dir_all(&host_dir)?;
        mounts.push(format!("{host_dir}:{volume_path}"));
    }

    Ok(mounts)
}

fn sanitize_volume_path(volume_path: &str) -> String {
    let trimmed = volume_path.trim().trim_start_matches('/');
    let normalized = if trimmed.is_empty() { "root" } else { trimmed };
    normalized
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() || matches!(char, '.' | '_' | '-') {
                char
            } else {
                '_'
            }
        })
        .collect()
}

fn start_container(
    project: &Project,
    deploy_path: &str,
    volume_paths: &[String],
    mut log_buf: Option<&mut String>,
) -> anyhow::Result<()> {
    let container = format!("plaatdeploy-{}", project.name);
    let mounts = volume_mounts(project, deploy_path, volume_paths)?;
    let mut args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--name".to_string(),
        container.clone(),
        "--restart".to_string(),
        "unless-stopped".to_string(),
        "--network".to_string(),
        "plaatdeploy".to_string(),
    ];
    for mount in &mounts {
        args.push("-v".to_string());
        args.push(mount.clone());
    }
    args.push(container);

    if let Some(log_buf) = log_buf.as_mut() {
        if !mounts.is_empty() {
            log_buf.push_str(&format!("Mounting volumes: {}\n", mounts.join(", ")));
        }
        run_cmd(Command::new("docker").args(&args), log_buf)
    } else {
        let output = Command::new("docker").args(&args).output()?;
        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Command failed with exit code {:?}",
                output.status.code()
            ))
        }
    }
}
