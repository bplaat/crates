-- Copyright (c) 2026 Bastiaan van der Plaat
-- SPDX-License-Identifier: MIT

CREATE TABLE users (
    id              BLOB PRIMARY KEY,
    first_name      TEXT NOT NULL,
    last_name       TEXT NOT NULL,
    email           TEXT NOT NULL UNIQUE,
    password        TEXT NOT NULL,
    theme           INTEGER NOT NULL DEFAULT 0,
    language        TEXT NOT NULL DEFAULT 'en',
    role            INTEGER NOT NULL,
    github_token    TEXT,
    github_login    TEXT,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
) STRICT;

CREATE TABLE sessions (
    id              BLOB PRIMARY KEY,
    user_id         BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
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
) STRICT;
CREATE INDEX idx_sessions_user_id ON sessions (user_id);

CREATE TABLE projects (
    id                  BLOB PRIMARY KEY,
    user_id             BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name                TEXT NOT NULL,
    slug                TEXT NOT NULL UNIQUE,
    github_repo         TEXT NOT NULL,
    github_branch       TEXT NOT NULL,
    github_webhook_id   INTEGER,
    webhook_secret      TEXT NOT NULL,
    deployment_mode     INTEGER NOT NULL,
    dockerfile_path     TEXT,
    container_port      INTEGER,
    status              INTEGER NOT NULL,
    last_deployed_at    INTEGER,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL,
    UNIQUE (user_id, github_repo)
) STRICT;
CREATE INDEX idx_projects_user_id ON projects (user_id);

CREATE TABLE deployments (
    id                      BLOB PRIMARY KEY,
    project_id              BLOB NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    commit_sha              TEXT NOT NULL,
    commit_message          TEXT NOT NULL,
    github_deployment_id    INTEGER,
    status                  INTEGER NOT NULL,
    log                     TEXT NOT NULL,
    created_at              INTEGER NOT NULL,
    updated_at              INTEGER NOT NULL
) STRICT;
CREATE INDEX idx_deployments_project_id ON deployments (project_id, created_at DESC);
