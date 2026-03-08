/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple bwebview multi-window example

use bwebview::{
    EventLoop, EventLoopBuilder, EventLoopHandler, LogicalPoint, LogicalSize, WebviewBuilder,
    WebviewHandler, Window, Webview, WindowBuilder, WindowHandler,
};

#[derive(Default)]
struct App {
    window_a: Option<Window>,
    window_b: Option<Window>,
    _webview_a: Option<Webview>,
    _webview_b: Option<Webview>,
}

impl EventLoopHandler for App {
    fn on_init(&mut self) {
        let window_a = WindowBuilder::new()
            .title("Window A")
            .position(LogicalPoint::new(100.0, 100.0))
            .size(LogicalSize::new(1024.0, 768.0))
            .handler(self)
            .build();
        let webview_a = WebviewBuilder::new(&window_a)
            .load_url("https://example.com")
            .build();

        let window_b = WindowBuilder::new()
            .title("Window B")
            .position(LogicalPoint::new(100.0 + 1024.0, 100.0))
            .size(LogicalSize::new(1024.0, 768.0))
            .handler(self)
            .build();
        let webview_b = WebviewBuilder::new(&window_b)
            .load_url("https://example.com")
            .build();

        self.window_a = Some(window_a);
        self.window_b = Some(window_b);
        self._webview_a = Some(webview_a);
        self._webview_b = Some(webview_b);
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
