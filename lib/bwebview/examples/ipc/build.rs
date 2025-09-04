/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A webview ipc example

fn main() {
    // Minify app.html
    let out_dir = std::env::var("OUT_DIR").expect("Should be some");
    minify_html::minify_file("app.html", format!("{out_dir}/app.min.html"))
        .expect("Should minify html");
}
