/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple webview example

use webview::WebviewBuilder;

fn main() {
    let mut webview = WebviewBuilder::new()
        .title("Webview Simple Example")
        .size(1024, 768)
        .url("https://github.com/bplaat/crates")
        .build();
    webview.run(|_, _| {});
}
