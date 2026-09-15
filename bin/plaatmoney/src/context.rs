/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::Result;
use bsql::{Connection, PoolOptions, SqliteMode, run_migrations};
use small_http::Client;

#[derive(Clone)]
pub(crate) struct Context {
    pub(crate) client: Arc<Mutex<Client>>,
    pub(crate) cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    pub(crate) database: Connection,
}

impl Context {
    pub(crate) fn new(database_path: String) -> Result<Self> {
        let database =
            Connection::open_sqlite(database_path, SqliteMode::ReadWrite, PoolOptions::default())?;
        database.execute_script("PRAGMA foreign_keys = ON")?;
        database.enable_wal_logging()?;
        run_migrations!(database, "src/migrations")?;
        Ok(Self {
            client: Arc::new(Mutex::new(Client::new().header(
                "User-Agent",
                "plaatmoney/0.1 (self-hosted personal finance dashboard)",
            ))),
            cache: Arc::new(Mutex::new(HashMap::new())),
            database,
        })
    }
}

#[derive(Clone)]
pub(crate) struct CacheEntry {
    pub(crate) stored_at: Instant,
    pub(crate) body: Vec<u8>,
}
