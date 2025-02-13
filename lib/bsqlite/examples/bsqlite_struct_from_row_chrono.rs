/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A example that inserts and reads rows with structs that derive [FromRow] and have [chrono::DateTime] fields.

use std::fmt::Debug;

use bsqlite::{Connection, FromRow};
use chrono::{DateTime, Utc};
use const_format::formatcp;

#[derive(FromRow)]
struct NewPerson {
    name: String,
    age: i64,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct Person {
    id: i64,
    name: String,
    age: i64,
    created_at: DateTime<Utc>,
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
            formatcp!(
                "INSERT INTO persons ({}) VALUES ({})",
                NewPerson::columns(),
                NewPerson::values()
            ),
            person,
        );
    }

    // Read rows back
    let persons = db.query::<Person>(formatcp!("SELECT {} FROM persons", Person::columns()), ());
    for person in persons {
        println!("{:?}", person);
    }
}
