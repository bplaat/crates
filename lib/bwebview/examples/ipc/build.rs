/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A webview ipc example

fn main() {
    // Minify app.html
    minify_html::minify_file(
        "app.html",
        std::env::var("OUT_DIR").expect("Should be some") + "/app.min.html",
    )
    .expect("Should minify html");
}
