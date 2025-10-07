/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A Todo GUI example

#![forbid(unsafe_code)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{env, fs};

use bwebview::dpi::LogicalSize;
use bwebview::event_loop::{EventLoopBuilder, EventLoopHandler};
use bwebview::webview::{Webview, WebviewBuilder, WebviewHandler};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// MARK: Assets
#[derive(Embed)]
#[folder = "$OUT_DIR/web"]
struct WebAssets;

// MARK: IPC Messages
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

// MARK: App
#[derive(Default)]
struct App {
    webview: Option<Webview>,
}

impl App {
    fn config_path() -> String {
        format!(
            "{}/{}/{}",
            dirs::config_dir().expect("Can't get config dir").display(),
            env!("CARGO_PKG_NAME"),
            "todos.json"
        )
    }
}

impl EventLoopHandler for App {
    fn activate(&mut self, _event_loop: &mut EventLoop) {
        if let Some(parent) = std::path::Path::new(&Self::config_path()).parent() {
            fs::create_dir_all(parent).expect("Can't create config directory");
        }

        self.webview = Some(
            WebviewBuilder::new()
                .title("Todo App")
                .size(LogicalSize::new(1024.0, 768.0))
                .min_size(LogicalSize::new(640.0, 480.0))
                .center()
                .remember_window_state()
                .load_rust_embed::<WebAssets>()
                // .handler(&mut self)
                .build(),
        );
    }
}

impl WebviewHandler for App {
    fn ipc_message(&mut self, webview: &mut Webview, message: String) {
        match serde_json::from_str(&message).expect("Can't parse message") {
            IpcMessage::GetTodos => {
                let todos: Vec<Todo> = fs::read_to_string(&Self::config_path())
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
                    &Self::config_path(),
                    serde_json::to_string(&todos).expect("Failed to serialize todos"),
                )
                .expect("Failed to write todos to file");
            }
            _ => unimplemented!(),
        }
    }
}

// MARK: Main
fn main() {
    let mut app = App::default();
    let event_loop = EventLoopBuilder::new()
        .app_id("com.example.todo")
        .handler(&mut app)
        .build();
    event_loop.run();
}
