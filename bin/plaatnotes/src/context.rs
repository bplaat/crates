/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::Path;

use bsqlite::{Connection, OpenMode};
use const_format::formatcp;

use crate::models::{Note, User};

#[derive(Clone)]
pub(crate) struct Context {
    pub database: Connection,
}

impl Context {
    pub(crate) fn with_database(path: impl AsRef<Path>) -> Self {
        let database =
            Connection::open(path.as_ref(), OpenMode::ReadWrite).expect("Can't open database");
        database.enable_wal_logging();
        database.apply_various_performance_settings();
        database_create_tables(&database);
        Self { database }
    }

    #[cfg(test)]
    pub(crate) fn with_test_database() -> Self {
        let database = Connection::open_memory().expect("Can't open in-memory database");
        database_create_tables(&database);
        Self { database }
    }
}

// MARK: Database
pub(crate) trait DatabaseHelpers {
    fn insert_note(&self, note: Note);
    fn insert_user(&self, user: User);
}

impl DatabaseHelpers for Connection {
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
}

fn database_create_tables(database: &Connection) {
    database.execute(
        "CREATE TABLE IF NOT EXISTS notes(
            id BLOB PRIMARY KEY,
            body TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        ) STRICT",
        (),
    );

    database.execute(
        "CREATE TABLE IF NOT EXISTS users(
            id BLOB PRIMARY KEY,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        ) STRICT",
        (),
    );
}
