/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A bwebview background example

use bwebview::{EventLoopBuilder, Theme, WebviewBuilder};

#[allow(unused_mut)]
fn main() {
    let event_loop = EventLoopBuilder::new().build();

    let mut webview_builder = WebviewBuilder::new()
        .title("Webview Background Example")
        .background_color(0x05445e)
        .theme(Theme::Dark)
        .center()
        .load_html("<body style=\"height:100vh;margin:0;display:flex;align-items:center;justify-content:center\"><h1 style=\"color:#fff\">Hello Webview!</h1>");
    #[cfg(target_os = "macos")]
    {
        webview_builder =
            webview_builder.macos_titlebar_style(bwebview::MacosTitlebarStyle::Transparent);
    }
    let _webview = webview_builder.build();

    event_loop.run(|_| {});
}
