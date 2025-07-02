/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A tiny webview background example

use tiny_webview::{EventLoopBuilder, Theme, WebviewBuilder};

#[allow(unused_mut)]
fn main() {
    let event_loop = EventLoopBuilder::build();

    let mut webview_builder = WebviewBuilder::new()
        .title("Webview Background Example")
        .background_color(0x05445e)
        .theme(Theme::Dark)
        .center()
        .load_html("<body style=\"height:100vh;margin:0;display:flex;align-items:center;justify-content:center\"><h1 style=\"color:#fff\">Hello Webview!</h1>");
    #[cfg(target_os = "macos")]
    {
        webview_builder =
            webview_builder.macos_titlebar_style(tiny_webview::MacosTitlebarStyle::Transparent);
    }
    let _webview = webview_builder.build();

    event_loop.run(|_| {});
}
