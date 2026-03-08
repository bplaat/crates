/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A Todo GUI example

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use std::fs;

use bwebview::{
    EventLoop, EventLoopBuilder, EventLoopHandler, LogicalSize, Webview, WebviewBuilder,
    WebviewHandler, Window, WindowBuilder, WindowHandler,
};
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
#[folder = "web"]
struct WebAssets;

struct App {
    window: Option<Window>,
    webview: Option<Webview>,
    todos_config_path: std::path::PathBuf,
}

impl App {
    fn new() -> Self {
        let project_dirs = ProjectDirs::from("nl", "bplaat", "TodoApp").expect("Can't get dirs");
        let config_dir = project_dirs.config_dir().to_path_buf();
        fs::create_dir_all(&config_dir).expect("Can't create config directory");
        Self {
            window: None,
            webview: None,
            todos_config_path: config_dir.join("todos.json"),
        }
    }
}

impl EventLoopHandler for App {
    fn on_init(&mut self) {
        let window = WindowBuilder::new()
            .title("Todo App")
            .size(LogicalSize::new(1024.0, 768.0))
            .min_size(LogicalSize::new(640.0, 480.0))
            .center()
            .remember_window_state()
            .handler(self)
            .build();

        let webview = WebviewBuilder::new(&window)
            .load_rust_embed::<WebAssets>()
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
}

impl WebviewHandler for App {
    fn on_message(&mut self, _webview: &mut Webview, message: String) {
        match serde_json::from_str(&message).expect("Can't parse message") {
            IpcMessage::GetTodos => {
                let todos: Vec<Todo> = fs::read_to_string(&self.todos_config_path)
                    .ok()
                    .and_then(|data| serde_json::from_str(&data).ok())
                    .unwrap_or_default();
                let response = IpcMessage::GetTodosResponse { todos };
                if let Some(webview) = self.webview.as_mut() {
                    webview.send_ipc_message(
                        serde_json::to_string(&response).expect("Failed to serialize response"),
                    );
                }
            }
            IpcMessage::UpdateTodos { todos } => {
                fs::write(
                    &self.todos_config_path,
                    serde_json::to_string(&todos).expect("Failed to serialize todos"),
                )
                .expect("Failed to write todos to file");
            }
            _ => unimplemented!(),
        }
    }
}

fn main() {
    let mut app = App::new();
    EventLoopBuilder::new()
        .app_id("nl", "bplaat", "TodoApp")
        .handler(&mut app)
        .build()
        .run();
}
