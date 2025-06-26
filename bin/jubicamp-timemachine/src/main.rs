/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Time machine software for the JubiCamp 2025 scouting camp

#![forbid(unsafe_code)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use rust_embed::Embed;
use tiny_webview::{EventLoopBuilder, WebviewBuilder};

#[derive(Embed)]
#[folder = "web"]
struct WebAssets;

fn main() {
    let event_loop = EventLoopBuilder::build();
    let monitors = event_loop.available_monitors();

    let _webview = WebviewBuilder::new()
        .title("Screen 1")
        .position(monitors[0].position())
        .size(monitors[0].size())
        .fullscreen(true)
        .load_rust_embed::<WebAssets>()
        .load_url("/screen1.html")
        .build();

    if let Some(monitor) = monitors.get(1) {
        let _webview = WebviewBuilder::new()
            .title("Screen 2")
            .position(monitor.position())
            .size(monitor.size())
            .fullscreen(true)
            .load_rust_embed::<WebAssets>()
            .load_url("/screen2.html")
            .build();
    }

    if let Some(monitor) = monitors.get(2) {
        let _webview = WebviewBuilder::new()
            .title("Screen 3")
            .position(monitor.position())
            .size(monitor.size())
            .fullscreen(true)
            .load_rust_embed::<WebAssets>()
            .load_url("/screen3.html")
            .build();
    }

    event_loop.run(move |_event| {});
}
