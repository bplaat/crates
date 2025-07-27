/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A example that inserts and reads rows with structs that derive [FromRow].

use bsqlite::{Connection, FromRow};
use const_format::formatcp;

#[derive(FromRow)]
struct NewPerson {
    name: String,
    age: i64,
}

#[derive(Debug, FromRow)]
struct Person {
    id: i64,
    name: String,
    age: i64,
}

fn main() {
    // Connect and create table
    let db = Connection::open_memory().expect("Can't open database");
    db.execute(
        "CREATE TABLE IF NOT EXISTS persons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            age INTEGER NOT NULL
        ) STRICT",
        (),
    );

    // Insert a rows
    let persons = [
        NewPerson {
            name: "Alice".to_string(),
            age: 30,
        },
        NewPerson {
            name: "Bob".to_string(),
            age: 40,
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
        println!("{person:?}");
    }
}
