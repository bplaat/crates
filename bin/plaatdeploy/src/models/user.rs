/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::{FromRow, FromValue};
use chrono::{DateTime, Utc};
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
            first_name: String::new(),
            last_name: String::new(),
            email: String::new(),
            password: String::new(),
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
#[derive(Copy, Clone, PartialEq, Eq, FromValue)]
pub(crate) enum UserRole {
    Normal = 0,
    Admin = 1,
}

impl From<UserRole> for api::UserRole {
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::Normal => api::UserRole::Normal,
            UserRole::Admin => api::UserRole::Admin,
        }
    }
}

impl From<api::UserRole> for UserRole {
    fn from(role: api::UserRole) -> Self {
        match role {
            api::UserRole::Normal => UserRole::Normal,
            api::UserRole::Admin => UserRole::Admin,
        }
    }
}
