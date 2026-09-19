/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use anyhow::{Context as _, Result, bail};
use bsql::{Connection, PoolOptions, SqliteMode, run_migrations};
use zeroize::Zeroizing;

use crate::deploy::DeployTask;
use crate::models::{Deployment, Project, Session, User, UserRole};

#[derive(Clone)]
pub(crate) struct Context {
    pub server_origin: String,
    pub deployments_domain: String,
    pub data_path: Arc<PathBuf>,
    pub database: Connection,
    pub deploy_tx: Sender<DeployTask>,
    pub auth_session: Option<Session>,
    pub auth_user: Option<User>,
    pub login_attempts: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    pub maxminddb_reader: Arc<OnceLock<maxminddb::Reader<Vec<u8>>>>,
    pub trusted_proxies: Arc<[IpAddr]>,
    pub is_e2e: bool,
}

impl Context {
    pub(crate) fn with_database(
        path: impl AsRef<Path>,
        server_origin: String,
        deployments_domain: String,
        data_path: PathBuf,
        deploy_tx: Sender<DeployTask>,
    ) -> Result<Self> {
        log::info!("Using database at {}", path.as_ref().display());
        let database =
            Connection::open_sqlite(path.as_ref(), SqliteMode::ReadWrite, PoolOptions::default())?;
        database.execute_script("PRAGMA foreign_keys = ON")?;
        database.enable_wal_logging()?;
        run_migrations!(database, "src/migrations")?;
        database_seed(&database)?;
        Ok(Self {
            server_origin,
            deployments_domain: normalize_deployments_domain(&deployments_domain)?,
            data_path: Arc::new(data_path),
            database,
            deploy_tx,
            auth_session: None,
            auth_user: None,
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            maxminddb_reader: Arc::new(OnceLock::new()),
            trusted_proxies: trusted_proxies_from_env()?,
            is_e2e: false,
        })
    }

    #[cfg(test)]
    pub(crate) fn with_test_database() -> Result<Self> {
        let database = Connection::open_sqlite_memory()?;
        database.execute_script("PRAGMA foreign_keys = ON")?;
        run_migrations!(database, "src/migrations")?;
        let (deploy_tx, _) = std::sync::mpsc::channel();
        Ok(Self {
            server_origin: "http://localhost".to_string(),
            deployments_domain: "apps.example.com".to_string(),
            data_path: Arc::new(PathBuf::new()),
            database,
            deploy_tx,
            auth_session: None,
            auth_user: None,
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            maxminddb_reader: Arc::new(OnceLock::new()),
            trusted_proxies: Arc::from([]),
            is_e2e: false,
        })
    }
}

fn normalize_deployments_domain(value: &str) -> Result<String> {
    let value = value.trim();
    let Some(domain) = value.strip_prefix("*.") else {
        bail!("deployments domain must start with '*.'");
    };
    let domain = domain.trim_end_matches('.').to_ascii_lowercase();
    if domain.is_empty()
        || domain.len() > 253
        || !domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
    {
        bail!("invalid deployments domain: {value}");
    }
    Ok(domain)
}

fn trusted_proxies_from_env() -> Result<Arc<[IpAddr]>> {
    let Some(value) = std::env::var("TRUSTED_PROXIES").ok() else {
        return Ok(Arc::from([]));
    };
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .parse()
                .with_context(|| format!("invalid trusted proxy IP address: {value}"))
        })
        .collect::<Result<Vec<_>>>()
        .map(Arc::from)
}

pub(crate) trait DatabaseHelpers {
    fn insert_user(&self, user: User) -> Result<()>;
    fn insert_session(&self, session: Session) -> Result<()>;
    fn insert_project(&self, project: Project) -> Result<()>;
    fn insert_deployment(&self, deployment: Deployment) -> Result<()>;
}

impl DatabaseHelpers for Connection {
    fn insert_user(&self, user: User) -> Result<()> {
        self.execute(
            format!(
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
            format!(
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
            format!(
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
            format!(
                "INSERT INTO deployments ({}) VALUES ({})",
                Deployment::columns(),
                Deployment::values()
            ),
            deployment,
        )?;
        Ok(())
    }
}

fn database_seed(database: &Connection) -> Result<()> {
    if database.query_some::<i64>("SELECT COUNT(id) FROM users", ())? == 0 {
        let password = Zeroizing::new(std::env::var("ADMIN_PASSWORD").context(
            "ADMIN_PASSWORD must be set when creating the initial administrator account",
        )?);
        if !(8..=128).contains(&password.len()) {
            bail!("ADMIN_PASSWORD must be between 8 and 128 bytes");
        }
        let email = std::env::var("ADMIN_EMAIL")
            .unwrap_or_else(|_| "admin@example.com".to_string())
            .trim()
            .to_ascii_lowercase();
        if !validate::is_valid_email(&email) {
            bail!("ADMIN_EMAIL must be a valid email address");
        }
        database.insert_user(User {
            first_name: "Admin".to_string(),
            last_name: "Admin".to_string(),
            email,
            password: pbkdf2::password_hash(&password),
            role: UserRole::Admin,
            ..Default::default()
        })?;
        log::info!("Admin account created");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_wildcard_deployment_domain() {
        assert_eq!(
            normalize_deployments_domain("*.example.com").expect("valid domain"),
            "example.com"
        );
        assert_eq!(
            normalize_deployments_domain(" *.BPLAAT.nl. ").expect("valid domain"),
            "bplaat.nl"
        );
        assert_eq!(
            normalize_deployments_domain("*.apps.bplaat.nl").expect("valid domain"),
            "apps.bplaat.nl"
        );
    }

    #[test]
    fn rejects_invalid_deployment_domains() {
        for domain in [
            "",
            "apps.example.com",
            "*.",
            ".example.com",
            "example..com",
            "-apps.example.com",
            "apps_.example.com",
        ] {
            assert!(normalize_deployments_domain(domain).is_err(), "{domain}");
        }
    }
}
