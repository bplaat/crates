/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A example that uses the execute_args and query_args macro's to bind named parameters.

use bsqlite::{execute_args, query_args, Connection};

fn main() -> anyhow::Result<()> {
    // Connect and create table
    let db = Connection::open_memory().expect("Can't open database");
    db.execute(
        "CREATE TABLE IF NOT EXISTS persons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            age INTEGER NOT NULL
        ) STRICT",
        (),
    )?;

    // Insert a rows
    execute_args!(
        db,
        "INSERT INTO persons (name, age) VALUES (:name, :age)",
        Args {
            name: "Alice".to_string(),
            age: 30,
        },
    )?;
    execute_args!(
        db,
        "INSERT INTO persons (name, age) VALUES (:name, :age)",
        Args {
            name: "Bob".to_string(),
            age: 40,
        },
    )?;

    // Read rows
    for row in query_args!(
        (String, i64),
        db,
        "SELECT name, age FROM persons LIMIT :limit",
        Args { limit: 2 }
    )? {
        println!("{:?}", row?);
    }
    Ok(())
}
