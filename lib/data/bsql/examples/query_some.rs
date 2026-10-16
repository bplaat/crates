/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A example that inserts and reads a single column back.

use bsql::Connection;

fn main() -> anyhow::Result<()> {
    // Connect and create table
    let db = Connection::open_sqlite_memory()?;
    db.execute(
        "CREATE TABLE IF NOT EXISTS persons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            age INTEGER NOT NULL
        ) STRICT",
        (),
    )?;

    // Insert a rows
    db.execute(
        "INSERT INTO persons (name, age) VALUES (?, ?), (?, ?)",
        ("Alice".to_string(), 30, "Bob".to_string(), 40),
    )?;

    // Read total rows
    println!(
        "Total person rows: {}",
        db.query_some::<i64>("SELECT COUNT(id) FROM persons", ())?
    );
    Ok(())
}
