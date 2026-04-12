/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::{FromRow, FromValue};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::api;

// MARK: Team
#[derive(Clone, FromRow)]
pub(crate) struct Team {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Team {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            name: String::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<Team> for api::Team {
    fn from(team: Team) -> Self {
        Self {
            id: team.id,
            name: team.name,
            created_at: team.created_at,
            updated_at: team.updated_at,
        }
    }
}

#[derive(Clone, FromRow)]
pub(crate) struct TeamUser {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: TeamUserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for TeamUser {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            team_id: Uuid::nil(),
            user_id: Uuid::nil(),
            role: TeamUserRole::Member,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Clone, FromRow)]
pub(crate) struct TeamUserRow {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub role: TeamUserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TeamUserRow> for api::TeamUser {
    fn from(team_user: TeamUserRow) -> Self {
        Self {
            id: team_user.id,
            team_id: team_user.team_id,
            user_id: team_user.user_id,
            first_name: team_user.first_name,
            last_name: team_user.last_name,
            email: team_user.email,
            role: team_user.role.into(),
            created_at: team_user.created_at,
            updated_at: team_user.updated_at,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, FromValue)]
pub(crate) enum TeamUserRole {
    Member = 0,
    Owner = 1,
}

impl From<TeamUserRole> for api::TeamUserRole {
    fn from(role: TeamUserRole) -> Self {
        match role {
            TeamUserRole::Member => api::TeamUserRole::Member,
            TeamUserRole::Owner => api::TeamUserRole::Owner,
        }
    }
}

impl From<api::TeamUserRole> for TeamUserRole {
    fn from(role: api::TeamUserRole) -> Self {
        match role {
            api::TeamUserRole::Member => TeamUserRole::Member,
            api::TeamUserRole::Owner => TeamUserRole::Owner,
        }
    }
}
