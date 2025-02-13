/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple tiny webview example

use tiny_webview::{Webview, WebviewBuilder};

fn main() {
    let mut webview = WebviewBuilder::new()
        .title("Webview Simple Example")
        .load_url("https://github.com/bplaat/crates")
        .build();
    webview.run(|_, _| {});
}
