/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A tiny webview ipc example

use serde::{Deserialize, Serialize};
use tiny_webview::{Event, EventLoop, EventLoopBuilder, LogicalSize, Webview, WebviewBuilder};

const APP_HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/app.min.html"));

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
enum IpcMessage {
    #[serde(rename = "hello")]
    Hello { name: String },
}

fn main() {
    let mut event_loop = EventLoopBuilder::build();

    let mut webview = WebviewBuilder::new()
        .title("Webview IPC Example")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state(true)
        .force_dark_mode(true)
        .load_html(APP_HTML)
        .build();

    event_loop.run(move |event| match event {
        // Window events
        Event::WindowCreated => {
            println!("Window created");
        }
        Event::WindowMoved(point) => {
            println!("Window moved: {:?}", point);
        }
        Event::WindowResized(size) => {
            println!("Window resized: {}x{}", size.width, size.height);
            webview.set_title(format!(
                "Webview IPC Example ({}x{})",
                size.width, size.height
            ));
        }
        Event::WindowClosed => {
            println!("Window closed");
        }

        // Page events
        Event::PageLoadStarted => {
            println!("Page load started");
        }
        Event::PageLoadFinished => {
            println!("Page load finished");
            let message = IpcMessage::Hello {
                name: "Webview".to_string(),
            };
            webview.send_ipc_message(
                serde_json::to_string(&message).expect("Should serialize message"),
            );
        }
        Event::PageMessageReceived(message) => {
            match serde_json::from_str(&message).expect("Can't parse message") {
                IpcMessage::Hello { name } => {
                    println!("Hello, {}!", name);
                }
            }
        }
    });
}
