/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::{FromRow, FromValue};
use chrono::{DateTime, Utc};
use from_enum::FromEnum;
use uuid::Uuid;

use crate::api;

// MARK: User
#[derive(Clone, FromRow)]
pub(crate) struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for User {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            first_name: String::default(),
            last_name: String::default(),
            email: String::default(),
            password: String::default(),
            role: UserRole::Normal,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<User> for api::User {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            role: user.role.into(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

// MARK: UserRole
#[derive(Copy, Clone, FromValue, FromEnum)]
#[from_enum(api::UserRole)]
pub(crate) enum UserRole {
    Normal = 0,
    Admin = 1,
}

// MARK: Policies
pub(crate) mod policies {
    use super::{User, UserRole};

    pub(crate) fn can_index(auth_user: &User) -> bool {
        matches!(auth_user.role, UserRole::Admin)
    }

    pub(crate) fn can_create(auth_user: &User) -> bool {
        matches!(auth_user.role, UserRole::Admin)
    }

    pub(crate) fn can_show(auth_user: &User, user: &User) -> bool {
        match auth_user.role {
            UserRole::Admin => true,
            UserRole::Normal => auth_user.id == user.id,
        }
    }

    pub(crate) fn can_update(auth_user: &User, user: &User) -> bool {
        match auth_user.role {
            UserRole::Admin => true,
            UserRole::Normal => auth_user.id == user.id,
        }
    }

    pub(crate) fn can_delete(auth_user: &User, user: &User) -> bool {
        match auth_user.role {
            UserRole::Admin => true,
            UserRole::Normal => auth_user.id == user.id,
        }
    }
}
