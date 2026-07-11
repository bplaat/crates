/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Application context

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use anyhow::Result;
use bsqlite::{Connection, OpenMode, run_migrations};
use chrono::Utc;
use const_format::formatcp;
use uuid::Uuid;

use crate::deploy::DeployTask;
use crate::models::{
    Deployment, Project, Session, Team, TeamGitHubToken, TeamUser, TeamUserRole, User, UserRole,
};
use crate::utils::password_hash;

fn database_seed(database: &Connection) -> Result<()> {
    if database.query_some::<i64>("SELECT COUNT(id) FROM users", ())? == 0 {
        let admin_email =
            env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());
        let admin_password =
            env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "Password123!".to_string());

        database.create_user_with_default_team(User {
            first_name: "Admin".to_string(),
            last_name: "Admin".to_string(),
            email: admin_email,
            password: password_hash(&admin_password),
            role: UserRole::Admin,
            ..Default::default()
        })?;
        log::info!("Admin account created");
    }
    Ok(())
}

// MARK: Context
#[derive(Clone)]
pub(crate) struct Context {
    pub server_origin: String,
    pub server_host: String,
    pub deployments_origin: String,
    pub deployments_host: String,
    pub deploy_path: String,
    pub database: Connection,
    pub auth_session: Option<Session>,
    pub auth_user: Option<User>,
    pub login_attempts: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    pub maxminddb_reader: Arc<OnceLock<maxminddb::Reader<Vec<u8>>>>,
    pub deploy_tx: Sender<DeployTask>,
}

impl Context {
    fn server_host_from_origin(server_origin: &str) -> String {
        server_origin
            .trim()
            .strip_prefix("https://")
            .or_else(|| server_origin.trim().strip_prefix("http://"))
            .unwrap_or(server_origin.trim())
            .trim_end_matches('/')
            .split('/')
            .next()
            .unwrap_or("localhost")
            .to_string()
    }

    pub(crate) fn default_deployments_origin(server_origin: &str) -> String {
        let scheme = if server_origin.trim().starts_with("http://") {
            "http"
        } else {
            "https"
        };
        format!(
            "{scheme}://*.{}",
            Self::server_host_from_origin(server_origin)
        )
    }

    pub(crate) fn deployment_url(&self, project_name: &str) -> String {
        self.deployments_origin.replace('*', project_name)
    }

    pub(crate) fn deployment_log_url(&self, deployment_id: Uuid) -> String {
        format!(
            "{}/deployments/{deployment_id}",
            self.server_origin.trim_end_matches('/')
        )
    }

    pub(crate) fn with_database(
        path: impl AsRef<Path>,
        server_origin: String,
        deployments_origin: String,
        deploy_path: String,
        deploy_tx: Sender<DeployTask>,
    ) -> Result<Self> {
        let deployments_origin = deployments_origin.trim_end_matches('/').to_string();
        let deployments_host = Self::server_host_from_origin(&deployments_origin);
        if !deployments_host.starts_with("*.") {
            anyhow::bail!(
                "SERVER_DEPLOYMENTS_ORIGIN must contain a wildcard hostname, e.g. https://*.example.com"
            );
        }
        log::info!("Using database at {}", path.as_ref().display());
        let database = Connection::open(path.as_ref(), OpenMode::ReadWrite)?;
        database.enable_wal_logging()?;
        database.apply_various_performance_settings()?;
        log::info!("Running database migrations...");
        run_migrations!(database, "src/migrations")?;
        database_seed(&database)?;
        Ok(Self {
            server_host: Self::server_host_from_origin(&server_origin),
            server_origin,
            deployments_origin,
            deployments_host: deployments_host.trim_start_matches("*.").to_string(),
            deploy_path,
            database,
            auth_session: None,
            auth_user: None,
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            maxminddb_reader: Arc::new(OnceLock::new()),
            deploy_tx,
        })
    }

    #[cfg(test)]
    pub(crate) fn with_test_database() -> Result<Self> {
        let (deploy_tx, _) = std::sync::mpsc::channel::<DeployTask>();
        let database = Connection::open_memory()?;
        log::info!("Running database migrations...");
        run_migrations!(database, "src/migrations")?;
        Ok(Self {
            server_host: "localhost".to_string(),
            server_origin: "http://localhost".to_string(),
            deployments_origin: "http://*.localhost".to_string(),
            deployments_host: "localhost".to_string(),
            deploy_path: "deploy".to_string(),
            database,
            auth_session: None,
            auth_user: None,
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            maxminddb_reader: Arc::new(OnceLock::new()),
            deploy_tx,
        })
    }
}

