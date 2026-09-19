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

// MARK: User
#[derive(Clone, FromRow)]
pub(crate) struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub theme: UserTheme,
    pub language: String,
    pub role: UserRole,
    pub github_token: Option<String>,
    pub github_login: Option<String>,
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
            theme: UserTheme::System,
            language: "en".to_string(),
            role: UserRole::Normal,
            github_token: None,
            github_login: None,
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
            theme: user.theme.into(),
            language: user.language,
            role: user.role.into(),
            github_login: user.github_login,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

// MARK: UserTheme
#[derive(Copy, Clone, PartialEq, Eq, FromValue, FromEnum)]
#[from_enum(api::UserTheme)]
pub(crate) enum UserTheme {
    System = 0,
    Light = 1,
    Dark = 2,
}

// MARK: UserRole
#[derive(Copy, Clone, PartialEq, Eq, FromValue, FromEnum)]
#[from_enum(api::UserRole)]
pub(crate) enum UserRole {
    Normal = 0,
    Admin = 1,
}
