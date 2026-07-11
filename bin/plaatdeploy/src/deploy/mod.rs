/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Deploy task queue and runner

use uuid::Uuid;

pub(crate) mod runner;

// MARK: DeployTask
#[derive(Debug, Clone)]
pub(crate) struct DeployTask {
    pub deployment_id: Uuid,
    pub project_id: Uuid,
    pub github_deployment_id: Option<u64>,
}
