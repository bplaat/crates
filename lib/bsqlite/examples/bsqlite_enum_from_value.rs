/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A example that inserts and reads rows with tuples and enums that derive [FromValue].

use bsqlite::{Connection, FromValue};

#[derive(Debug, FromValue)]
enum HairColor {
    Brown = 0,
    Blond = 1,
    Black = 2,
}

fn main() {
    // Connect and create table
    let db = Connection::open_memory().expect("Can't open database");
    db.execute(
        "CREATE TABLE IF NOT EXISTS persons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            age INTEGER NOT NULL,
            hair_color INTEGER NOT NULL
        ) STRICT",
        (),
    );

    // Insert a rows
    db.execute(
        "INSERT INTO persons (name, age, hair_color) VALUES (?, ?, ?), (?, ?, ?)",
        (
            "Alice".to_string(),
            30,
            HairColor::Brown,
            // ...
            "Bob".to_string(),
            40,
            HairColor::Blond,
        ),
    );

    // Read rows
    let rows =
        db.query::<(String, i64, HairColor)>("SELECT name, age, hair_color FROM persons", ());
    for row in rows {
        println!("{row:?}");
    }
}
