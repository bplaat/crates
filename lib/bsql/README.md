# Bassie SQL crate

A simple and minimal Rust SQLite and MySQL library with an ergonomic API.

SQLite is enabled by default and can be selected explicitly with the `sqlite` feature. The
MySQL backend is enabled with the `mysql` feature. On Unix targets, enabling `mysql` also makes
Unix socket transports available. Use `default-features = false` to build with only the backend
features an application needs. Enable `sqlite-bundled` to compile and link the bundled SQLite
source instead of using the system library.

## SQLite example

An example that inserts and reads rows to and from structs:

```rs
use bsql::{Connection, FromRow};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // Insert rows
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
        )?;
    }

    // Group related writes atomically
    db.transaction(|transaction| -> Result<(), bsql::StatementError> {
        transaction.execute(
            "UPDATE persons SET age = age + 1 WHERE name = ?",
            "Alice".to_string(),
        )?;
        transaction.execute(
            "UPDATE persons SET age = age + 1 WHERE name = ?",
            "Bob".to_string(),
        )?;
        Ok(())
    })
    .expect("Can't update persons");

    // Read rows back
    let persons = db.query::<Person>(format!("SELECT {} FROM persons", Person::columns()), ())?;
    for person in persons {
        let person = person?;
        println!("{person:?}"); // -> Person { id: 1, name: "Alice", age: 31 }
    }
    Ok(())
}
```

See the [examples](examples/) for many more examples.

## MySQL example

The MySQL backend implements the MySQL classic protocol directly and does not use another
database client crate:

```rs
use bsql::Connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database =
        Connection::open_mysql_tcp("localhost", 3306, "app", "secret", Some("app"), true)?;
    let names = database
        .query::<String>("SELECT name FROM persons WHERE age >= ?", 18_i64)?
        .collect::<Result<Vec<_>, _>>()?;
    println!("{names:?}");
    Ok(())
}
```

On Unix, `Connection::open_mysql_unix("/tmp/mysql.sock", "app", "secret", Some("app"))`
connects through a local socket. `Connection::open_mysql_tcp` supports optional database selection
and choosing whether verified TLS is required.
Empty-password accounts, MySQL 8.4 `caching_sha2_password`, and the `auth_socket`/`unix_socket`
account plugins are supported, including server-requested authentication switches. Enable the
`mysql-native-password` feature for the legacy MySQL/MariaDB `mysql_native_password` plugin.
Password exchange over TCP requires verified TLS when `caching_sha2_password` requests full
authentication; insecure RSA password exchange is intentionally not implemented.

## Design goals

- Connect to SQLite or MySQL through one API
- Implement the MySQL protocol without depending on a MySQL client crate
- Bind and read portable `Value` types through server-side prepared statements
- Have `FromRow` and `FromValue` derive macros for typed application models
- Work well and efficient with popular crates like `uuid` and `chrono`
- Have helpful error messages on query errors

## License

Copyright © 2024-2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
