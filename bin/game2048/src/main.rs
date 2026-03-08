/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use bwebview::{
    EventLoop, EventLoopBuilder, EventLoopHandler, LogicalSize, WebviewBuilder, WebviewHandler,
    Window, Webview, WindowBuilder, WindowHandler,
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "web"]
struct WebAssets;

#[derive(Default)]
struct App {
    window: Option<Window>,
    _webview: Option<Webview>,
}

impl EventLoopHandler for App {
    fn on_init(&mut self) {
        #[allow(unused_mut)]
        let mut window_builder = WindowBuilder::new()
            .title("2048")
            .size(LogicalSize::new(600.0, 850.0))
            .resizable(false)
            .background_color(0xfaf8ef)
            .center()
            .remember_window_state()
            .handler(self);
        #[cfg(target_os = "macos")]
        {
            window_builder =
                window_builder.macos_titlebar_style(bwebview::MacosTitlebarStyle::Hidden);
        }
        let window = window_builder.build();

        #[allow(unused_mut)]
        let mut webview = WebviewBuilder::new(&window)
            .load_rust_embed::<WebAssets>()
            .handler(self)
            .build();

        #[cfg(target_os = "macos")]
        webview.add_user_script(
            format!(
                "document.documentElement.style.setProperty('--macos-titlebar-height', '{}px');",
                window.macos_titlebar_size().height
            ),
            bwebview::InjectionTime::DocumentStart,
        );

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
    EventLoopBuilder::new()
        .app_id("nl", "bplaat", "2048")
        .handler(&mut app)
        .build()
        .run();
}

