# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

_Placeholder_

## [0.1.2] - 2025-02-13

### Changed

-   Examples use `formatcp!()` macro for const formatting of the query string.

### Added

-   Add `Statement::bind_named()` function to bind named parameters to a statement.
-   Add `query_args!()` and `execute_args!()` macros to bind with struct named parameters to a query or execute statement.
-   Add more examples about the added features.

## [0.1.1] - 2025-02-12

### Changed

-   Make the `derive` feature default.
-   Improved error message they now also include the query string.

### Added

-   Add many more examples about the different features op the crate.
-   Add `Connection::open_memory()` function to open an in-memory database.

## [0.1.0] - 2025-02-10

_Initial release_

[Unreleased]: https://github.com/bplaat/crates/compare/bsqlite%2Fv0.1.2...HEAD
[0.1.2]: https://github.com/bplaat/crates/releases/tag/bsqlite%2Fv0.1.2
[0.1.1]: https://github.com/bplaat/crates/releases/tag/bsqlite%2Fv0.1.1
[0.1.0]: https://github.com/bplaat/crates/releases/tag/bsqlite%2Fv0.1.0
