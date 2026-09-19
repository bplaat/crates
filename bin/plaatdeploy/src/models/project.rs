/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsql::{FromRow, FromValue};
use chrono::{DateTime, Utc};
use from_derive::FromEnum;
use uuid::Uuid;

use crate::api;

#[derive(Clone, FromRow)]
pub(crate) struct Project {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub slug: String,
    pub github_repo: String,
    pub github_branch: String,
    pub github_webhook_id: Option<i64>,
    pub webhook_secret: String,
    pub deployment_mode: DeploymentMode,
    pub dockerfile_path: Option<String>,
    pub container_port: Option<i64>,
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
            user_id: Uuid::nil(),
            name: String::new(),
            slug: String::new(),
            github_repo: String::new(),
            github_branch: String::new(),
            github_webhook_id: None,
            webhook_secret: String::new(),
            deployment_mode: DeploymentMode::Static,
            dockerfile_path: None,
            container_port: None,
            status: ProjectStatus::Pending,
            last_deployed_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Project {
    pub(crate) fn into_api(self, deployments_domain: &str) -> api::Project {
        api::Project {
            id: self.id,
            name: self.name,
            slug: self.slug.clone(),
            github_repo: self.github_repo,
            github_branch: self.github_branch,
            deployment_url: format!("https://{}.{}", self.slug, deployments_domain),
            deployment_mode: self.deployment_mode.into(),
            dockerfile_path: self.dockerfile_path,
            container_port: self.container_port,
            status: self.status.into(),
            last_deployed_at: self.last_deployed_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromValue, FromEnum)]
#[from_enum(api::DeploymentMode)]
pub(crate) enum DeploymentMode {
    Static = 0,
    Dockerfile = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromValue, FromEnum)]
#[from_enum(api::ProjectStatus)]
pub(crate) enum ProjectStatus {
    Pending = 0,
    Deploying = 1,
    Running = 2,
    Failed = 3,
}