// MARK: Database helpers
pub(crate) trait DatabaseHelpers {
    fn insert_user(&self, user: User) -> Result<()>;
    fn insert_session(&self, session: Session) -> Result<()>;
    fn insert_project(&self, project: Project) -> Result<()>;
    fn insert_deployment(&self, deployment: Deployment) -> Result<()>;
    fn insert_team(&self, team: Team) -> Result<()>;
    fn insert_team_user(&self, team_user: TeamUser) -> Result<()>;
    fn create_user_with_default_team(&self, user: User) -> Result<Team>;
    fn create_team_with_owner(&self, team: Team, owner_user_id: Uuid) -> Result<()>;
    fn find_user_by_id(&self, id: Uuid) -> Result<Option<User>>;
    fn find_user_by_email(&self, email: &str) -> Result<Option<User>>;
    fn find_session_by_id(&self, id: Uuid) -> Result<Option<Session>>;
    fn find_project_by_id(&self, id: Uuid) -> Result<Option<Project>>;
    fn find_deployment_by_id(&self, id: Uuid) -> Result<Option<Deployment>>;
    fn find_team_by_id(&self, id: Uuid) -> Result<Option<Team>>;
    fn find_team_github_token_by_team_id(&self, team_id: Uuid) -> Result<Option<TeamGitHubToken>>;
    fn find_first_team_by_user_id(&self, user_id: Uuid) -> Result<Option<Team>>;
    fn team_user_role(&self, user_id: Uuid, team_id: Uuid) -> Result<Option<TeamUserRole>>;
    fn user_is_team_member(&self, user_id: Uuid, team_id: Uuid) -> Result<bool>;
    fn user_is_team_owner(&self, user_id: Uuid, team_id: Uuid) -> Result<bool>;
}

fn in_transaction<T>(database: &Connection, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
    database.execute("BEGIN IMMEDIATE", ())?;
    let result = f(database);
    match result {
        Ok(value) => {
            database.execute("COMMIT", ())?;
            Ok(value)
        }
        Err(err) => {
            let _ = database.execute("ROLLBACK", ());
            Err(err)
        }
    }
}

impl DatabaseHelpers for Connection {
    fn insert_user(&self, user: User) -> Result<()> {
        self.execute(
            const_format::formatcp!(
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
            const_format::formatcp!(
                "INSERT INTO sessions ({}) VALUES ({})",
                Session::columns(),
                Session::values()
            ),
            session,
        )?;
        Ok(())
    }

    fn insert_project(&self, project: Project) -> Result<()> {
        self.execute(
            const_format::formatcp!(
                "INSERT INTO projects ({}) VALUES ({})",
                Project::columns(),
                Project::values()
            ),
            project,
        )?;
        Ok(())
    }

    fn insert_deployment(&self, deployment: Deployment) -> Result<()> {
        self.execute(
            const_format::formatcp!(
                "INSERT INTO deployments ({}) VALUES ({})",
                Deployment::columns(),
                Deployment::values()
            ),
            deployment,
        )?;
        Ok(())
    }

    fn insert_team(&self, team: Team) -> Result<()> {
        self.execute(
            const_format::formatcp!(
                "INSERT INTO teams ({}) VALUES ({})",
                Team::columns(),
                Team::values()
            ),
            team,
        )?;
        Ok(())
    }

    fn insert_team_user(&self, team_user: TeamUser) -> Result<()> {
        self.execute(
            const_format::formatcp!(
                "INSERT INTO team_users ({}) VALUES ({})",
                TeamUser::columns(),
                TeamUser::values()
            ),
            team_user,
        )?;
        Ok(())
    }

    fn create_user_with_default_team(&self, user: User) -> Result<Team> {
        let display_name = format!("{} {}", user.first_name.trim(), user.last_name.trim())
            .trim()
            .to_string();
        let team = Team {
            name: format!("{display_name}'s Team"),
            created_at: user.created_at,
            updated_at: user.updated_at,
            ..Default::default()
        };
        let team_user = TeamUser {
            team_id: team.id,
            user_id: user.id,
            role: TeamUserRole::Owner,
            created_at: user.created_at,
            updated_at: user.updated_at,
            ..Default::default()
        };

        in_transaction(self, |database| {
            database.insert_user(user)?;
            database.insert_team(team.clone())?;
            database.insert_team_user(team_user)?;
            Ok(team.clone())
        })
    }

    fn create_team_with_owner(&self, team: Team, owner_user_id: Uuid) -> Result<()> {
        let now = Utc::now();
        let team_user = TeamUser {
            team_id: team.id,
            user_id: owner_user_id,
            role: TeamUserRole::Owner,
            created_at: now,
            updated_at: now,
            ..Default::default()
        };

        in_transaction(self, |database| {
            database.insert_team(team.clone())?;
            database.insert_team_user(team_user)?;
            Ok(())
        })
    }

    fn find_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
        Ok(self
            .query::<User>(
                const_format::formatcp!(
                    "SELECT {} FROM users WHERE id = ? LIMIT 1",
                    User::columns()
                ),
                id,
            )?
            .next()
            .transpose()?)
    }

