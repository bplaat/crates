/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::{FromRow, FromValue};
use chrono::{DateTime, Utc};
use from_derive::{FromEnum, FromStruct};
use uuid::Uuid;

use crate::api;

// MARK: Person
#[derive(Clone, FromRow, FromStruct)]
#[from_struct(api::Person)]
pub(crate) struct Person {
    pub id: Uuid,
    pub name: String,
    #[sqlite(rename = "age")]
    pub age_in_years: i64,
    pub relation: Relation,
    pub created_at: DateTime<Utc>,
}

impl Default for Person {
    fn default() -> Self {
        Self {
            id: Uuid::now_v7(),
            name: String::default(),
            age_in_years: 0,
            relation: Relation::Me,
            created_at: Utc::now(),
        }
    }
}

// MARK: Relation
#[derive(Copy, Clone, FromEnum, FromValue)]
#[from_enum(api::Relation)]
pub(crate) enum Relation {
    Me = 0,
    Brother = 1,
    Sister = 2,
}
