/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::Result;
use bsqlite::{Connection, OpenMode};
use const_format::formatcp;
use uuid::Uuid;

use crate::migrations::database_migrate;
use crate::models::{Note, Session, User};

// MARK: Context
#[derive(Clone)]
pub(crate) struct Context {
    pub server_origin: String,
    pub database: Connection,
    pub auth_session: Option<Session>,
    pub auth_user: Option<User>,
    pub update_target_user_id: Option<Uuid>,
    #[allow(dead_code)]
    pub login_attempts: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
}

impl Context {
    #[allow(dead_code)]
    pub(crate) fn with_database(path: impl AsRef<Path>, server_origin: String) -> Result<Self> {
        log::info!("Using database at {}", path.as_ref().display());
        let database = Connection::open(path.as_ref(), OpenMode::ReadWrite)?;
        database.enable_wal_logging()?;
        database.apply_various_performance_settings()?;
        database_migrate(&database)?;
        Ok(Self {
            server_origin,
            database,
            auth_session: None,
            auth_user: None,
            update_target_user_id: None,
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    #[cfg(any(test, feature = "test-e2e"))]
    pub(crate) fn with_test_database() -> Result<Self> {
        let database = Connection::open_memory()?;
        database_migrate(&database)?;
        Ok(Self {
            server_origin: "*".to_string(),
            database,
            auth_session: None,
            auth_user: None,
            update_target_user_id: None,
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

// MARK: Database helpers
pub(crate) trait DatabaseHelpers {
    fn create_fts_tables(&self, table: &str, columns: &[&str]) -> Result<()>;
    fn insert_user(&self, user: User) -> Result<()>;
    fn insert_session(&self, session: Session) -> Result<()>;
    fn insert_note(&self, note: Note) -> Result<()>;
}

impl DatabaseHelpers for Connection {
    fn create_fts_tables(&self, table: &str, columns: &[&str]) -> Result<()> {
        let cols = columns.join(", ");
        let new_cols = columns
            .iter()
            .map(|c| format!("new.{c}"))
            .collect::<Vec<_>>()
            .join(", ");
        let set_cols = columns
            .iter()
            .map(|c| format!("{c} = new.{c}"))
            .collect::<Vec<_>>()
            .join(", ");

        self.execute(
            format!(
                "CREATE VIRTUAL TABLE IF NOT EXISTS {table}_fts USING fts5({cols}, id UNINDEXED)"
            ),
            (),
        )?;

        self.execute(
            format!(
                "CREATE TRIGGER IF NOT EXISTS {table}_ai AFTER INSERT ON {table} BEGIN
                    INSERT INTO {table}_fts({cols}, id) VALUES ({new_cols}, new.id);
                END"
            ),
            (),
        )?;

        self.execute(
            format!(
                "CREATE TRIGGER IF NOT EXISTS {table}_au AFTER UPDATE ON {table} BEGIN
                    UPDATE {table}_fts SET {set_cols} WHERE id = old.id;
                END"
            ),
            (),
        )?;

        self.execute(
            format!(
                "CREATE TRIGGER IF NOT EXISTS {table}_ad BEFORE DELETE ON {table} BEGIN
                    DELETE FROM {table}_fts WHERE id = old.id;
                END"
            ),
            (),
        )?;

        Ok(())
    }

    fn insert_user(&self, user: User) -> Result<()> {
        self.execute(
            formatcp!(
                "INSERT INTO users ({}) VALUES ({})",
                User::columns(),
                User::values()
            ),
            user,
        )?;
        Ok(())
    }

    fn insert_session(&self, session: Session) -> Result<()> {
        self.execute(
            formatcp!(
                "INSERT INTO sessions ({}) VALUES ({})",
                Session::columns(),
                Session::values()
            ),
            session,
        )?;
        Ok(())
    }

    fn insert_note(&self, note: Note) -> Result<()> {
        self.execute(
            formatcp!(
                "INSERT INTO notes ({}) VALUES ({})",
                Note::columns(),
                Note::values()
            ),
            note,
        )?;
        Ok(())
    }
}
