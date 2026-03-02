/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::Path;

use bsqlite::{Connection, OpenMode};
use const_format::formatcp;
use uuid::Uuid;

use crate::migrations::database_migrate;
use crate::models::{Note, Session, User};

// MARK: Context
#[derive(Clone)]
pub(crate) struct Context {
    pub database: Connection,
    pub auth_session: Option<Session>,
    pub auth_user: Option<User>,
    pub update_target_user_id: Option<Uuid>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            database: Connection::open_memory().expect("Can't open in-memory database"),
            auth_session: None,
            auth_user: None,
            update_target_user_id: None,
        }
    }
}

impl Context {
    pub(crate) fn with_database(path: impl AsRef<Path>) -> Self {
        log::info!("Using database at {}", path.as_ref().display());
        let database =
            Connection::open(path.as_ref(), OpenMode::ReadWrite).expect("Can't open database");
        database.enable_wal_logging().expect("Database error");
        database
            .apply_various_performance_settings()
            .expect("Database error");
        database_migrate(&database);
        Self {
            database,
            auth_session: None,
            auth_user: None,
            update_target_user_id: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_test_database() -> Self {
        let database = Connection::open_memory().expect("Can't open in-memory database");
        database_migrate(&database);
        Self {
            database,
            auth_session: None,
            auth_user: None,
            update_target_user_id: None,
        }
    }
}

// MARK: Database helpers
pub(crate) trait DatabaseHelpers {
    fn insert_user(&self, user: User);
    fn insert_session(&self, session: Session);
    fn insert_note(&self, note: Note);
}

impl DatabaseHelpers for Connection {
    fn insert_user(&self, user: User) {
        self.execute(
            formatcp!(
                "INSERT INTO users ({}) VALUES ({})",
                User::columns(),
                User::values()
            ),
            user,
        )
        .expect("Database error");
    }

    fn insert_session(&self, session: Session) {
        self.execute(
            formatcp!(
                "INSERT INTO sessions ({}) VALUES ({})",
                Session::columns(),
                Session::values()
            ),
            session,
        )
        .expect("Database error");
    }

    fn insert_note(&self, note: Note) {
        self.execute(
            formatcp!(
                "INSERT INTO notes ({}) VALUES ({})",
                Note::columns(),
                Note::values()
            ),
            note,
        )
        .expect("Database error");
    }
}
