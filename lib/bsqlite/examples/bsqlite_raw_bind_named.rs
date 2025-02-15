/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A example that inserts and reads rows with tuples with a raw name binded parameters.

use bsqlite::Connection;

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

    // Insert a rows by preparing a statement and raw binding values
    let mut stat = db.prepare::<()>("INSERT INTO persons (name, age) VALUES (:name, :age)");
    stat.bind_named_value(":name", "Alice".to_string());
    stat.bind_named_value(":age", 30);
    stat.next(); // Execute the statement
    stat.reset(); // Reset to rebind values

    stat.bind_named_value(":name", "Bob".to_string());
    stat.bind_named_value(":age", 40);
    stat.next();

    // Read rows
    let rows = db.query::<(String, i64)>("SELECT name, age FROM persons", ());
    for row in rows {
        println!("{:?}", row);
    }
}
