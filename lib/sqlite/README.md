# Rust SQLite

A simple Rust SQLite library with derive macro's

## Design goals

-   Connect and execute queries on a SQLite database
-   Have a generic `Value` enum type to represent SQLite values
-   Bind and read `Value` types to and from sqlite statements
-   Have `Bind` and `FromRow` derive macros to convert between Rust types and SQLite values
