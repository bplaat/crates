# Bassie's Rust crates

A collection of minimal Rust crates that I created for myself

## Crates

-   [base64](lib/base64) A base64 encoder and decoder
-   [getrandom](lib/getrandom) A minimal crypto random bytes library
-   [from_enum](lib/from_enum) A FromEnum derive macro library
-   [http](lib/http) A simple HTTP/1.1 server/client library
-   [minify-html](lib/minify-html) A simple HTML minifier library
-   [objc](lib/objc) A basic Objective-C ffi library
-   [openapi-generator](lib/openapi-generator) A simple OpenAPI code generator
-   [pbkdf2](lib/pbkdf2) A unsecure PBKDF2-HMAC-SHA256 password hashing library
-   [router](lib/router) A simple router for HTTP library
-   [sqlite](lib/sqlite) A SQLite Rust library
-   [sqlite_derive](lib/sqlite_derive) SQLite derive macro's library
-   [sqlite3-sys](lib/sqlite3-sys) A SQLite3 sys bindings library
-   [threadpool](lib/threadpool) A very basic thread pool library
-   [time](lib/time) A simple UTC DateTime library similar to `chrono`
-   [url](lib/url) A minimal URL parser library
-   [useragent](lib/useragent) An user agent parser library
-   [uuid](lib/uuid) A minimal UUID library
-   [validate](lib/validate) A simple struct validation library
-   [validate_derive](lib/validate_derive) Validation derive macro's library
-   [webview](lib/webview) A simple webview library

## Getting Started

-   Install the latest Rust toolchain with [rustup](https://rustup.rs/)
-   Install nightly `rustfmt`, `cargo-deny` and `cargo-nextest`

    ```sh
    rustup toolchain add nightly --component rustfmt
    cargo install cargo-deny cargo-nextest
    ```

-   Run checks:

    ```sh
    make -C check
    ```

-   Or run an example:

    ```sh
    cargo run --bin example-persons-api
    ```

## TODO items

-   [ ] webview: Support opening links in browser
-   [ ] webview: Add Windows (win32 + Webview2) support

## License

Copyright © 2024-2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](LICENSE) license.
