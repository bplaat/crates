/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A bwebview background example

use bwebview::{EventLoop, Theme, WebviewBuilder, WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();

    #[allow(unused_mut)]
    let mut window_builder = WindowBuilder::new()
        .title("Webview Background Example")
        .background_color(0x05445e)
        .theme(Theme::Dark)
        .center();
    #[cfg(target_os = "macos")]
    {
        window_builder =
            window_builder.macos_titlebar_style(bwebview::MacosTitlebarStyle::Transparent);
    }
    let window = window_builder.build();
    let mut _webview = WebviewBuilder::new(&window)
        .load_html("<body style=\"height:100vh;margin:0;display:flex;align-items:center;justify-content:center\"><h1 style=\"color:#fff\">Hello Webview!</h1>")
        .build();

    event_loop.run(|_| {});
}
