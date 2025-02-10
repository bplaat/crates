/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [libsqlite3-sys](https://crates.io/crates/libsqlite3-sys) crate for the [sqlite](lib/sqlite) crate.

fn main() {
    // Link to the system SQLite library
    #[cfg(not(feature = "bundled"))]
    {
        println!("cargo:rustc-link-lib=sqlite3");
    }

    // Or compile and link the SQLite library from source
    #[cfg(feature = "bundled")]
    {
        cc::Build::new()
            .file("sqlite3/sqlite3.c")
            .compile("sqlite3");
    }
}
