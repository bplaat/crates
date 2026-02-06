/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::api;
use crate::models::{User, UserRole};

#[derive(Clone, FromRow)]
pub(crate) struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Session {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            user_id: Uuid::nil(),
            token: String::default(),
            expires_at: now,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<Session> for api::Session {
    fn from(user: Session) -> Self {
        Self {
            id: user.id,
            user_id: user.user_id,
            token: user.token,
            expires_at: user.expires_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

// MARK: Policies
pub(crate) mod policies {
    use super::*;

    pub(crate) fn can_index(auth_user: &User) -> bool {
        matches!(auth_user.role, UserRole::Admin)
    }

    pub(crate) fn can_show(auth_user: &User, session: &Session) -> bool {
        match auth_user.role {
            UserRole::Admin => true,
            UserRole::Normal => auth_user.id == session.user_id,
        }
    }

    pub(crate) fn can_delete(auth_user: &User, session: &Session) -> bool {
        match auth_user.role {
            UserRole::Admin => true,
            UserRole::Normal => auth_user.id == session.user_id,
        }
    }
}
