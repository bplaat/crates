# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

_Placeholder_

## [0.2.1] - 2025-09-11

### Changed

-   Don't return `&String` types, return `&str` instead
-   Fix bug where headers where not case insensitive compared correctly

## [0.2.0] - 2025-06-27

### Added

-   Added `Request::get` method shorthand constructors
-   Added `Client` HTTP client with connection pool for making multiple requests
-   Added `HeaderMap` type for managing HTTP headers instead of `HashMap`
-   Added `Response::takeover` method to take ownership of the TCP stream after response, needed for WebSocket support
-   Added WebSocket server example
-   Added `server_single_threaded` for single-threaded HTTP server

### Changed

-   Moved `ThreadPool` and `serve` behind `multi-threaded` default feature flag
-   Updated some error messages for clarity
-   Fixed some small bugs in request parsing and response handling

## [0.1.0] - 2025-02-21

_Initial release_

[Unreleased]: https://github.com/bplaat/crates/compare/small-http%2Fv0.2.1...HEAD
[0.2.1]: https://github.com/bplaat/crates/releases/tag/small-http%2Fv0.2.1
[0.2.0]: https://github.com/bplaat/crates/releases/tag/small-http%2Fv0.2.0
[0.1.0]: https://github.com/bplaat/crates/releases/tag/small-http%2Fv0.1.0
