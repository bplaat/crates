/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A example that inserts and reads rows with structs that derive [FromRow] and have [chrono::DateTime] fields.

use std::fmt::Debug;

use bsqlite::{Connection, FromRow};
use chrono::{DateTime, Utc};

#[derive(FromRow)]
struct NewPerson {
    name: String,
    age: i64,
    created_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct Person {
    id: i64,
    name: String,
    age: i64,
    created_at: DateTime<Utc>,
}

impl Debug for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Person")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("age", &self.age)
            .field("created_at", &self.created_at.to_string())
            .finish()
    }
}

fn main() {
    // Connect and create table
    let db = Connection::open_memory().expect("Can't open database");
    db.execute(
        "CREATE TABLE IF NOT EXISTS persons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            age INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        ) STRICT",
        (),
    );

    // Insert a rows
    let persons = [
        NewPerson {
            name: "Alice".to_string(),
            age: 30,
            created_at: Utc::now(),
        },
        NewPerson {
            name: "Bob".to_string(),
            age: 40,
            created_at: Utc::now(),
        },
    ];
    for person in persons {
        db.execute(
            format!(
                "INSERT INTO persons ({}) VALUES ({})",
                NewPerson::columns(),
                NewPerson::values()
            ),
            person,
        );
    }

    // Read rows back
    let persons = db
        .query::<Person>(format!("SELECT {} FROM persons", Person::columns()), ())
        .collect::<Vec<_>>();
    for person in &persons {
        println!("{:?}", person);
    }
}
