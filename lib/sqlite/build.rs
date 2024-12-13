/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A SQLite Rust library

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
