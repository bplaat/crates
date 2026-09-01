/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A example that opens database in file, enables WAL-mode and inserts and reads rows with tuples.

use bsql::{Connection, PoolOptions, SqliteMode};

fn main() -> anyhow::Result<()> {
    // Connect and create table
    let db = Connection::open_sqlite(
        "database.db",
        SqliteMode::ReadWrite,
        PoolOptions::single_connection(),
    )
    .expect("Can't open database");
    db.enable_wal_logging()?;
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

    // Read rows
    for row in db.query::<(String, i64)>("SELECT name, age FROM persons", ())? {
        println!("{:?}", row?);
    }
    Ok(())
}
