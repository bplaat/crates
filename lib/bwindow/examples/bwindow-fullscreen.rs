/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple bwindow example

use bwindow::event_loop::EventLoopBuilder;
use bwindow::window::WindowBuilder;

fn main() {
    let event_loop = EventLoopBuilder::build();

    let mut _webview = WindowBuilder::new()
        .title("Webview Simple Fullscreen Example")
        .fullscreen() // Set fullscreen to true
        .build();

    event_loop.run(|_| {});
}
