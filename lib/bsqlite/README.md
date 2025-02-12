# Bassie SQLight crate

A simple and minimal Rust SQLite library with an ergonomic API

## Example

A simple example that connects, creates a table, inserts rows, and reads them back:

```rs
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

    // Insert row
    db.execute(
        "INSERT INTO persons (name, age) VALUES (?, ?)",
        ("Bastiaan".to_string(), 22),
    );

    // Read row back
    let row = db
        .query::<(String, i64)>("SELECT name, age FROM persons", ())
        .next();
    println!("{:?}", row); // -> ("Bastiaan", 22)
}
```

See the [examples](examples/) for many more examples.

## Design goals

-   Connect and execute queries on a SQLite database
-   Have a generic `Value` enum type to represent SQLite values
-   Bind and read `Value` types to and from SQLite statements
-   Have `FromRow` and `FromValue` derive macros to convert between Rust types to SQLite `Value`'s
-   Work well and efficient with popular crates like `uuid` and `chrono`
-   Have helpful error messages on query errors

## Documentation

See the [documentation](https://docs.rs/bsqlite) for more information.

## License

Copyright © 2024-2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
