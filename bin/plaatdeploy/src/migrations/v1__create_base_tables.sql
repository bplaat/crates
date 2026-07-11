-- Copyright (c) 2026 Bastiaan van der Plaat
-- SPDX-License-Identifier: MIT

CREATE TABLE users (
    id          BLOB PRIMARY KEY,
    first_name  TEXT NOT NULL,
    last_name   TEXT NOT NULL,
    email       TEXT NOT NULL UNIQUE,
    password    TEXT NOT NULL,
    role        INTEGER NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
) STRICT;

CREATE TABLE sessions (
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
) STRICT;
CREATE INDEX idx_sessions_user_id ON sessions (user_id);

CREATE TABLE teams (
    id          BLOB PRIMARY KEY,
    name        TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
) STRICT;

CREATE TABLE team_users (
    id          BLOB PRIMARY KEY,
    team_id     BLOB NOT NULL,
    user_id     BLOB NOT NULL,
    role        INTEGER NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    UNIQUE (team_id, user_id)
) STRICT;
CREATE INDEX idx_team_users_user_id ON team_users (user_id);

CREATE TABLE projects (
    id                  BLOB PRIMARY KEY,
    team_id             BLOB NOT NULL,
    name                TEXT NOT NULL UNIQUE,
    github_repo         TEXT NOT NULL,
    github_branch       TEXT NOT NULL,
    base_dir            TEXT NOT NULL,
    container_port      INTEGER,
    container_ip        TEXT,
    build_type          INTEGER NOT NULL,
    status              INTEGER NOT NULL,
    last_deployed_at    INTEGER,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL
) STRICT;
CREATE INDEX idx_projects_team_id ON projects (team_id);

CREATE TABLE deployments (
    id                    BLOB PRIMARY KEY,
    project_id            BLOB NOT NULL,
    commit_sha            TEXT NOT NULL,
    commit_message        TEXT NOT NULL,
    status                INTEGER NOT NULL,
    log                   TEXT,
    github_deployment_id INTEGER,
    created_at            INTEGER NOT NULL,
    updated_at            INTEGER NOT NULL
) STRICT;
CREATE INDEX idx_deployments_project_id ON deployments (project_id);

CREATE TABLE team_github_tokens (
    id              BLOB PRIMARY KEY,
    team_id         BLOB NOT NULL UNIQUE,
    access_token    TEXT NOT NULL,
    webhook_secret  TEXT NOT NULL,
    account_login   TEXT NOT NULL,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
) STRICT;
