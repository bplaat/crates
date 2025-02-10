/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple example that connects, creates a table, inserts rows, and reads them back.

fn main() {
    // Connect to in SQLite database
    let db = bsqlite::Connection::open("database.db").expect("Can't open database");

    // Create a table
    db.execute(
        "CREATE TABLE IF NOT EXISTS persons (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER NOT NULL
        ) STRICT",
        (),
    );

    // Insert a row
    db.execute(
        "INSERT INTO persons (name, age) VALUES (?, ?), (?, ?)",
        ("Alice".to_string(), 30, "Bob".to_string(), 40),
    );

    // Read total rows
    println!(
        "Total persons: {}",
        db.query_some::<i64>("SELECT COUNT(id) FROM persons", ())
    );

    // Read rows
    let rows = db.query::<(String, i64)>("SELECT name, age FROM persons", ());
    for (name, age) in rows {
        println!("Hello {}, you are {} years old!", name, age);
    }
}
