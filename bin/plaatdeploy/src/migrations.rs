/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Database migrations

use anyhow::Result;
use bsqlite::Connection;

pub(crate) fn database_migrate(database: &Connection) -> Result<()> {
    database.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version     INTEGER PRIMARY KEY,
            applied_at  INTEGER NOT NULL
        ) STRICT",
        (),
    )?;

    let applied_version = database.query_some::<i64>(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        (),
    )?;

    if applied_version >= 1 {
        return Ok(());
    }

    log::info!("Applying database migration 1: create baseline schema");

    database.execute(
        "CREATE TABLE users (
            id          BLOB PRIMARY KEY,
            first_name  TEXT NOT NULL,
            last_name   TEXT NOT NULL,
            email       TEXT NOT NULL UNIQUE,
            password    TEXT NOT NULL,
            role        INTEGER NOT NULL,
            created_at  INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL
        ) STRICT",
        (),
    )?;

    database.execute(
        "CREATE TABLE sessions (
            id              BLOB PRIMARY KEY,
            user_id         BLOB NOT NULL,
            token           TEXT NOT NULL UNIQUE,
            ip_address      TEXT NOT NULL,
            ip_latitude     REAL,
            ip_longitude    REAL,
            ip_country      TEXT,
            ip_city         TEXT,
            client_name     TEXT,
            client_version  TEXT,
            client_os       TEXT,
            expires_at      INTEGER NOT NULL,
            created_at      INTEGER NOT NULL,
            updated_at      INTEGER NOT NULL
        ) STRICT",
        (),
    )?;

    database.execute(
        "CREATE TABLE teams (
            id          BLOB PRIMARY KEY,
            name        TEXT NOT NULL,
            created_at  INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL
        ) STRICT",
        (),
    )?;

    database.execute(
        "CREATE TABLE team_users (
            id          BLOB PRIMARY KEY,
            team_id     BLOB NOT NULL,
            user_id     BLOB NOT NULL,
            role        INTEGER NOT NULL,
            created_at  INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL,
            UNIQUE (team_id, user_id)
        ) STRICT",
        (),
    )?;
    database.execute(
        "CREATE INDEX idx_team_users_user_id ON team_users (user_id)",
        (),
    )?;

    database.execute(
        "CREATE TABLE projects (
            id                  BLOB PRIMARY KEY,
            team_id             BLOB NOT NULL,
            name                TEXT NOT NULL UNIQUE,
            github_repo         TEXT NOT NULL,
            github_branch       TEXT NOT NULL,
            base_dir            TEXT NOT NULL,
            container_port      INTEGER,
            host_port           INTEGER,
            container_ip        TEXT,
            build_type          INTEGER NOT NULL,
            status              INTEGER NOT NULL,
            last_deployed_at    INTEGER,
            created_at          INTEGER NOT NULL,
            updated_at          INTEGER NOT NULL
        ) STRICT",
        (),
    )?;
    database.execute(
        "CREATE INDEX idx_projects_team_id ON projects (team_id)",
        (),
    )?;

    database.execute(
        "CREATE TABLE deployments (
            id              BLOB PRIMARY KEY,
            project_id      BLOB NOT NULL,
            commit_sha      TEXT NOT NULL,
            commit_message  TEXT NOT NULL,
            status          INTEGER NOT NULL,
            log             TEXT,
            created_at      INTEGER NOT NULL,
            updated_at      INTEGER NOT NULL
        ) STRICT",
        (),
    )?;

    database.execute(
        "CREATE TABLE team_github_connections (
            id               BLOB PRIMARY KEY,
            team_id          BLOB NOT NULL UNIQUE,
            app_id           TEXT,
            app_private_key  TEXT,
            webhook_secret   TEXT,
            app_slug         TEXT,
            setup_state      TEXT UNIQUE,
            installation_id  INTEGER UNIQUE,
            account_login    TEXT,
            account_type     TEXT,
            created_at       INTEGER NOT NULL,
            updated_at       INTEGER NOT NULL
        ) STRICT",
        (),
    )?;
    database.execute(
        "CREATE INDEX idx_team_github_connections_installation_id ON team_github_connections (installation_id)",
        (),
    )?;

    database.execute(
        "INSERT INTO schema_migrations (version, applied_at) VALUES (1, strftime('%s', 'now'))",
        (),
    )?;

    Ok(())
}
