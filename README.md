# Bassie's Rust crates

A collection of minimal Rust crates and tools that I created for myself and others

## Crates

### Published libraries

These crates are published to [crates.io](https://crates.io) and are more polished and documented

- [bsqlite](lib/bsqlite) A simple and minimal Rust SQLite library with an ergonomic API **([crates.io](https://crates.io/crates/bsqlite))**
- [bsqlite_derive](lib/bsqlite_derive) The derive macro's for the [bsqlite](lib/bsqlite) crate **([crates.io](https://crates.io/crates/bsqlite_derive))**
- [simple-useragent](lib/simple-useragent) A simple user agent parser library based on the uap-core regexes **([crates.io](https://crates.io/crates/simple-useragent))**
- [small-http](lib/small-http) A simple and small HTTP/1.1 server/client library **([crates.io](https://crates.io/crates/small-http))**
- [small-router](lib/small-router) A simple and small router for the [small-http](lib/small-http) library **([crates.io](https://crates.io/crates/small-router))**

### Libraries

These libraries are not published to [crates.io](https://crates.io) and are more intended for personal use but can still be useful

- [bwebview](lib/bwebview) A cross-platform webview library for Rust with minimal dependencies
- [from_derive](lib/from_derive) A FromEnum and FromStruct derive macro library
- [js](lib/js) A WIP JavaScript interpreter
- [openapi-generator](lib/openapi-generator) A simple OpenAPI code generator
- [pbkdf2](lib/pbkdf2) A PBKDF2-HMAC-SHA256 password hashing library
- [small-websocket](lib/small-websocket) A simple and small websocket library for the [small-http](lib/small-http) library
- [validate](lib/validate) A simple struct validation library
- [validate_derive](lib/validate_derive) Validation derive macro's library

### Apps

Some desktop apps written with the [bwebview](lib/bwebview) library

<table>
<tr>
<td width="100" align="center">
    <a href="bin/game2048">
        <img src="bin/game2048/docs/images/icon.svg" alt="2048 icon" width="48" height="48"/><br/>
        2048
    </a>
</td>
<td width="100" align="center">
    <a href="bin/bassielight">
        <img src="bin/bassielight/docs/images/icon.svg" alt="BassieLight icon" width="48" height="48"/><br/>
        BassieLight
    </a>
</td>
<td width="100" align="center">
    <a href="bin/manexplorer">
        <img src="bin/manexplorer/docs/images/icon.svg" alt="ManExplorer icon" width="48" height="48"/><br/>
        ManExplorer
    </a>
</td>
<td width="100" align="center">
    <a href="bin/navidrome">
        <img src="bin/navidrome/docs/images/icon.svg" alt="Navidrome icon" width="48" height="48"/><br/>
        Navidrome
    </a>
</td>
<td width="100" align="center">
    <a href="bin/music-dl">
        <img src="bin/music-dl/docs/images/icon.svg" alt="Music Downloader icon" width="48" height="48"/><br/>
        Music Downloader
    </a>
</td>
<td width="100" align="center">
    <a href="bin/pixelfont">
        <img src="bin/pixelfont/docs/images/icon.svg" alt="Pixel Font Editor icon" width="48" height="48"/><br/>
        Pixel Font Editor
    </a>
</td>
<td width="100" align="center">
    <a href="bin/sequelexplorer">
        <img src="bin/sequelexplorer/docs/images/icon.svg" alt="Sequel Explorer icon" width="48" height="48"/><br/>
        Sequel Explorer
    </a>
</td>
</tr>
</table>

- [2048](bin/game2048) An offline desktop 2048 game app
- [BassieLight](bin/bassielight) A simple lights controller with GUI
- [ManExplorer](bin/manexplorer) A simple man page explorer tool
- [Navidrome](bin/navidrome) A [music.bplaat.nl](https://music.bplaat.nl/) webview wrapper
- [Music Downloader](bin/music-dl) A tool that downloads complete albums with the correct metadata
- [Pixel Font Editor](bin/pixelfont) An 8x8 pixel font editor
- [Sequel Explorer](bin/sequelexplorer) A simple SQLite database GUI viewer

### Tools / Websites

Various tools and websites, for the live deployed sites go to [crates.bplaat.nl](https://crates.bplaat.nl/)

- [baksteen](bin/baksteen/) A brick laying robot simulator
- [bob](bin/bob) A simple build system for my projects, because I like the simplicity of Cargo
- [ccontinue](bin/ccontinue) A transpiler that translates an OOP-extension for the C programming language back to C
- [cargo-bundle](bin/cargo-bundle) A simple Cargo plugin the builds macOS app bundles
- [plaatnotes](bin/plaatnotes) A self-hosted note taking web app with rich markdown support

### Replacement libraries

These libraries are created as minimal / smaller replacements for common used crates

- [base64](lib/base64) A minimal replacement for the [base64](https://crates.io/crates/base64) crate
- [block2](lib/block2) A minimal replacement for the [block2](https://crates.io/crates/block2) crate
- [chrono](lib/chrono) A minimal replacement for the [chrono](https://crates.io/crates/chrono) crate
- [copy_dir](lib/copy_dir) A minimal replacement for the [copy_dir](https://crates.io/crates/copy_dir) crate
- [digest](lib/digest) A minimal replacement for the [digest](https://crates.io/crates/digest) crate
- [directories](lib/directories) A minimal replacement for the [directories](https://crates.io/crates/directories) crate
- [dotenv](lib/dotenv) A minimal replacement for the [dotenv](https://crates.io/crates/dotenv) crate
- [enable-ansi-support](lib/enable-ansi-support) A minimal replacement for the [enable-ansi-support](https://crates.io/crates/enable-ansi-support) crate
- [form_urlencoded](lib/form_urlencoded) A minimal replacement for the [form_urlencoded](https://crates.io/crates/form_urlencoded) crate
- [getrandom](lib/getrandom) A minimal replacement for the [getrandom](https://crates.io/crates/getrandom) crate
- [hmac](lib/hmac) A minimal replacement for the [hmac](https://crates.io/crates/hmac) crate
- [libsqlite3-sys](lib/libsqlite3-sys) A minimal replacement for the [libsqlite3-sys](https://crates.io/crates/libsqlite3-sys) crate
- [local-ip-address](lib/local-ip-address) A minimal replacement for the [local-ip-address](https://crates.io/crates/local-ip-address) crate
- [native-tls](lib/native-tls) A minimal replacement for the [native-tls](https://crates.io/crates/native-tls) crate
- [maxminddb](lib/maxminddb) A minimal replacement for the [maxminddb](https://crates.io/crates/maxminddb) crate
- [mime](lib/mime) A minimal replacement for the [mime](https://crates.io/crates/mime) crate
- [mime_guess](lib/mime_guess) A minimal replacement for the [mime_guess](https://crates.io/crates/mime_guess) crate
- [objc2](lib/objc2) A minimal replacement for the [objc2](https://crates.io/crates/objc2) crate
- [objc2-proc-macros](lib/objc2-proc-macros) A minimal replacement for the [objc2](https://crates.io/crates/objc2-proc-macros) crate
- [percent-encoding](lib/percent-encoding) A minimal replacement for the [percent-encoding](https://crates.io/crates/percent-encoding) crate
- [plist](lib/plist) A minimal replacement for the [plist](https://crates.io/crates/plist) crate
- [rust-embed](lib/rust-embed) A minimal replacement for the [rust-embed](https://crates.io/crates/rust-embed) crate
- [rust-embed-impl](lib/rust-embed-impl) A minimal replacement for the [rust-embed-impl](https://crates.io/crates/rust-embed-impl) crate
- [semver](lib/semver) A minimal replacement for the [semver](https://crates.io/crates/semver) crate
- [serde_yaml](lib/serde_yaml) A minimal replacement for the [serde_yaml](https://crates.io/crates/serde_yaml) crate
- [sha1](lib/sha1) A minimal replacement for the [sha1](https://crates.io/crates/sha1) crate
- [sha2](lib/sha2) A minimal replacement for the [sha2](https://crates.io/crates/sha2) crate
- [simple_logger](lib/simple_logger) A minimal replacement for the [simple_logger](https://crates.io/crates/simple_logger) crate
- [subtle](lib/subtle) A minimal replacement for the [subtle](https://crates.io/crates/subtle) crate
- [terminal_size](lib/terminal_size) A minimal replacement for the [terminal_size](https://crates.io/crates/terminal_size) crate
- [threadpool](lib/threadpool) A minimal replacement for the [threadpool](https://crates.io/crates/threadpool) crate
- [url](lib/url) A minimal replacement for the [url](https://crates.io/crates/url) crate
- [uuid](lib/uuid) A minimal replacement for the [uuid](https://crates.io/crates/uuid) crate
- [winresource](lib/winresource) A minimal replacement for the [winresource](https://crates.io/crates/winresource) crate
- [zip](lib/zip) A minimal replacement for the [zip](https://crates.io/crates/zip) crate

## Getting Started

- Install [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/), [Hadolint](https://github.com/hadolint/hadolint) and [clang-format](https://clang.llvm.org/docs/ClangFormat.html)
- Install Rust nightly toolchain, `cargo-binstall`, `cargo-deny` and `cargo-nextest`:

    ```sh
    rustup toolchain add nightly --component rust-src --component rustfmt
    cargo install cargo-binstall
    cargo binstall -y --locked cargo-deny cargo-nextest
    ```

- Run checks, or run an example:

    ```sh
    cargo xtask check
    cargo run --bin example-persons-api
    cargo run --bin example-todo-app
    ```

### Additional tools

- For coverage reports, install the `llvm-tools` and `cargo-llvm-cov` tool:

    ```sh
    rustup component add llvm-tools
    cargo binstall -y --locked cargo-llvm-cov
    cargo xtask coverage
    ```

- To build pages, install a wasm target and the `wasm-bindgen-cli` tool:

    ```sh
    rustup target add wasm32-unknown-unknown
    cargo binstall -y --locked wasm-bindgen-cli --version 0.2.104
    cargo xtask build-pages
    ```

- To build macOS app bundles, only on macOS, install targets:

    ```sh
    rustup target add aarch64-apple-darwin x86_64-apple-darwin
    cargo xtask build-bundle
    ```

- To build and install bins and GUI applications to your system:

    ```sh
    cargo xtask install
    ```

## Goals

This project is driven by three main goals:

**1. A self-written minimal crates collection for backend software**

Rather than pulling in large, general-purpose dependencies, this repo grows a curated set of small, focused libraries that cover exactly what is needed - nothing more. This includes an HTTP/1.1 server, a router, a SQLite wrapper, validation, password hashing, UUID generation, and more. Many of these are intentional minimal replacements for well-known crates on [crates.io](https://crates.io), rewritten from scratch to be lean, auditable, and free of transitive dependencies.

**2. A minimal Electron/Tauri-like desktop platform**

[bwebview](lib/bwebview) is a cross-platform native webview library that lets you ship desktop applications using web technologies, without the weight of bundling a full browser engine. It uses the system-native webview on each platform (WebKit on macOS/iOS, WebKitGTK on Linux, WebView2 on Windows) and exposes a small, clean Rust API. The [desktop apps](bin/) in this repo: a 2048 game, a SQLite viewer, a pixel font editor, a man page browser, serve as real-world validation of this platform.

**3. Tools and a custom build system for complex multi-language projects**

Building projects that span multiple languages and artifact types (Rust, JavaScript, WASM, native bundles) requires coordination that general-purpose build systems handle poorly. [bob](bin/bob) is a minimal build system inspired by `Cargo`'s simplicity, designed to describe and orchestrate such projects cleanly. [cargo-bundle](bin/cargo-bundle) handles macOS `.app` bundle packaging from within the `Cargo` ecosystem. Together with the [xtask](xtask) crate these tools keep the build pipeline typed, cross-platform and easy to follow.

### Repo organization

This repo is organized as a monorepo with `Cargo` as the single build system. Monorepos work best when one build system acts as the central organization point for all submodules, and `Cargo` fills that role here perfectly.

All Rust code lives in a single `Cargo` workspace that builds everything together. Some crates extend this with custom build steps that invoke other tools like `npm` or `vite`, but those are orchestrated from within `Cargo` build scripts.

The root-level [xtask](xtask) crate contains all top-level tasks and is used through `cargo xtask` both from the command line and in CI. The only exception is the [bob/examples](bin/bob/examples) directory, which intentionally uses a different build system as a showcase for the [bob](bin/bob) build tool itself.

## License

Copyright © 2021-2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](LICENSE) license.
