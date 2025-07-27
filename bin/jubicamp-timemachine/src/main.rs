/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Time machine software for the JubiCamp 2025 scouting camp

#![forbid(unsafe_code)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rust_embed::Embed;
use small_http::{Request, Response};
use small_websocket::Message;
use tiny_webview::{Event, EventLoopBuilder, WebviewBuilder};

use crate::ipc::{IPC_CONNECTIONS, IpcConnection, ipc_message_handler};

mod ipc;

const PORT: u16 = 25198;

#[derive(Embed)]
#[folder = "web"]
struct WebAssets;

fn internal_http_server_handle(req: &Request) -> Option<Response> {
    if req.url.path() == "/ipc" {
        return Some(small_websocket::upgrade(req, |mut ws| {
            IPC_CONNECTIONS
                .lock()
                .expect("Failed to lock IPC connections")
                .push(IpcConnection::WebSocket(ws.clone()));
            loop {
                let message = match ws.recv_non_blocking() {
                    Ok(message) => message,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(err) => panic!("WebSocket recv error: {err}"),
                };
                match message {
                    Some(Message::Close(_, _)) => break,
                    Some(Message::Text(text)) => {
                        ipc_message_handler(IpcConnection::WebSocket(ws.clone()), &text);
                    }
                    None => {
                        // FIXME: Create async framework no not micro sleep threads
                        thread::sleep(Duration::from_millis(100));
                    }
                    _ => {}
                }
            }
            IPC_CONNECTIONS
                .lock()
                .expect("Failed to lock IPC connections")
                .retain(|conn| conn != &IpcConnection::WebSocket(ws.clone()));
        }));
    }
    None
}

fn main() {
    let event_loop = EventLoopBuilder::build();
    let mut monitors = event_loop.available_monitors();
    monitors.sort_by_key(|m| !m.is_primary());

    let mut webview1 = WebviewBuilder::new()
        .title("Screen 1")
        .monitor(&monitors[0])
        .fullscreen()
        .internal_http_server_port(PORT)
        .internal_http_server_expose()
        .internal_http_server_handle(internal_http_server_handle)
        .load_rust_embed::<WebAssets>()
        .load_url("/matrix.html")
        .build();

    let mut webview2 = monitors.get(1).map(|monitor| {
        WebviewBuilder::new()
            .title("Screen 2")
            .monitor(monitor)
            .fullscreen()
            .load_rust_embed::<WebAssets>()
            .load_url("/shader.html")
            .build()
    });

    let mut webview3 = monitors.get(2).map(|monitor| {
        WebviewBuilder::new()
            .title("Screen 3")
            .monitor(monitor)
            .fullscreen()
            .load_rust_embed::<WebAssets>()
            .load_url("/terminal.html")
            .build()
    });

    println!(
        "Webserver running at: {}",
        webview1.url().expect("Should be some")
    );

    let event_loop_proxy = Arc::new(event_loop.create_proxy());
    event_loop.run(move |event| match event {
        Event::PageLoadFinished => {
            IPC_CONNECTIONS
                .lock()
                .expect("Failed to lock IPC connections")
                .push(IpcConnection::WebviewIpc(event_loop_proxy.clone()));
        }
        Event::PageMessageReceived(message) => ipc_message_handler(
            IpcConnection::WebviewIpc(event_loop_proxy.clone()),
            &message,
        ),
        Event::UserEvent(data) => {
            webview1.send_ipc_message(&data);
            if let Some(ref mut webview) = webview2 {
                webview.send_ipc_message(&data);
            }
            if let Some(ref mut webview) = webview3 {
                webview.send_ipc_message(&data);
            }
        }
        _ => {}
    });
}
