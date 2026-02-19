/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::FromRow;
use chrono::{DateTime, Utc};
use from_derive::FromStruct;
use uuid::Uuid;

use crate::api;
use crate::models::user::{User, UserRole};
#[derive(Clone, FromRow, FromStruct)]
#[from_struct(api::Note)]
pub(crate) struct Note {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: Option<String>,
    pub body: String,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub is_trashed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Note {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            user_id: Uuid::nil(),
            title: None,
            body: String::default(),
            is_pinned: false,
            is_archived: false,
            is_trashed: false,
            created_at: now,
            updated_at: now,
        }
    }
}

// MARK: Policies
pub(crate) mod policies {
    use super::*;

    pub(crate) fn can_index(_auth_user: &User) -> bool {
        // Both admin and normal users can index (admins see all, normal users see their own)
        true
    }

    pub(crate) fn can_create(_auth_user: &User) -> bool {
        // Both admin and normal users can create notes
        true
    }

    pub(crate) fn can_show(auth_user: &User, note: &Note) -> bool {
        match auth_user.role {
            UserRole::Admin => true,
            UserRole::Normal => auth_user.id == note.user_id,
        }
    }

    pub(crate) fn can_update(auth_user: &User, note: &Note) -> bool {
        match auth_user.role {
            UserRole::Admin => true,
            UserRole::Normal => auth_user.id == note.user_id,
        }
    }

    pub(crate) fn can_delete(auth_user: &User, note: &Note) -> bool {
        match auth_user.role {
            UserRole::Admin => true,
            UserRole::Normal => auth_user.id == note.user_id,
        }
    }
}
