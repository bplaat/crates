/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A local version of the 2048 game

use bwebview::{EventLoopBuilder, LogicalSize, WebviewBuilder};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "web"]
struct WebAssets;

fn main() {
    let event_loop = EventLoopBuilder::new().app_id("nl.bplaat.2048").build();

    #[allow(unused_mut)]
    let mut webview_builder = WebviewBuilder::new()
        .title("2048")
        .size(LogicalSize::new(600.0, 840.0))
        .resizable(false)
        .background_color(0xfaf8ef)
        .center()
        .remember_window_state()
        .load_rust_embed::<WebAssets>();
    #[cfg(target_os = "macos")]
    {
        webview_builder =
            webview_builder.macos_titlebar_style(bwebview::MacosTitlebarStyle::Hidden);
    }
    let _webview = webview_builder.build();

    event_loop.run(move |_event| {});
}
