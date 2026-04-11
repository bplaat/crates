/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use bsqlite::Connection;

use crate::context::DatabaseHelpers;

pub(crate) fn database_migrate(database: &Connection) -> Result<()> {
    // Ensure migrations tracking table exists
    database.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version     INTEGER PRIMARY KEY,
            applied_at  INTEGER NOT NULL
        ) STRICT",
        (),
    )?;

    let applied_version = database
        .query::<i64>(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            (),
        )?
        .next()
        .transpose()?
        .unwrap_or(0);

    // Migration 1: create base tables
    if applied_version < 1 {
        log::info!("Applying database migration 1: create base tables");

        database.execute(
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
        )?;

        database.execute(
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
        )?;

        database.execute(
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
        )?;

        #[cfg(not(test))]
        {
            use pbkdf2::password_hash;

            use crate::models::{User, UserRole};

            let admin_email =
                std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());
            let admin_password =
                std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "Password123!".to_string());

            let user = User {
                first_name: "Admin".to_string(),
                last_name: "Admin".to_string(),
                email: admin_email,
                password: password_hash(&admin_password),
                role: UserRole::Admin,
                ..Default::default()
            };
            database.insert_user(user)?;

            log::info!("Admin account created");
        }

        database.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (1, unixepoch())",
            (),
        )?;
    }

    // Migration 2: add position column to notes
    if applied_version < 2 {
        log::info!("Applying database migration 2: add position column to notes");

        database.execute(
            "ALTER TABLE notes ADD COLUMN position INTEGER NOT NULL DEFAULT 0",
            (),
        )?;

        database.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (2, unixepoch())",
            (),
        )?;
    }

    // Migration 3: create notes, users FTS tables
    if applied_version < 3 {
        log::info!("Applying database migration 3: create notes, users FTS tables");

        database.create_fts_tables("notes", &["title", "body"])?;
        database.execute(
            "INSERT INTO notes_fts(title, body, id) SELECT title, body, id FROM notes",
            (),
        )?;

        database.create_fts_tables("users", &["first_name", "last_name", "email"])?;
        database.execute(
            "INSERT INTO users_fts(first_name, last_name, email, id) SELECT first_name, last_name, email, id FROM users",
            (),
        )?;

        database.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (3, unixepoch())",
            (),
        )?;
    }

    // Migration 4: create sessions FTS tables
    if applied_version < 4 {
        log::info!("Applying database migration 4: create sessions FTS tables");

        database.create_fts_tables(
            "sessions",
            &[
                "ip_address",
                "ip_country",
                "ip_city",
                "client_name",
                "client_os",
            ],
        )?;
        database.execute(
            "INSERT INTO sessions_fts(ip_address, ip_country, ip_city, client_name, client_os, id) SELECT ip_address, ip_country, ip_city, client_name, client_os, id FROM sessions",
            (),
        )?;

        database.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (4, unixepoch())",
            (),
        )?;
    }

    Ok(())
}
