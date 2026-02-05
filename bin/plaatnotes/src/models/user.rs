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
pub(crate) struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
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
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
