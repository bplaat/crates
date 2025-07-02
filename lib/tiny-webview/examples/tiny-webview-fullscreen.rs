/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple tiny webview example

use tiny_webview::{EventLoopBuilder, WebviewBuilder};

fn main() {
    let event_loop = EventLoopBuilder::build();

    let mut _webview = WebviewBuilder::new()
        .title("Webview Simple Fullscreen Example")
        .fullscreen() // Set fullscreen to true
        .load_url("https://example.com")
        .build();

    event_loop.run(|_| {});
}
