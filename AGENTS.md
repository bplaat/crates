# Agent Development Guide

A file for [guiding coding agents](https://agents.md/).

## Project Overview

This is a Rust monorepo containing a collection of personal libraries, tools, and applications. The project emphasizes minimalism and simplicity, with many crates serving as smaller replacements for common published alternatives.

**Key characteristics:**

- **Single Cargo workspace** managing all Rust code
- **Multiple crate types**: Published libraries (crates.io), internal libraries, CLI tools, and desktop GUI applications
- **Cross-platform support**: macOS, Windows, and Linux (Freedesktop)
- **Minimal dependencies philosophy**: Custom replacements for standard crates when possible

## Build, Test, and Lint Commands

### Quick checks (most common)

```sh
./meta.sh check        # Run all checks: copyright, formatting, linting, tests (full suite)
```

### Individual check categories

```sh
./meta.sh coverage     # Generate coverage reports (requires cargo-llvm-cov)
./meta.sh build-pages  # Build WebAssembly pages (requires wasm target + wasm-bindgen-cli)
./meta.sh build-bundle # Build macOS app bundles (macOS only, requires targets)
./meta.sh install      # Build and install GUI apps and tools system-wide
./meta.sh clean        # Deep clean including databases, node_modules, dist directories
```

### Running individual tests

```sh
cargo test -p <crate-name>              # Run tests for a specific crate
cargo test -p <crate-name> -- --test-threads=1   # Run tests serially
cargo nextest run -p <crate-name>       # Run with nextest (faster, parallelized)
cargo test --doc                        # Run doc tests only
```

### Running individual examples

```sh
cargo run --example <example-name> -p <crate-name>
cargo run --bin <binary-name>
```

### Common dev workflow

```sh
cargo check -p <crate-name>             # Quick compile check
cargo clippy -p <crate-name>            # Lint without full compilation
cargo build --release -p <crate-name>   # Optimized build
```

## Architecture

### Repository Structure

- **lib/** - Published and internal libraries
    - Published to crates.io: `bsqlite`, `bsqlite_derive`, `simple-useragent`, `small-http`, `small-router`
    - Internal libraries: `bwebview`, `js`, `openapi-generator`, `validate`, etc.
    - Minimal replacements: `chrono`, `directories`, `dotenv`, `mime`, `uuid`, etc.
    - Derive macros: `bsqlite_derive`, `validate_derive`, `from_enum`
- **bin/** - Executable binaries (CLI tools and GUI apps)
    - GUI apps: `bassielight`, `game2048`, `manexplorer`, `navidrome`
    - CLI tools: `bob` (build system), `cargo-bundle`, `music-dl`, `webhook-puller`
    - Web projects: `baksteen` (WebAssembly game)
    - User apps: `plaatnotes`
- **examples/** - Example applications demonstrating library usage
    - `persons-api` - REST API example using `small-http`
    - `todo-app` - Web app example
    - `http-simple` - Basic HTTP example

### Workspace Configuration

The `[workspace]` in Cargo.toml includes:

- `members = ["bin/*", "examples/*", "lib/*", "lib/*/examples/*"]`
- Uses Cargo edition 2024
- Defines workspace-level lints and dependencies via `[workspace.package]`

### Patching Strategy

The workspace patches many popular crates with local minimal alternatives:

```toml
[patch.crates-io]
chrono = { path = "lib/chrono" }
directories = { path = "lib/directories" }
# ... many more
```

These are automatically used throughout dependencies when crates try to pull the published versions.

## Key Conventions

### Code Structure

1. **Copyright headers** - All source files MUST start with:

    ```rust
    /*
     * Copyright (c) 20XX[-20XX] Bastiaan van der Plaat
     *
     * SPDX-License-Identifier: MIT
     */
    ```

    This is enforced by `check_copyright()` in meta.sh.

2. **Module organization** - Each library typically has:

    ```rust
    mod module1;
    mod module2;

    pub use module1::PublicType;
    pub use module2::PublicFunction;
    ```

3. **Documentation** - All public items require doc comments
    - Workspace lint: `missing_docs = "deny"`
    - Include README.md in lib.rs: `#![doc = include_str!("../README.md")]`

4. **Safety and style**
    - `forbid(unsafe_code)` is commonly used in libraries
    - `unreachable_pub` and `unused_qualifications` = "deny" enforced
    - `unwrap_used` in clippy = "deny" (use `?` operator instead)
    - No `#![allow(...)]` overrides without discussion

### Formatting and Linting

1. **Rust formatting** - Uses nightly rustfmt:

    ```sh
    cargo +nightly fmt -- --check    # Check
    cargo +nightly fmt               # Fix
    ```

    Config in `rustfmt.toml`: `group_imports = "StdExternalCrate"`, `imports_granularity = "Module"`

2. **Non-Rust formatting** - Prettier for JS/TS/CSS/HTML/JSON/YAML:

    ```sh
    npx prettier --check <files>     # Check
    npx prettier --write <files>     # Fix
    ```

3. **C/C++/Java formatting** - clang-format in bob/examples

4. **Clippy linting**:

    ```sh
    cargo clippy --all-targets --all-features -- -D warnings
    ```

5. **Dependency audit**:
    ```sh
    cargo deny check
    ```

### Testing Patterns

- Tests are typically in `#[cfg(test)]` modules at the end of files
- Use `cargo test` for basic testing; `cargo nextest run` for parallelized testing
- Doc tests are run with `cargo test --doc`
- No external testing frameworks; uses standard Rust test harness

### Release Profile

The workspace optimizes for small binary size:

```toml
[profile.release]
opt-level = "z"    # Optimize for size
lto = "fat"        # Link-time optimization
strip = true       # Strip symbols
panic = "abort"    # Abort on panic (smaller binaries)
overflow-checks = false
```

### Dependency Philosophy

- Prefer minimal/internal implementations over external crates
- When using external crates, they should be:
    - Small and with few dependencies
    - Actively maintained or very stable
    - Well-tested
- Popular alternatives are patched with local minimal versions

### Examples Structure

Examples are in `examples/` directory and are part of the workspace. Run with:

```sh
cargo run --example <name>
cargo run --bin <name>  # For binary examples
```

## Installation and Dependencies

### Required tools

- Rust (via rustup): `curl --proto '=https' --tlsv1.2 -sSf https://rustup.rs/ | sh`
- Rust nightly with rustfmt: `rustup toolchain add nightly --component rustfmt`
- Node.js (for prettier and web tooling)
- clang-format (for C/C++/Java formatting)
- Cargo tools: `cargo binstall cargo-deny cargo-nextest`

### Optional tools

- `cargo-llvm-cov` - For coverage reports
- `wasm-bindgen-cli` (v0.2.104) - For WebAssembly builds
- Platform-specific targets for cross-compilation

## Notes for Copilot

1. **Always use workspace member paths**: When referring to crates, use `-p <crate-name>` with cargo commands
2. **Check meta.sh first**: For complex build tasks, see meta.sh before implementing custom scripts
3. **Minimal dependencies**: If suggesting a new dependency, consider if a minimal alternative exists in this repo
4. **Edition 2024**: Code uses Rust edition 2024 (relatively new; same as 2021 mostly but with some future changes)
5. **No unsafe code**: Most crates forbid unsafe; suggest safe alternatives or discuss necessity
6. **Test before changing**: Run `./meta.sh check` to verify nothing breaks after modifications
