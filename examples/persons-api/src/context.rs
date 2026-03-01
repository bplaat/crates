/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::{Connection, OpenMode};
use const_format::formatcp;

use crate::models::{Person, Relation};

// MARK: Context
#[derive(Clone)]
pub(crate) struct Context {
    pub database: Connection,
}

impl Context {
    pub(crate) fn with_database(path: &str) -> Self {
        let database = Connection::open(path, OpenMode::ReadWrite).expect("Can't open database");
        database.enable_wal_logging().expect("Database error");
        database
            .apply_various_performance_settings()
            .expect("Database error");
        database_create_tables(&database);
        database_seed(&database);
        Self { database }
    }

    #[cfg(test)]
    pub(crate) fn with_test_database() -> Self {
        let database = Connection::open_memory().expect("Can't open database");
        database_create_tables(&database);
        Self { database }
    }
}

// MARK: Database Helpers
pub(crate) trait DatabaseHelpers {
    fn insert_person(&self, person: Person);
}
impl DatabaseHelpers for Connection {
    fn insert_person(&self, person: Person) {
        self.execute(
            formatcp!(
                "INSERT INTO persons ({}) VALUES ({})",
                Person::columns(),
                Person::values()
            ),
            person,
        )
        .expect("Database error");
    }
}

fn database_create_tables(database: &Connection) {
    database
        .execute(
            "CREATE TABLE IF NOT EXISTS persons(
            id BLOB PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER NOT NULL,
            relation INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        ) STRICT",
            (),
        )
        .expect("Database error");
}

fn database_seed(database: &Connection) {
    // Insert persons
    if database
        .query_some::<i64>("SELECT COUNT(id) FROM persons", ())
        .expect("Database error")
        == 0
    {
        database.insert_person(Person {
            name: "Bastiaan".to_string(),
            age_in_years: 20,
            relation: Relation::Me,
            ..Default::default()
        });
        database.insert_person(Person {
            name: "Sander".to_string(),
            age_in_years: 19,
            relation: Relation::Brother,
            ..Default::default()
        });
        database.insert_person(Person {
            name: "Leonard".to_string(),
            age_in_years: 16,
            relation: Relation::Brother,
            ..Default::default()
        });
        database.insert_person(Person {
            name: "Jiska".to_string(),
            age_in_years: 14,
            relation: Relation::Sister,
            ..Default::default()
        });
    }
}
