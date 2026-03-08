/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A bwebview ipc example

use bwebview::{
    EventLoop, EventLoopBuilder, EventLoopHandler, LogicalSize, Theme, Webview, WebviewBuilder,
    WebviewHandler, Window, WindowBuilder, WindowHandler,
};
use serde::{Deserialize, Serialize};

const APP_HTML: &str = include_str!("../app.html");

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum IpcMessage {
    Hello { name: String },
}

#[derive(Default)]
struct App {
    window: Option<Window>,
    webview: Option<Webview>,
}

impl EventLoopHandler for App {
    fn on_init(&mut self) {
        let window = WindowBuilder::new()
            .title("Webview IPC Example")
            .size(LogicalSize::new(1024.0, 768.0))
            .min_size(LogicalSize::new(640.0, 480.0))
            .center()
            .remember_window_state()
            .theme(Theme::Dark)
            .handler(self)
            .build();
        let webview = WebviewBuilder::new(&window)
            .load_html(APP_HTML)
            .handler(self)
            .build();
        self.window = Some(window);
        self.webview = Some(webview);
    }
}

impl WindowHandler for App {
    fn on_close(&mut self, _window: &mut Window) -> bool {
        EventLoop::quit();
        true
    }

    fn on_move(&mut self, _window: &mut Window, x: i32, y: i32) {
        println!("Window moved: ({x}, {y})");
    }

    fn on_resize(&mut self, window: &mut Window, width: u32, height: u32) {
        println!("Window resized: {width}x{height}");
        window.set_title(format!("Webview IPC Example ({width}x{height})"));
    }

    #[cfg(target_os = "macos")]
    fn on_fullscreen_change(&mut self, _window: &mut Window, is_fullscreen: bool) {
        println!("Window fullscreen changed: {is_fullscreen}");
    }
}

impl WebviewHandler for App {
    fn on_load_start(&mut self, _webview: &mut Webview) {
        println!("Page load started");
    }

    fn on_load(&mut self, webview: &mut Webview) {
        println!("Page load finished");
        let message = IpcMessage::Hello {
            name: "Webview".to_string(),
        };
        webview
            .send_ipc_message(serde_json::to_string(&message).expect("Should serialize message"));
    }

    fn on_title_change(&mut self, _webview: &mut Webview, title: String) {
        println!("Title changed: {title}");
        if let Some(window) = self.window.as_mut() {
            window.set_title(title);
        }
    }

    fn on_message(&mut self, _webview: &mut Webview, message: String) {
        match serde_json::from_str(&message).expect("Can't parse message") {
            IpcMessage::Hello { name } => {
                println!("Hello, {name}!");
            }
        }
    }
}

fn main() {
    let mut app = App::default();
    EventLoopBuilder::new()
        .app_id("nl", "bplaat", "WebviewIpcExample")
        .handler(&mut app)
        .build()
        .run();
}
