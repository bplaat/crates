/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Time machine software for the JubiCamp 2025 scouting camp

#![forbid(unsafe_code)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use rust_embed::Embed;
use tiny_webview::{EventLoopBuilder, LogicalPoint, WebviewBuilder};

#[derive(Embed)]
#[folder = "web"]
struct WebAssets;

fn main() {
    let event_loop = EventLoopBuilder::build();

    let _webview_screen1 = WebviewBuilder::new()
        .title("Screen 1")
        .position(LogicalPoint::new(0.0, 0.0))
        .load_rust_embed::<WebAssets>()
        .load_url("/screen1.html")
        .build();

    let _webview_screen2 = WebviewBuilder::new()
        .title("Screen 2")
        .position(LogicalPoint::new(200.0, 0.0))
        .load_rust_embed::<WebAssets>()
        .load_url("/screen2.html")
        .build();

    let _webview_screen3 = WebviewBuilder::new()
        .title("Screen 3")
        .position(LogicalPoint::new(400.0, 0.0))
        .load_rust_embed::<WebAssets>()
        .load_url("/screen3.html")
        .build();

    event_loop.run(move |_event| {});
}
