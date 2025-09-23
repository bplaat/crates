# Bassie's Rust crates

A collection of minimal Rust crates and tools that I created for myself and others

## Crates

### Published libraries

These crates are published to [crates.io](https://crates.io) and are more polished and documented

-   [bsqlite](lib/bsqlite) A simple and minimal Rust SQLite library with an ergonomic API **([crates.io](https://crates.io/crates/bsqlite))**
-   [bsqlite_derive](lib/bsqlite_derive) The derive macro's for the [bsqlite](lib/bsqlite) crate **([crates.io](https://crates.io/crates/bsqlite_derive))**
-   [simple-useragent](lib/simple-useragent) A simple user agent parser library based on the uap-core regexes **([crates.io](https://crates.io/crates/simple-useragent))**
-   [small-http](lib/small-http) A simple and small HTTP/1.1 server/client library **([crates.io](https://crates.io/crates/small-http))**
-   [small-router](lib/small-router) A simple and small router for the [small-http](lib/small-http) library **([crates.io](https://crates.io/crates/small-router))**

### Libraries

These libraries are not published to [crates.io](https://crates.io) and are more intended for personal use but can still be useful

-   [bwebview](lib/bwebview) A cross-platform webview library for Rust with minimal dependencies
-   [from_enum](lib/from_enum) A FromEnum derive macro library
-   [ini](lib/ini) A simple INI file parser library
-   [js](lib/js) A WIP JavaScript interpreter
-   [minify-html](lib/minify-html) A simple HTML minifier library
-   [openapi-generator](lib/openapi-generator) A simple OpenAPI code generator
-   [pbkdf2](lib/pbkdf2) A unsecure PBKDF2-HMAC-SHA256 password hashing library
-   [small-websocket](lib/small-websocket) A simple and small websocket library for the [small-http](lib/small-http) library
-   [validate](lib/validate) A simple struct validation library
-   [validate_derive](lib/validate_derive) Validation derive macro's library

### Tools / Websites

Various tools and websites, for the live deployed sites go to [crates.bplaat.nl](https://crates.bplaat.nl/)

-   [baksteen](bin/baksteen/) A brick laying robot simulator
-   [bassielight](bin/bassielight) A simple lights controller with GUI
-   [bob](bin/bob) A simple build system for my projects, because I like the simplicity of Cargo
-   [cargo-bundle](bin/cargo-bundle) A simple Cargo plugin the builds macOS app bundles
-   [music-dl](bin/music-dl) A tool that downloads complete albums with the correct metadata
-   [navidrome](bin/navidrome) A [music.bplaat.nl](https://music.bplaat.nl/) webview wrapper
-   [plaatnotes](bin/plaatnotes) A simple note-taking app
-   [webhook-puller](bin/webhook-puller) A small service that pulls a Git repo when requested by a webhook

### Replacement libraries

These libraries are created as minimal / smaller replacements for common used crates

-   [chrono](lib/chrono) A minimal replacement for the [chrono](https://crates.io/crates/chrono) crate
-   [copy_dir](lib/copy_dir) A minimal replacement for the [copy_dir](https://crates.io/crates/copy_dir) crate
-   [dirs](lib/dirs) A minimal replacement for the [dirs](https://crates.io/crates/dirs) crate
-   [dotenv](lib/dotenv) A minimal replacement for the [dotenv](https://crates.io/crates/dotenv) crate
-   [enable-ansi-support](lib/enable-ansi-support) A minimal replacement for the [enable-ansi-support](https://crates.io/crates/enable-ansi-support) crate
-   [getrandom](lib/getrandom) A minimal replacement for the [getrandom](https://crates.io/crates/getrandom) crate
-   [libsqlite3-sys](lib/libsqlite3-sys) A minimal replacement for the [libsqlite3-sys](https://crates.io/crates/libsqlite3-sys) crate
-   [local-ip-address](lib/local-ip-address) A minimal replacement for the [local-ip-address](https://crates.io/crates/local-ip-address) crate
-   [mime_guess](lib/mime_guess) A minimal replacement for the [mime_guess](https://crates.io/crates/mime_guess) crate
-   [rust-embed](lib/rust-embed) A minimal replacement for the [rust-embed](https://crates.io/crates/rust-embed) crate
-   [rust-embed-impl](lib/rust-embed-impl) A minimal replacement for the [rust-embed-impl](https://crates.io/crates/rust-embed-impl) crate
-   [sha1](lib/sha1) A minimal replacement for the [sha1](https://crates.io/crates/sha1) crate
-   [terminal_size](lib/terminal_size) A minimal replacement for the [terminal_size](https://crates.io/crates/terminal_size) crate
-   [threadpool](lib/threadpool) A minimal replacement for the [threadpool](https://crates.io/crates/threadpool) crate
-   [url](lib/url) A minimal replacement for the [url](https://crates.io/crates/url) crate
-   [uuid](lib/uuid) A minimal replacement for the [uuid](https://crates.io/crates/uuid) crate

## Repo organization

This repo is organized as a monorepo with `Cargo` as the main build system. My personal vision is that monorepo's work vary well when there's only one main build system, that acts a single organization point for the whole project with all it's submodules.

So this all Rust code is contained in a single `Cargo` workspace, that builds everything. Some crates have custom build steps that can run other build scripts / systems like `npm` and `vite`.

The [meta.sh](meta.sh) script is contains all the main tasks, these are used from commandline and also CI. The only modules that use a different build system are the ones in the [bob/examples](bin/bob/examples) directory, but that is also a show case of the [bob](bin/bob) build system.

## Getting Started

-   Open a posix shell environment when you are on Windows (e.g. Git Bash)
-   Install [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/), [OpenJDK 17+](https://adoptium.net/) and [clang-format](https://clang.llvm.org/docs/ClangFormat.html)
-   Install Rust nightly `rustfmt`:

    ```sh
    rustup toolchain add nightly --component rustfmt
    ```

-   Install `cargo-binstall` see [documentation](https://github.com/cargo-bins/cargo-binstall#quickly)
-   Install `cargo-deny` and `cargo-nextest`:

    ```sh
    cargo binstall -y cargo-deny cargo-nextest
    ```

-   Run checks, or run an example:

    ```sh
    ./meta.sh check
    cargo run --bin example-persons-api
    cargo run --bin example-todo-app
    ```

-   For coverage reports, install the `llvm-tools` and `cargo-llvm-cov` tool:

    ```sh
    rustup component add llvm-tools
    cargo binstall -y cargo-llvm-cov
    ./meta.sh coverage
    ```

## License

Copyright © 2023-2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](LICENSE) license.
