/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

pub(crate) use crate::sqlite::connection::Connection;
pub use crate::sqlite::connection::SqliteMode;
pub use crate::sqlite::migration::{Migration, MigrationError};
pub(crate) use crate::sqlite::statement::Prepared;
pub use crate::sqlite::utils::preprocess_fts_query;

mod connection;
mod migration;
mod statement;
mod utils;