    fn find_user_by_email(&self, email: &str) -> Result<Option<User>> {
        Ok(self
            .query::<User>(
                const_format::formatcp!(
                    "SELECT {} FROM users WHERE email = ? LIMIT 1",
                    User::columns()
                ),
                email.to_string(),
            )?
            .next()
            .transpose()?)
    }

    fn find_session_by_id(&self, id: Uuid) -> Result<Option<Session>> {
        Ok(self
            .query::<Session>(
                const_format::formatcp!(
                    "SELECT {} FROM sessions WHERE id = ? LIMIT 1",
                    Session::columns()
                ),
                id,
            )?
            .next()
            .transpose()?)
    }

    fn find_project_by_id(&self, id: Uuid) -> Result<Option<Project>> {
        Ok(self
            .query::<Project>(
                const_format::formatcp!(
                    "SELECT {} FROM projects WHERE id = ? LIMIT 1",
                    Project::columns()
                ),
                id,
            )?
            .next()
            .transpose()?)
    }

    fn find_deployment_by_id(&self, id: Uuid) -> Result<Option<Deployment>> {
        Ok(self
            .query::<Deployment>(
                const_format::formatcp!(
                    "SELECT {} FROM deployments WHERE id = ? LIMIT 1",
                    Deployment::columns()
                ),
                id,
            )?
            .next()
            .transpose()?)
    }

    fn find_team_by_id(&self, id: Uuid) -> Result<Option<Team>> {
        Ok(self
            .query::<Team>(
                const_format::formatcp!(
                    "SELECT {} FROM teams WHERE id = ? LIMIT 1",
                    Team::columns()
                ),
                id,
            )?
            .next()
            .transpose()?)
    }

    fn find_team_github_token_by_team_id(&self, team_id: Uuid) -> Result<Option<TeamGitHubToken>> {
        Ok(self
            .query::<TeamGitHubToken>(
                const_format::formatcp!(
                    "SELECT {} FROM team_github_tokens WHERE team_id = ? LIMIT 1",
                    TeamGitHubToken::columns()
                ),
                team_id,
            )?
            .next()
            .transpose()?)
    }

    fn find_first_team_by_user_id(&self, user_id: Uuid) -> Result<Option<Team>> {
        Ok(self
            .query::<Team>(
                const_format::formatcp!(
                    "SELECT {} FROM teams
                     INNER JOIN team_users ON team_users.team_id = teams.id
                     WHERE team_users.user_id = ?
                     ORDER BY team_users.role DESC, teams.created_at ASC
                     LIMIT 1",
                    Team::columns()
                ),
                user_id,
            )?
            .next()
            .transpose()?)
    }

    fn team_user_role(&self, user_id: Uuid, team_id: Uuid) -> Result<Option<TeamUserRole>> {
        Ok(self
            .query::<TeamUser>(
                formatcp!(
                    "SELECT {} FROM team_users WHERE user_id = ? AND team_id = ? LIMIT 1",
                    TeamUser::columns()
                ),
                (user_id, team_id),
            )?
            .next()
            .transpose()?
            .map(|team_user| team_user.role))
    }

    fn user_is_team_member(&self, user_id: Uuid, team_id: Uuid) -> Result<bool> {
        Ok(self.team_user_role(user_id, team_id)?.is_some())
    }

    fn user_is_team_owner(&self, user_id: Uuid, team_id: Uuid) -> Result<bool> {
        Ok(self.team_user_role(user_id, team_id)? == Some(TeamUserRole::Owner))
    }
}
