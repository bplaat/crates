/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple bwebview example

use bwebview::{EventLoop, WebviewBuilder, WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .title("Webview Simple Fullscreen Example")
        .fullscreen()
        .build();
    let mut _webview = WebviewBuilder::new(&window)
        .load_url("https://example.com")
        .build();

    event_loop.run(|_| {});
}
