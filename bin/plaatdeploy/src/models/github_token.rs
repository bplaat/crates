/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::api;

#[derive(Clone, FromRow)]
pub(crate) struct TeamGitHubToken {
    pub id: Uuid,
    pub team_id: Uuid,
    pub access_token: String,
    pub webhook_secret: String,
    pub account_login: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for TeamGitHubToken {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            team_id: Uuid::nil(),
            access_token: String::new(),
            webhook_secret: String::new(),
            account_login: String::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<TeamGitHubToken> for api::TeamGithubConnection {
    fn from(token: TeamGitHubToken) -> Self {
        Self {
            account_login: token.account_login,
            created_at: token.created_at,
            updated_at: token.updated_at,
        }
    }
}
