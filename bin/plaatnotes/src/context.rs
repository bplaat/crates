/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::Path;

use bsqlite::{Connection, OpenMode};
use const_format::formatcp;

use crate::models::{Note, Session, User};

// MARK: Context
#[derive(Clone)]
pub(crate) struct Context {
    pub database: Connection,
    pub auth_session: Option<Session>,
    pub auth_user: Option<User>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            database: Connection::open_memory().expect("Can't open in-memory database"),
            auth_session: None,
            auth_user: None,
        }
    }
}

impl Context {
    pub(crate) fn with_database(path: impl AsRef<Path>) -> Self {
        let database =
            Connection::open(path.as_ref(), OpenMode::ReadWrite).expect("Can't open database");
        database.enable_wal_logging();
        database.apply_various_performance_settings();
        database_create_tables(&database);
        Self {
            database,
            auth_session: None,
            auth_user: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_test_database() -> Self {
        let database = Connection::open_memory().expect("Can't open in-memory database");
        database_create_tables(&database);
        Self {
            database,
            auth_session: None,
            auth_user: None,
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
        );
    }

    fn insert_session(&self, session: Session) {
        self.execute(
            formatcp!(
                "INSERT INTO sessions ({}) VALUES ({})",
                Session::columns(),
                Session::values()
            ),
            session,
        );
    }

    fn insert_note(&self, note: Note) {
        self.execute(
            formatcp!(
                "INSERT INTO notes ({}) VALUES ({})",
                Note::columns(),
                Note::values()
            ),
            note,
        );
    }
}

fn database_create_tables(database: &Connection) {
    database.execute(
        "CREATE TABLE IF NOT EXISTS users(
            id BLOB PRIMARY KEY,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            theme INTEGER NOT NULL,
            language TEXT NOT NULL,
            role INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        ) STRICT",
        (),
    );

    database.execute(
        "CREATE TABLE IF NOT EXISTS sessions(
            id BLOB PRIMARY KEY,
            user_id BLOB NOT NULL,
            token TEXT NOT NULL,
            expires_at INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        ) STRICT",
        (),
    );

    database.execute(
        "CREATE TABLE IF NOT EXISTS notes(
            id BLOB PRIMARY KEY,
            user_id BLOB NOT NULL,
            body TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        ) STRICT",
        (),
    );
}
