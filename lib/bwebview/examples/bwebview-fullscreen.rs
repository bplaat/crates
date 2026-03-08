/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple bwebview fullscreen example

use bwebview::{
    EventLoop, EventLoopBuilder, EventLoopHandler, WebviewBuilder, WebviewHandler, Window, Webview,
    WindowBuilder, WindowHandler,
};

#[derive(Default)]
struct App {
    window: Option<Window>,
    _webview: Option<Webview>,
}

impl EventLoopHandler for App {
    fn on_init(&mut self) {
        let window = WindowBuilder::new()
            .title("Webview Simple Fullscreen Example")
            .fullscreen()
            .handler(self)
            .build();
        let webview = WebviewBuilder::new(&window)
            .load_url("https://example.com")
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
