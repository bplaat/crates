/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use bsqlite::{Connection, OpenMode};
use const_format::formatcp;

use crate::models::{Person, Relation};

// MARK: Context
#[derive(Clone)]
pub(crate) struct Context {
    pub database: Connection,
}

impl Context {
    pub(crate) fn with_database(path: &str) -> Result<Self> {
        let database = Connection::open(path, OpenMode::ReadWrite)?;
        database.enable_wal_logging()?;
        database.apply_various_performance_settings()?;
        database_create_tables(&database)?;
        database_seed(&database)?;
        Ok(Self { database })
    }

    #[cfg(test)]
    pub(crate) fn with_test_database() -> Self {
        let database = Connection::open_memory().expect("Can't open in memory database");
        database_create_tables(&database).expect("Can't create tables in database");
        Self { database }
    }
}

// MARK: Database Helpers
pub(crate) trait DatabaseHelpers {
    fn create_fts_tables(&self, table: &str) -> Result<()>;
    fn insert_person(&self, person: Person) -> Result<()>;
}

impl DatabaseHelpers for Connection {
    fn create_fts_tables(&self, table: &str) -> Result<()> {
        self.execute(
            format!(
                "CREATE VIRTUAL TABLE IF NOT EXISTS {table}_fts USING fts5(name, id UNINDEXED)"
            ),
            (),
        )?;

        self.execute(
            format!(
                "CREATE TRIGGER IF NOT EXISTS {table}_ai AFTER INSERT ON {table} BEGIN
                    INSERT INTO {table}_fts(name, id) VALUES (new.name, new.id);
                END"
            ),
            (),
        )?;

        self.execute(
            format!(
                "CREATE TRIGGER IF NOT EXISTS {table}_au AFTER UPDATE ON {table} BEGIN
                    UPDATE {table}_fts SET name = new.name WHERE id = old.id;
                END"
            ),
            (),
        )?;

        self.execute(
            format!(
                "CREATE TRIGGER IF NOT EXISTS {table}_ad BEFORE DELETE ON {table} BEGIN
                    DELETE FROM {table}_fts WHERE id = old.id;
                END"
            ),
            (),
        )?;

        Ok(())
    }

    fn insert_person(&self, person: Person) -> Result<()> {
        self.execute(
            formatcp!(
                "INSERT INTO persons ({}) VALUES ({})",
                Person::columns(),
                Person::values()
            ),
            person,
        )?;
        Ok(())
    }
}

fn database_create_tables(database: &Connection) -> Result<()> {
    database.execute(
        "CREATE TABLE IF NOT EXISTS persons(
            id BLOB PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER NOT NULL,
            relation INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        ) STRICT",
        (),
    )?;
    database.create_fts_tables("persons")?;
    Ok(())
}

fn database_seed(database: &Connection) -> Result<()> {
    // Insert persons
    if database.query_some::<i64>("SELECT COUNT(id) FROM persons", ())? == 0 {
        database.insert_person(Person {
            name: "Bastiaan".to_string(),
            age_in_years: 20,
            relation: Relation::Me,
            ..Default::default()
        })?;
        database.insert_person(Person {
            name: "Sander".to_string(),
            age_in_years: 19,
            relation: Relation::Brother,
            ..Default::default()
        })?;
        database.insert_person(Person {
            name: "Leonard".to_string(),
            age_in_years: 16,
            relation: Relation::Brother,
            ..Default::default()
        })?;
        database.insert_person(Person {
            name: "Jiska".to_string(),
            age_in_years: 14,
            relation: Relation::Sister,
            ..Default::default()
        })?;
    }
    Ok(())
}
