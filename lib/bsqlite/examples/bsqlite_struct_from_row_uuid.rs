/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A example that inserts and reads rows with structs that derive [FromRow] and uses [uuid::Uuid] for ids.

use bsqlite::{Connection, FromRow};
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct Person {
    id: Uuid,
    name: String,
    age: i64,
}

fn main() {
    // Connect and create table
    let db = Connection::open_memory().expect("Can't open database");
    db.execute(
        "CREATE TABLE IF NOT EXISTS persons (
            id BLOB PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER NOT NULL
        ) STRICT",
        (),
    );

    // Insert a rows
    let persons = [
        Person {
            id: Uuid::now_v7(),
            name: "Alice".to_string(),
            age: 30,
        },
        Person {
            id: Uuid::now_v7(),
            name: "Bob".to_string(),
            age: 40,
        },
    ];
    for person in persons {
        db.execute(
            format!(
                "INSERT INTO persons ({}) VALUES ({})",
                Person::columns(),
                Person::values()
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
