/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use serde::Deserialize;
use validate::Validate;

pub(crate) use self::note::Note;

mod note;

// MARK: Index query
#[derive(Deserialize, Validate)]
#[serde(default)]
pub(crate) struct IndexQuery {
    #[serde(rename = "q")]
    pub query: String,
    #[validate(range(min = 1))]
    pub page: i64,
    #[validate(range(min = 1, max = 50))]
    pub limit: i64,
}

impl Default for IndexQuery {
    fn default() -> Self {
        Self {
            query: "".to_string(),
            page: 1,
            limit: 20,
        }
    }
}
