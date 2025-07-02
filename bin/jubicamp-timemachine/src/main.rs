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
    let mut monitors = event_loop.available_monitors();
    monitors.sort_by_key(|m| !m.is_primary());

    let _webview = WebviewBuilder::new()
        .title("Screen 1")
        .monitor(&monitors[0])
        .fullscreen()
        .load_rust_embed::<WebAssets>()
        .load_url("/screen1.html")
        .build();

    let _webview2 = monitors.get(1).map(|monitor| {
        WebviewBuilder::new()
            .title("Screen 2")
            .monitor(monitor)
            .fullscreen()
            .load_rust_embed::<WebAssets>()
            .load_url("/screen2.html")
            .build()
    });

    let _webview3 = monitors.get(2).map(|monitor| {
        WebviewBuilder::new()
            .title("Screen 3")
            .monitor(monitor)
            .fullscreen()
            .load_rust_embed::<WebAssets>()
            .load_url("/screen3.html")
            .build()
    });

    event_loop.run(move |_event| {});
}
