/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A Todo GUI example

#![forbid(unsafe_code)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{env, fs};

use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use tiny_webview::{Event, EventLoop, EventLoopBuilder, LogicalSize, Webview, WebviewBuilder};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
struct Todo {
    id: Uuid,
    text: String,
    completed: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
enum IpcMessage {
    #[serde(rename = "get-todos")]
    GetTodos,
    #[serde(rename = "get-todos-response")]
    GetTodosResponse { todos: Vec<Todo> },
    #[serde(rename = "update-todos")]
    UpdateTodos { todos: Vec<Todo> },
}

#[derive(Embed)]
#[folder = "$OUT_DIR/web"]
struct WebAssets;

fn main() {
    let mut event_loop = EventLoopBuilder::build();

    let mut webview = WebviewBuilder::new()
        .title("Todo App")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state(true)
        .force_dark_mode(true)
        .load_rust_embed::<WebAssets>()
        .build();

    let todos_config_path = format!(
        "{}/{}/{}",
        dirs::config_dir().expect("Can't get config dir").display(),
        env!("CARGO_PKG_NAME"),
        "todos.json"
    );
    if let Some(parent) = std::path::Path::new(&todos_config_path).parent() {
        fs::create_dir_all(parent).expect("Can't create config directory");
    }

    event_loop.run(move |event| {
        if let Event::PageMessageReceived(message) = event {
            match serde_json::from_str(&message).expect("Can't parse message") {
                IpcMessage::GetTodos => {
                    let todos: Vec<Todo> = fs::read_to_string(&todos_config_path)
                        .ok()
                        .and_then(|data| serde_json::from_str(&data).ok())
                        .unwrap_or_default();
                    let response = IpcMessage::GetTodosResponse { todos };
                    webview.send_ipc_message(
                        serde_json::to_string(&response).expect("Failed to serialize response"),
                    );
                }
                IpcMessage::UpdateTodos { todos } => {
                    fs::write(
                        &todos_config_path,
                        serde_json::to_string(&todos).expect("Failed to serialize todos"),
                    )
                    .expect("Failed to write todos to file");
                }
                _ => unimplemented!(),
            }
        }
    });
}
