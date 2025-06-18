/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![allow(non_upper_case_globals)]

use std::thread;
use std::time::Duration;

use rust_embed::Embed;
use tiny_webview::{Event, LogicalSize, Webview, WebviewBuilder};

use crate::ipc::IpcMessage;

mod config;
mod dmx;
mod ipc;
mod usb;

#[derive(Embed)]
#[folder = "$OUT_DIR/web"]
struct WebAssets;

fn main() {
    let mut webview = WebviewBuilder::new()
        .title("BassieLight")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state(true)
        .force_dark_mode(true)
        .load_rust_embed::<WebAssets>()
        .build();

    webview.run(|_, event| match event {
        Event::PageLoadFinished => {
            thread::spawn(move || {
                let config = config::load_config("config.json").expect("Can't load config.json");
                let device = usb::find_udmx_device().expect("Can't find uDMX device");
                dmx::dmx_thread(device, config);
            });
        }
        Event::PageMessageReceived(message) => {
            match serde_json::from_str::<IpcMessage>(&message).expect("Can't parse message") {
                IpcMessage::SetColor { color } => unsafe {
                    dmx::x_color = color;
                },
                IpcMessage::SetToggleColor { color } => unsafe {
                    dmx::x_toggle_color = color;
                },
                IpcMessage::SetToggleSpeed { speed } => unsafe {
                    dmx::x_toggle_speed = speed.map(Duration::from_millis);
                    dmx::x_is_toggle_color = speed.is_some();
                },
                IpcMessage::SetStrobeSpeed { speed } => unsafe {
                    dmx::x_strobe_speed = speed.map(Duration::from_millis);
                    dmx::x_is_strobe = speed.is_some();
                },
            }
        }
        _ => {}
    });
}
