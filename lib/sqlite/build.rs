/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

fn main() {
    #[cfg(not(feature = "bundled"))]
    {
        println!("cargo:rustc-link-lib=sqlite3");
    }

    #[cfg(feature = "bundled")]
    {
        cc::Build::new()
            .file("sqlite3/sqlite3.c")
            .compile("sqlite3");
    }
}
