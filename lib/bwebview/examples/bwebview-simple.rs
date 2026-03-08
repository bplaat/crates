/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple bwebview example

use bwebview::{
    EventLoop, EventLoopBuilder, EventLoopHandler, Key, Modifiers, WebviewBuilder, WebviewHandler,
    Window, Webview, WindowBuilder, WindowHandler,
};

#[derive(Default)]
struct App {
    window: Option<Window>,
    _webview: Option<Webview>,
}

impl EventLoopHandler for App {
    fn on_init(&mut self) {
        let window = WindowBuilder::new()
            .title("Webview Simple Example")
            .handler(self)
            .build();
        let webview = WebviewBuilder::new(&window)
            .load_url("https://github.com/bplaat/crates")
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

    fn on_key_down(&mut self, _window: &mut Window, key: Key, _mods: Modifiers) {
        if key == Key::Escape {
            EventLoop::quit();
        }
    }
}

impl WebviewHandler for App {}

fn main() {
    let mut app = App::default();
    EventLoopBuilder::new().handler(&mut app).build().run();
}
