/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [libsqlite3-sys](https://crates.io/crates/libsqlite3-sys) crate.

fn main() {
    // Compile and link the SQLite library from source
    if cfg!(feature = "bundled") {
        cc::Build::new()
            .file("sqlite3/sqlite3.c")
            .compile("sqlite3");
    }
    // Or link to the system SQLite library
    else {
        println!("cargo:rustc-link-lib=sqlite3");
    }
}
