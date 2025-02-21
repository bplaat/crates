# Bassie's Rust crates

A collection of minimal Rust crates and tools that I created for myself and others

## Crates

### Published

-   [bsqlite](lib/bsqlite) A simple and minimal Rust SQLite library with an ergonomic API **([crates.io](https://crates.io/crates/bsqlite))**
-   [bsqlite_derive](lib/bsqlite_derive) The derive macro's for the [bsqlite](lib/bsqlite) crate **([crates.io](https://crates.io/crates/bsqlite_derive))**
-   [simple-useragent](lib/simple-useragent) A simple user agent parser library based on the uap-core regexes **([crates.io](https://crates.io/crates/simple-useragent))**

### Normal

-   [bob](bin/bob) A simple meta-build system for my projects
-   [from_enum](lib/from_enum) A FromEnum derive macro library
-   [http](lib/http) A simple HTTP/1.1 server/client library
-   [minify-html](lib/minify-html) A simple HTML minifier library
-   [objc](lib/objc) An Objective-C ffi library
-   [openapi-generator](lib/openapi-generator) A simple OpenAPI code generator
-   [pbkdf2](lib/pbkdf2) A unsecure PBKDF2-HMAC-SHA256 password hashing library
-   [router](lib/router) A simple router for HTTP library
-   [tiny-webview](lib/tiny-webview) A simple webview library
-   [validate](lib/validate) A simple struct validation library
-   [validate_derive](lib/validate_derive) Validation derive macro's library

### Replacements

-   [chrono](lib/chrono) A minimal replacement for the [chrono](https://crates.io/crates/chrono) crate
-   [getrandom](lib/getrandom) A minimal replacement for the [getrandom](https://crates.io/crates/getrandom) crate
-   [libsqlite3-sys](lib/libsqlite3-sys) A minimal replacement for the [libsqlite3-sys](https://crates.io/crates/libsqlite3-sys) crate
-   [threadpool](lib/threadpool) A minimal replacement for the [threadpool](https://crates.io/crates/threadpool) crate
-   [url](lib/url) A minimal replacement for the [uuid](https://crates.io/crates/url) crate
-   [uuid](lib/uuid) A minimal replacement for the [uuid](https://crates.io/crates/uuid) crate

## Getting Started

-   Install the latest Rust toolchain with [rustup](https://rustup.rs/)
-   Install nightly `rustfmt`, `cargo-deny` and `cargo-nextest`

    ```sh
    rustup toolchain add nightly --component rustfmt
    cargo install cargo-deny cargo-nextest
    ```

-   Run checks:

    ```sh
    ./meta.sh check
    ```

-   Or run an example:

    ```sh
    cargo run --bin example-persons-api
    ```

## TODO items

-   [ ] tiny-webview: Add Windows (win32 + Webview2) support
-   [ ] tiny-webview: Copy subset of objc bindings to make crate standalone
-   [ ] bob: Add Android project type
-   [ ] bob: Build universal multi target binaries with macOS bundle package
-   [ ] bob: Add path dependencies like cargo
    -   Add Java library
    -   Add Android library
    -   Add static .ar library
    -   Add Java testing JUnit support

## License

Copyright © 2024-2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](LICENSE) license.
