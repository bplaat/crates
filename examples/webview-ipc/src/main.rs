/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A webview ipc example

use serde::{Deserialize, Serialize};
use webview::{Event, WebviewBuilder};

const APP_HTML: &[u8] = include_bytes!("../app.html");

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum IpcMessage {
    #[serde(rename = "hello")]
    Hello { name: String },
}

fn main() {
    let mut webview = WebviewBuilder::new()
        .title("Webview IPC Example")
        .size(1024, 768)
        .min_size(640, 480)
        .remember_window_state(true)
        .enable_ipc(true)
        .html(std::str::from_utf8(APP_HTML).expect("Not valid UTF-8"))
        .build();

    webview.run(|webview, event| match event {
        Event::PageLoaded => {
            let message = IpcMessage::Hello {
                name: "WebView".to_string(),
            };
            webview.send_ipc_message(
                serde_json::to_string(&message).expect("Should serialize message"),
            );
        }
        Event::IpcMessageReceived(message) => {
            match serde_json::from_str(&message).expect("Can't parse message") {
                IpcMessage::Hello { name } => {
                    println!("Hello, {}!", name);
                }
            }
        }
    });
}
