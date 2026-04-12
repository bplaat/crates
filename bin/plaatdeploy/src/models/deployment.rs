/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::{FromRow, FromValue};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::api;

// MARK: Deployment
#[derive(Clone, FromRow)]
pub(crate) struct Deployment {
    pub id: Uuid,
    pub project_id: Uuid,
    pub commit_sha: String,
    pub commit_message: String,
    pub status: DeploymentStatus,
    pub log: Option<String>,
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
            status: DeploymentStatus::Pending,
            log: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<Deployment> for api::Deployment {
    fn from(d: Deployment) -> Self {
        Self {
            id: d.id,
            project_id: d.project_id,
            commit_sha: d.commit_sha,
            commit_message: d.commit_message,
            status: d.status.into(),
            log: d.log,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

// MARK: DeploymentStatus
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromValue)]
pub(crate) enum DeploymentStatus {
    Pending = 0,
    Building = 1,
    Succeeded = 2,
    Failed = 3,
}

impl From<DeploymentStatus> for api::DeploymentStatus {
    fn from(s: DeploymentStatus) -> Self {
        match s {
            DeploymentStatus::Pending => api::DeploymentStatus::Pending,
            DeploymentStatus::Building => api::DeploymentStatus::Building,
            DeploymentStatus::Succeeded => api::DeploymentStatus::Succeeded,
            DeploymentStatus::Failed => api::DeploymentStatus::Failed,
        }
    }
}
