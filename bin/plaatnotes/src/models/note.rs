/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::api;

#[derive(Clone, FromRow)]
pub(crate) struct Note {
    pub id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Note {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            body: String::default(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<Note> for api::Note {
    fn from(note: Note) -> Self {
        Self {
            id: note.id,
            body: note.body,
            created_at: note.created_at,
            updated_at: note.updated_at,
        }
    }
}
