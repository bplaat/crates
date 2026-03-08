/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A bwebview background example

use bwebview::{
    EventLoop, EventLoopBuilder, EventLoopHandler, Theme, WebviewBuilder, WebviewHandler, Window,
    Webview, WindowBuilder, WindowHandler,
};

#[derive(Default)]
struct App {
    window: Option<Window>,
    _webview: Option<Webview>,
}

impl EventLoopHandler for App {
    fn on_init(&mut self) {
        #[allow(unused_mut)]
        let mut window_builder = WindowBuilder::new()
            .title("Webview Background Example")
            .background_color(0x05445e)
            .theme(Theme::Dark)
            .center()
            .handler(self);
        #[cfg(target_os = "macos")]
        {
            window_builder = window_builder
                .macos_titlebar_style(bwebview::MacosTitlebarStyle::Transparent);
        }
        let window = window_builder.build();
        let webview = WebviewBuilder::new(&window)
            .load_html("<body style=\"height:100vh;margin:0;display:flex;align-items:center;justify-content:center\"><h1 style=\"color:#fff\">Hello Webview!</h1>")
            .build();
        self.window = Some(window);
        self._webview = Some(webview);
    }
}

impl WindowHandler for App {
    fn on_close(&mut self, _window: &mut Window) -> bool {
        EventLoop::quit();
        true
    }
}

impl WebviewHandler for App {}

fn main() {
    let mut app = App::default();
    EventLoopBuilder::new().handler(&mut app).build().run();
}
