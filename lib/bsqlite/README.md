# Bassie SQLight crate

A simple and minimal Rust SQLite library with an ergonomic API

## Example

A example that inserts and reads rows from and too structs:

```rs
use bsqlite::{Connection, FromRow};

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
            format!(
                "INSERT INTO persons ({}) VALUES ({})",
                NewPerson::columns(),
                NewPerson::values()
            ),
            person,
        );
    }

    // Read rows back
    let persons = db.query::<Person>(format!("SELECT {} FROM persons", Person::columns()), ());
    for person in persons {
        println!("{person:?}"); // -> Person { id: 1, name: "Alice", age: 30 }
    }
}
```

See the [examples](examples/) for many more examples.

## Design goals

- Connect and execute queries on a SQLite database
- Have a generic `Value` enum type to represent SQLite values
- Bind and read `Value` types to and from SQLite statements
- Have `FromRow` and `FromValue` derive macros to convert between Rust types to SQLite `Value`'s
- Work well and efficient with popular crates like `uuid` and `chrono`
- Have helpful error messages on query errors

## Documentation

See the [documentation](https://docs.rs/bsqlite) for more information.

## License

Copyright © 2024-2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
