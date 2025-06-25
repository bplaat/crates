/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use std::thread;
use std::time::Duration;

use rust_embed::Embed;
use small_websocket::Message;
use tiny_webview::{Event, EventLoop, EventLoopBuilder, LogicalSize, WebviewBuilder};

use crate::dmx::DMX_STATE;
use crate::ipc::IpcMessage;

mod config;
mod dmx;
mod ipc;
mod usb;

#[derive(Embed)]
#[folder = "$OUT_DIR/web"]
struct WebAssets;

fn ipc_message_received(message: &str) {
    let mut dmx_state = DMX_STATE.lock().expect("Failed to lock DMX state");
    let message = serde_json::from_str(message).expect("Failed to parse IPC message");
    println!("[RUST] Received IPC message: {:?}", message);
    match message {
        IpcMessage::SetColor { color } => dmx_state.color = color,
        IpcMessage::SetToggleColor { color } => dmx_state.toggle_color = color,
        IpcMessage::SetToggleSpeed { speed } => {
            dmx_state.toggle_speed = speed.map(Duration::from_millis);
            dmx_state.is_toggle_color = speed.is_some();
        }
        IpcMessage::SetStrobeSpeed { speed } => {
            dmx_state.strobe_speed = speed.map(Duration::from_millis);
            dmx_state.is_strobe = speed.is_some();
        }
        IpcMessage::SetMode { mode } => dmx_state.mode = mode,
    }
}

fn main() {
    let mut event_loop = EventLoopBuilder::build();

    let mut _webview = WebviewBuilder::new()
        .title("BassieLight")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state(true)
        .force_dark_mode(true)
        .load_rust_embed::<WebAssets>()
        .internal_http_serve_handle(|req| {
            if req.url.path() == "/ipc" {
                return Some(small_websocket::upgrade(req, |mut ws| {
                    loop {
                        let message = match ws.recv() {
                            Ok(message) => message,
                            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => continue,
                            Err(err) => panic!("WebSocket recv error: {}", err),
                        };
                        match message {
                            Message::Text(text) => ipc_message_received(&text),
                            Message::Close(_, _) => break,
                            _ => {}
                        }
                    }
                }));
            }
            None
        })
        .build();

    let config = config::load_config("config.json").expect("Can't load config.json");
    event_loop.run(move |event| match event {
        Event::PageLoadFinished => {
            let config = config.clone();
            thread::spawn(move || {
                if let Some(device) = usb::find_udmx_device() {
                    dmx::dmx_thread(device, config);
                } else {
                    eprintln!("[RUST] No uDMX device found");
                }
            });
        }
        Event::PageMessageReceived(message) => ipc_message_received(&message),
        _ => {}
    });
}
