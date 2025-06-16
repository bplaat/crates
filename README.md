# Bassie's Rust crates

A collection of minimal Rust crates and tools that I created for myself and others

## Crates

### Published libraries

-   [bsqlite](lib/bsqlite) A simple and minimal Rust SQLite library with an ergonomic API **([crates.io](https://crates.io/crates/bsqlite))**
-   [bsqlite_derive](lib/bsqlite_derive) The derive macro's for the [bsqlite](lib/bsqlite) crate **([crates.io](https://crates.io/crates/bsqlite_derive))**
-   [simple-useragent](lib/simple-useragent) A simple user agent parser library based on the uap-core regexes **([crates.io](https://crates.io/crates/simple-useragent))**
-   [small-http](lib/small-http) A simple and small HTTP/1.1 server/client library **([crates.io](https://crates.io/crates/small-http))**
-   [small-router](lib/small-router) A simple and small router for the [small-http](lib/small-http) library **([crates.io](https://crates.io/crates/small-router))**

### Libraries

-   [from_enum](lib/from_enum) A FromEnum derive macro library
-   [js](lib/js) A WIP JavaScript interpreter
-   [minify-html](lib/minify-html) A simple HTML minifier library
-   [objc](lib/objc) An Objective-C ffi library
-   [openapi-generator](lib/openapi-generator) A simple OpenAPI code generator
-   [pbkdf2](lib/pbkdf2) A unsecure PBKDF2-HMAC-SHA256 password hashing library
-   [tiny-webview](lib/tiny-webview) A cross-platform webview library for Rust with minimal dependencies
-   [validate](lib/validate) A simple struct validation library
-   [validate_derive](lib/validate_derive) Validation derive macro's library

### Tools

-   [bassielight](bin/bassielight) A simple lights controller with GUI
-   [bob](bin/bob) A simple meta-build system for my projects
-   [music-dl](bin/music-dl) A tool that downloads complete albums with the correct metadata
-   [webhook-puller](bin/webhook-puller) A small service that pulls a Git repo when requested by a webhook

### Replacement libraries

-   [chrono](lib/chrono) A minimal replacement for the [chrono](https://crates.io/crates/chrono) crate
-   [dirs](lib/dirs) A minimal replacement for the [dirs](https://crates.io/crates/dirs) crate
-   [getrandom](lib/getrandom) A minimal replacement for the [getrandom](https://crates.io/crates/getrandom) crate
-   [libsqlite3-sys](lib/libsqlite3-sys) A minimal replacement for the [libsqlite3-sys](https://crates.io/crates/libsqlite3-sys) crate
-   [rust-embed](lib/rust-embed) A minimal replacement for the [rust-embed](https://crates.io/crates/rust-embed) crate
-   [rust-embed-impl](lib/rust-embed-impl) A minimal replacement for the [rust-embed-impl](https://crates.io/crates/rust-embed-impl) crate
-   [threadpool](lib/threadpool) A minimal replacement for the [threadpool](https://crates.io/crates/threadpool) crate
-   [url](lib/url) A minimal replacement for the [url](https://crates.io/crates/url) crate
-   [uuid](lib/uuid) A minimal replacement for the [uuid](https://crates.io/crates/uuid) crate

## Getting Started

-   Install the latest Rust toolchain with [rustup](https://rustup.rs/)
-   Install [Node.js](https://nodejs.org/)
-   Install the Rust nightly `rustfmt`:

    ```sh
    rustup toolchain add nightly --component rustfmt
    ```

-   Install `cargo-binstall` for [your platform](https://github.com/cargo-bins/cargo-binstall#quickly)
-   Install the `cargo-deny` and `cargo-nextest` tools:

    ```sh
    cargo binstall cargo-deny cargo-nextest
    ```

-   Run checks:

    ```sh
    ./meta.sh check
    ```

-   Or run an example:

    ```sh
    cargo run --bin example-persons-api
    cargo run --bin example-todo-app
    ```

## License

Copyright © 2023-2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](LICENSE) license.
