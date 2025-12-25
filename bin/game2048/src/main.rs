/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use bwebview::{EventLoopBuilder, LogicalSize, WebviewBuilder};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "web"]
struct WebAssets;

fn main() {
    let event_loop = EventLoopBuilder::new()
        .app_id("nl", "bplaat", "2048")
        .build();

    #[allow(unused_mut)]
    let mut webview_builder = WebviewBuilder::new()
        .title("2048")
        .size(LogicalSize::new(600.0, 850.0))
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
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    let mut webview = webview_builder.build();

    #[cfg(target_os = "macos")]
    webview.add_user_script(
        format!(
            "document.documentElement.style.setProperty('--macos-titlebar-height', '{}px');",
            webview.macos_titlebar_size().height
        ),
        bwebview::InjectionTime::DocumentStart,
    );

    event_loop.run(move |_event| {});
}
