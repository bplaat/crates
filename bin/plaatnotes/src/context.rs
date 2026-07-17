/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use anyhow::Result;
use bsqlite::{Connection, OpenMode, run_migrations};
use uuid::Uuid;

use crate::models::{Note, Session, User, UserRole};

// MARK: Context
#[derive(Clone)]
pub(crate) struct Context {
    pub server_origin: String,
    pub database: Connection,
    pub auth_session: Option<Session>,
    pub auth_user: Option<User>,
    pub update_target_user_id: Option<Uuid>,
    pub login_attempts: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    pub maxminddb_reader: Arc<OnceLock<maxminddb::Reader<Vec<u8>>>>,
    pub is_e2e: bool,
}

impl Context {
    #[allow(dead_code)]
    pub(crate) fn with_database(path: impl AsRef<Path>, server_origin: String) -> Result<Self> {
        log::info!("Using database at {}", path.as_ref().display());
        let database = Connection::open(path.as_ref(), OpenMode::ReadWrite)?;
        database.enable_wal_logging()?;
        database.apply_various_performance_settings()?;
        log::info!("Running database migrations...");
        run_migrations!(database, "src/migrations")?;
        database_seed(&database)?;
        Ok(Self {
            server_origin,
            database,
            auth_session: None,
            auth_user: None,
            update_target_user_id: None,
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            maxminddb_reader: Arc::new(OnceLock::new()),
            is_e2e: false,
        })
    }

    #[cfg(test)]
    pub(crate) fn with_test_database() -> Result<Self> {
        let database = Connection::open_memory()?;
        log::info!("Running database migrations...");
        run_migrations!(database, "src/migrations")?;
        Ok(Self {
            server_origin: "*".to_string(),
            database,
            auth_session: None,
            auth_user: None,
            update_target_user_id: None,
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            maxminddb_reader: Arc::new(OnceLock::new()),
            is_e2e: false,
        })
    }

    pub(crate) fn with_e2e_database() -> Result<Self> {
        let database = Connection::open_memory()?;
        log::info!("Running database migrations...");
        run_migrations!(database, "src/migrations")?;
        database_seed(&database)?;
        Ok(Self {
            server_origin: "*".to_string(),
            database,
            auth_session: None,
            auth_user: None,
            update_target_user_id: None,
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            maxminddb_reader: Arc::new(OnceLock::new()),
            is_e2e: true,
        })
    }
}

// MARK: Database helpers
pub(crate) trait DatabaseHelpers {
    fn insert_user(&self, user: User) -> Result<()>;
    fn insert_session(&self, session: Session) -> Result<()>;
    fn insert_note(&self, note: Note) -> Result<()>;
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

    fn insert_note(&self, note: Note) -> Result<()> {
        self.execute(
            format!(
                "INSERT INTO notes ({}) VALUES ({})",
                Note::columns(),
                Note::values()
            ),
            note,
        )?;
        Ok(())
    }
}

fn database_seed(database: &Connection) -> Result<()> {
    if database.query_some::<i64>("SELECT COUNT(id) FROM users", ())? == 0 {
        let admin_email =
            std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());
        let admin_password =
            std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "Password123!".to_string());
        let user = User {
            first_name: "Admin".to_string(),
            last_name: "Admin".to_string(),
            email: admin_email,
            password: pbkdf2::password_hash(&admin_password),
            role: UserRole::Admin,
            ..Default::default()
        };
        database.insert_user(user)?;
        log::info!("Admin account created");
    }
    Ok(())
}
