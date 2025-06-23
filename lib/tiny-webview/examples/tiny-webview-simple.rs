/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple tiny webview example

use tiny_webview::{EventLoop, EventLoopBuilder, WebviewBuilder};

fn main() {
    let mut event_loop = EventLoopBuilder::build();

    let mut _webview = WebviewBuilder::new()
        .title("Webview Simple Example")
        .load_url("https://github.com/bplaat/crates")
        .build();

    event_loop.run(|_| {});
}
