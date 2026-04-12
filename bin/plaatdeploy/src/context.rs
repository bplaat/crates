/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Application context

use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::Result;
use bsqlite::{Connection, OpenMode};
use chrono::Utc;
use const_format::formatcp;
use uuid::Uuid;

use crate::deploy::DeployTask;
use crate::migrations::database_migrate;
use crate::models::{
    Deployment, Project, Session, Team, TeamGitHubConnection, TeamUser, TeamUserRole, User,
};

// MARK: Context
#[derive(Clone)]
pub(crate) struct Context {
    pub server_origin: String,
    pub server_host: String,
    pub database: Connection,
    pub auth_session: Option<Session>,
    pub auth_user: Option<User>,
    pub login_attempts: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
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

    pub(crate) fn with_database(
        path: impl AsRef<Path>,
        server_origin: String,
        deploy_tx: Sender<DeployTask>,
    ) -> Result<Self> {
        log::info!("Using database at {}", path.as_ref().display());
        let database = Connection::open(path.as_ref(), OpenMode::ReadWrite)?;
        database.enable_wal_logging()?;
        database.apply_various_performance_settings()?;
        database_migrate(&database)?;
        Ok(Self {
            server_host: Self::server_host_from_origin(&server_origin),
            server_origin,
            database,
            auth_session: None,
            auth_user: None,
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
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
    fn find_team_github_connection_by_team_id(
        &self,
        team_id: Uuid,
    ) -> Result<Option<TeamGitHubConnection>>;
    fn find_team_github_connection_by_setup_state(
        &self,
        setup_state: &str,
    ) -> Result<Option<TeamGitHubConnection>>;
    fn find_team_github_connection_by_installation_id(
        &self,
        installation_id: i64,
    ) -> Result<Option<TeamGitHubConnection>>;
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

    fn find_team_github_connection_by_team_id(
        &self,
        team_id: Uuid,
    ) -> Result<Option<TeamGitHubConnection>> {
        Ok(self
            .query::<TeamGitHubConnection>(
                const_format::formatcp!(
                    "SELECT {} FROM team_github_connections WHERE team_id = ? LIMIT 1",
                    TeamGitHubConnection::columns()
                ),
                team_id,
            )?
            .next()
            .transpose()?)
    }

    fn find_team_github_connection_by_setup_state(
        &self,
        setup_state: &str,
    ) -> Result<Option<TeamGitHubConnection>> {
        Ok(self
            .query::<TeamGitHubConnection>(
                const_format::formatcp!(
                    "SELECT {} FROM team_github_connections WHERE setup_state = ? LIMIT 1",
                    TeamGitHubConnection::columns()
                ),
                setup_state.to_string(),
            )?
            .next()
            .transpose()?)
    }

    fn find_team_github_connection_by_installation_id(
        &self,
        installation_id: i64,
    ) -> Result<Option<TeamGitHubConnection>> {
        Ok(self
            .query::<TeamGitHubConnection>(
                const_format::formatcp!(
                    "SELECT {} FROM team_github_connections WHERE installation_id = ? LIMIT 1",
                    TeamGitHubConnection::columns()
                ),
                installation_id,
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
