/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::api;

// MARK: TeamGitHubConnection
#[derive(Clone, FromRow)]
pub(crate) struct TeamGitHubConnection {
    pub id: Uuid,
    pub team_id: Uuid,
    pub app_id: Option<String>,
    pub app_private_key: Option<String>,
    pub webhook_secret: Option<String>,
    pub app_slug: Option<String>,
    pub setup_state: Option<String>,
    pub installation_id: Option<i64>,
    pub account_login: Option<String>,
    pub account_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for TeamGitHubConnection {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            team_id: Uuid::nil(),
            app_id: None,
            app_private_key: None,
            webhook_secret: None,
            app_slug: None,
            setup_state: None,
            installation_id: None,
            account_login: None,
            account_type: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<TeamGitHubConnection> for api::TeamGithubConnection {
    fn from(connection: TeamGitHubConnection) -> Self {
        Self {
            installation_id: connection.installation_id.unwrap_or_default(),
            account_login: connection.account_login.unwrap_or_default(),
            account_type: connection.account_type,
            created_at: connection.created_at,
            updated_at: connection.updated_at,
        }
    }
}
