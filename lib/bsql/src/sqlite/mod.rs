/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

mod connection;
mod migration;
mod statement;
mod utils;

pub(crate) use connection::Connection;
pub use migration::{Migration, MigrationError};
pub(crate) use statement::Prepared;
pub use utils::preprocess_fts_query;
