/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A Todo GUI example

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use std::fs;

use bwebview::{Event, EventLoopBuilder, LogicalSize, WebviewBuilder};
use directories::ProjectDirs;
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
struct Todo {
    id: Uuid,
    text: String,
    completed: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum IpcMessage {
    GetTodos,
    GetTodosResponse { todos: Vec<Todo> },
    UpdateTodos { todos: Vec<Todo> },
}

#[derive(Embed)]
#[folder = "$OUT_DIR/web"]
struct WebAssets;

fn main() {
    let event_loop = EventLoopBuilder::new()
        .app_id("nl", "bplaat", "TodoApp")
        .build();

    let mut webview = WebviewBuilder::new()
        .title("Todo App")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state()
        .load_rust_embed::<WebAssets>()
        .build();

    let project_dirs = ProjectDirs::from("nl", "bplaat", "TodoApp").expect("Can't get dirs");
    let config_dir = project_dirs.config_dir();
    fs::create_dir_all(&config_dir).expect("Can't create config directory");
    let todos_config_path = config_dir.join("todos.json");

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
