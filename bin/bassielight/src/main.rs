/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rust_embed::Embed;
use small_http::{Request, Response};
use small_websocket::Message;
use tiny_webview::{Event, EventLoopBuilder, LogicalSize, Theme, WebviewBuilder};

use crate::config::Config;
use crate::ipc::{IPC_CONNECTIONS, IpcConnection, ipc_message_handler};

mod config;
mod dmx;
mod ipc;
mod usb;

const PORT: u16 = 39027;
pub(crate) static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

#[derive(Embed)]
#[folder = "$OUT_DIR/web"]
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

// MARK: Main
#[allow(unused_mut)]
fn main() {
    // Load config
    let config_path = if cfg!(not(debug_assertions)) {
        format!(
            "{}/BassieLight/config.json",
            dirs::config_dir()
                .expect("Can't find config directory")
                .display()
        )
    } else {
        "config.json".to_string()
    };
    let config = config::load_config(config_path);
    *CONFIG.lock().expect("Failed to lock config") = Some(config);

    // Create webview
    let event_loop = EventLoopBuilder::build();

    let mut webview = WebviewBuilder::new()
        .title("BassieLight")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state()
        .theme(Theme::Dark)
        .background_color(0x1a1a1a)
        .load_rust_embed::<WebAssets>()
        .internal_http_server_port(PORT)
        .internal_http_server_expose()
        .internal_http_server_handle(internal_http_server_handle)
        .build();

    let event_loop_proxy = Arc::new(event_loop.create_proxy());
    event_loop.run(move |event| match event {
        Event::PageLoadFinished => {
            IPC_CONNECTIONS
                .lock()
                .expect("Failed to lock IPC connections")
                .push(IpcConnection::WebviewIpc(event_loop_proxy.clone()));

            thread::spawn(move || {
                if let Some(device) = usb::find_udmx_device() {
                    let config = CONFIG
                        .lock()
                        .expect("Failed to lock config")
                        .clone()
                        .expect("Config is not set");
                    dmx::dmx_thread(device, config);
                } else {
                    eprintln!("[RUST] No uDMX device found");
                }
            });
        }
        Event::PageMessageReceived(message) => ipc_message_handler(
            IpcConnection::WebviewIpc(event_loop_proxy.clone()),
            &message,
        ),
        Event::UserEvent(data) => webview.send_ipc_message(&data),
        _ => {}
    });
}
