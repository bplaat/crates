/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A Todo GUI example

#![windows_subsystem = "windows"]

use std::net::{Ipv4Addr, TcpListener};
use std::{fs, thread};

use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use small_http::{Response, Status};
use tiny_webview::{Event, LogicalSize, Webview, WebviewBuilder};
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

fn mime_guess(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html",
        Some("js") => "application/javascript",
        Some("css") => "text/css",
        _ => "application/octet-stream",
    }
}

fn main() {
    // Spawn a local http server
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .unwrap_or_else(|_| panic!("Can't bind to port"));
    let port = listener
        .local_addr()
        .expect("Can't get local http server port")
        .port();
    thread::spawn(move || {
        small_http::serve(listener, |req| {
            let path = req.url.path().trim_start_matches('/');
            let path = if path.is_empty() {
                "index.html".to_string()
            } else {
                path.to_string()
            };
            if let Some(asset) = WebAssets::get(&path) {
                Response::with_header("Content-Type", mime_guess(&path)).body(asset.data)
            } else {
                Response::with_status(Status::NotFound).body(b"404 Not Found".to_vec())
            }
        });
    });

    let mut webview = WebviewBuilder::new()
        .title("Todo App")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state(true)
        .force_dark_mode(true)
        .load_url(format!("http://127.0.0.1:{}/", port))
        .build();

    webview.run(|webview, event| {
        if let Event::PageMessageReceived(message) = event {
            match serde_json::from_str(&message).expect("Can't parse message") {
                IpcMessage::GetTodos => {
                    let todos: Vec<Todo> = fs::read_to_string("todos.json")
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
                        "todos.json",
                        serde_json::to_string(&todos).expect("Failed to serialize todos"),
                    )
                    .expect("Failed to write todos to file");
                }
                _ => unimplemented!(),
            }
        }
    });
}
