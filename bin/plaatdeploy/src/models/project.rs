/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::{FromRow, FromValue};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::api;

// MARK: Project
#[derive(Clone, FromRow)]
pub(crate) struct Project {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub github_repo: String,
    pub github_branch: String,
    pub base_dir: String,
    pub container_port: Option<i64>,
    pub host_port: Option<i64>,
    pub container_ip: Option<String>,
    pub build_type: BuildType,
    pub status: ProjectStatus,
    pub last_deployed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Project {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            team_id: Uuid::nil(),
            name: String::new(),
            github_repo: String::new(),
            github_branch: "master".to_string(),
            base_dir: String::new(),
            container_port: None,
            host_port: None,
            container_ip: None,
            build_type: BuildType::Unknown,
            status: ProjectStatus::Idle,
            last_deployed_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<Project> for api::Project {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            team_id: project.team_id,
            name: project.name,
            github_repo: project.github_repo,
            github_branch: project.github_branch,
            base_dir: project.base_dir,
            container_port: project.container_port,
            host_port: project.host_port,
            container_ip: project.container_ip,
            build_type: project.build_type.into(),
            status: project.status.into(),
            last_deployed_at: project.last_deployed_at,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}

// MARK: BuildType
#[derive(Copy, Clone, PartialEq, Eq, FromValue)]
pub(crate) enum BuildType {
    Unknown = 0,
    Static = 1,
    Docker = 2,
}

impl From<BuildType> for api::BuildType {
    fn from(t: BuildType) -> Self {
        match t {
            BuildType::Unknown => api::BuildType::Unknown,
            BuildType::Static => api::BuildType::Static,
            BuildType::Docker => api::BuildType::Docker,
        }
    }
}

// MARK: ProjectStatus
#[derive(Copy, Clone, PartialEq, Eq, FromValue)]
pub(crate) enum ProjectStatus {
    Idle = 0,
    Building = 1,
    Running = 2,
    Failed = 3,
}

impl From<ProjectStatus> for api::ProjectStatus {
    fn from(s: ProjectStatus) -> Self {
        match s {
            ProjectStatus::Idle => api::ProjectStatus::Idle,
            ProjectStatus::Building => api::ProjectStatus::Building,
            ProjectStatus::Running => api::ProjectStatus::Running,
            ProjectStatus::Failed => api::ProjectStatus::Failed,
        }
    }
}
