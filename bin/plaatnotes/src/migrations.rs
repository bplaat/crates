/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::Connection;

use crate::context::DatabaseHelpers;

pub(crate) fn database_migrate(database: &Connection) {
    // Ensure migrations tracking table exists
    database
        .execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version     INTEGER PRIMARY KEY,
                applied_at  INTEGER NOT NULL
            ) STRICT",
            (),
        )
        .expect("Database error");

    let applied_version = database
        .query::<i64>(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            (),
        )
        .expect("Database error")
        .next()
        .map(|r| r.expect("Database error"))
        .unwrap_or(0);

    // Migration 1: create base tables
    if applied_version < 1 {
        log::info!("Applying database migration 1: create base tables");

        database
            .execute(
                "CREATE TABLE users(
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
            )
            .expect("Database error");

        database
            .execute(
                "CREATE TABLE sessions(
                id BLOB PRIMARY KEY,
                user_id BLOB NOT NULL,
                token TEXT NOT NULL,
                ip_address TEXT NOT NULL,
                ip_latitude REAL,
                ip_longitude REAL,
                ip_country TEXT,
                ip_city TEXT,
                client_name TEXT,
                client_version TEXT,
                client_os TEXT,
                expires_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            ) STRICT",
                (),
            )
            .expect("Database error");

        database
            .execute(
                "CREATE TABLE notes(
                id BLOB PRIMARY KEY,
                user_id BLOB NOT NULL,
                title TEXT NULL,
                body TEXT NOT NULL,
                is_pinned INTEGER NOT NULL,
                is_archived INTEGER NOT NULL,
                is_trashed INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            ) STRICT",
                (),
            )
            .expect("Database error");

        #[cfg(not(test))]
        {
            use pbkdf2::password_hash;

            use crate::context::DatabaseHelpers;
            use crate::models::{User, UserRole};

            let user = User {
                first_name: "Admin".to_string(),
                last_name: "Admin".to_string(),
                email: "admin@example.com".to_string(),
                password: password_hash("admin123"),
                role: UserRole::Admin,
                ..Default::default()
            };
            database.insert_user(user);
        }

        database
            .execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (1, unixepoch())",
                (),
            )
            .expect("Database error");
    }

    // Migration 2: add position column to notes
    if applied_version < 2 {
        log::info!("Applying database migration 2: add position column to notes");

        database
            .execute(
                "ALTER TABLE notes ADD COLUMN position INTEGER NOT NULL DEFAULT 0",
                (),
            )
            .expect("Database error");

        database
            .execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (2, unixepoch())",
                (),
            )
            .expect("Database error");
    }

    // Migration 3: create notes FTS tables
    if applied_version < 3 {
        log::info!("Applying database migration 3: create notes FTS tables");

        database.create_fts_tables("notes", &["title", "body"]);
        database
            .execute(
                "INSERT INTO notes_fts(title, body, id) SELECT title, body, id FROM notes",
                (),
            )
            .expect("Database error");

        database.create_fts_tables("users", &["first_name", "last_name", "email"]);
        database
            .execute(
                "INSERT INTO users_fts(first_name, last_name, email, id) SELECT first_name, last_name, email, id FROM users",
                (),
            )
            .expect("Database error");

        database
            .execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (3, unixepoch())",
                (),
            )
            .expect("Database error");
    }
}
