/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::Path;

use bsqlite::{Connection, OpenMode};
use const_format::formatcp;

use crate::models::{Note, Session, User};

// MARK: Context
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

// MARK: Database helpers
pub(crate) trait DatabaseHelpers {
    fn insert_user(&self, user: User);
    fn insert_session(&self, session: Session);
    fn insert_note(&self, note: Note);
}

impl DatabaseHelpers for Connection {
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

    fn insert_session(&self, session: Session) {
        self.execute(
            formatcp!(
                "INSERT INTO sessions ({}) VALUES ({})",
                Session::columns(),
                Session::values()
            ),
            session,
        );
    }

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
}

fn database_create_tables(database: &Connection) {
    database.execute(
        "CREATE TABLE IF NOT EXISTS users(
            id BLOB PRIMARY KEY,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            role INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        ) STRICT",
        (),
    );

    database.execute(
        "CREATE TABLE IF NOT EXISTS sessions(
            id BLOB PRIMARY KEY,
            user_id BLOB NOT NULL,
            token TEXT NOT NULL,
            expires_at INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        ) STRICT",
        (),
    );

    database.execute(
        "CREATE TABLE IF NOT EXISTS notes(
            id BLOB PRIMARY KEY,
            user_id BLOB NOT NULL,
            body TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        ) STRICT",
        (),
    );
}

// MARK: Test helpers
#[cfg(test)]
pub(crate) mod test_helpers {
    use std::time::Duration;

    use chrono::Utc;

    use super::{Context, DatabaseHelpers};
    use crate::consts::SESSION_EXPIRY_SECONDS;
    use crate::models::user::UserRole;
    use crate::models::{Session, User};

    // Creates a test user and returns (user, token)
    pub(crate) fn create_test_user_with_session(ctx: &Context) -> (User, String) {
        create_test_user_with_session_and_role(ctx, UserRole::Normal)
    }

    // Creates a test user with specific role and returns (user, token)
    pub(crate) fn create_test_user_with_session_and_role(
        ctx: &Context,
        role: UserRole,
    ) -> (User, String) {
        // Create test user with specified role
        let user = User {
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: format!("test-{}@example.com", uuid::Uuid::now_v7()),
            password: pbkdf2::password_hash("password123"),
            role,
            ..Default::default()
        };
        ctx.database.insert_user(user.clone());

        // Create session with token
        let token = format!("test-token-{}", user.id);
        let session = Session {
            user_id: user.id,
            token: token.clone(),
            expires_at: Utc::now() + Duration::from_secs(SESSION_EXPIRY_SECONDS),
            ..Default::default()
        };
        ctx.database.insert_session(session);

        (user, token)
    }
}
