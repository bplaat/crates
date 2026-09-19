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
pub(crate) struct Deployment {
    pub id: Uuid,
    pub project_id: Uuid,
    pub commit_sha: String,
    pub commit_message: String,
    pub github_deployment_id: Option<i64>,
    pub status: DeploymentStatus,
    pub log: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Deployment {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            project_id: Uuid::nil(),
            commit_sha: String::new(),
            commit_message: String::new(),
            github_deployment_id: None,
            status: DeploymentStatus::Pending,
            log: String::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<Deployment> for api::Deployment {
    fn from(deployment: Deployment) -> Self {
        Self {
            id: deployment.id,
            project_id: deployment.project_id,
            commit_sha: deployment.commit_sha,
            commit_message: deployment.commit_message,
            status: deployment.status.into(),
            log: deployment.log,
            created_at: deployment.created_at,
            updated_at: deployment.updated_at,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromValue, FromEnum)]
#[from_enum(api::DeploymentStatus)]
pub(crate) enum DeploymentStatus {
    Pending = 0,
    Deploying = 1,
    Succeeded = 2,
    Failed = 3,
}
