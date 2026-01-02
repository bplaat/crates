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

use bwebview::{Event, EventLoopBuilder, LogicalSize, Theme, WebviewBuilder};
use log::{error, info};
use rust_embed::Embed;
use small_http::Response;
use small_websocket::Message;

use crate::config::Config;
use crate::ipc::{IPC_CONNECTIONS, IpcConnection, ipc_message_handler};

mod config;
mod dmx;
mod ipc;
mod usb;

// MARK: Internal HTTP server
const PORT: u16 = 39027;
pub(crate) static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

#[derive(Embed)]
#[folder = "$OUT_DIR/web"]
struct WebAssets;

// MARK: Main
fn main() {
    // Init logger
    simple_logger::init_with_level(log::LevelFilter::Debug).expect("Failed to init logger");

    // Create event loop
    let event_loop = EventLoopBuilder::new()
        .app_id("nl", "bplaat", "BassieLight")
        .build();

    // Load config
    let config = Config::load();
    info!("Config: {config:?}");
    let cloned_config = config.clone();
    *CONFIG.lock().expect("Failed to lock config") = Some(config);

    // Start DMX thread
    thread::spawn(move || {
        if let Some(device) = usb::find_udmx_device() {
            info!("uDMX device found: {device:?}");
            dmx::dmx_thread(Some(device), cloned_config);
        } else {
            error!("No uDMX device found");
            dmx::dmx_thread(None, cloned_config);
        }
    });

    // Try to get local IP address, fallback to localhost if it fails
    let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, PORT))
        .unwrap_or_else(|_| panic!("Can't start local http server"));
    let local_addr = listener
        .local_addr()
        .expect("Can't get local http server port");
    let url = if let Ok(ip) = local_ip_address::local_ip() {
        format!("http://{}:{}", ip, local_addr.port())
    } else {
        format!("http://127.0.0.1:{}", local_addr.port())
    };

    // Start internal http server thread
    info!("Starting internal HTTP server at {url}");
    thread::spawn(move || {
        small_http::serve_single_threaded(listener, move |req| {
            let mut path = req.url.path().to_string();
            if path.ends_with('/') {
                path = format!("{path}index.html");
            }

            if req.url.path() == "/ipc" {
                return small_websocket::upgrade(req, |mut ws| {
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
                                // FIXME: Create async framework don't do micro sleeps
                                thread::sleep(Duration::from_millis(100));
                            }
                            _ => {}
                        }
                    }
                    IPC_CONNECTIONS
                        .lock()
                        .expect("Failed to lock IPC connections")
                        .retain(|conn| conn != &IpcConnection::WebSocket(ws.clone()));
                });
            }

            if let Some(file) = WebAssets::get(path.trim_start_matches('/')) {
                let mime = mime_guess::from_path(&path).first_or_octet_stream();
                Response::with_header("Content-Type", mime.to_string()).body(file.data)
            } else {
                let file = WebAssets::get("index.html").expect("Should be some");
                Response::with_header("Content-Type", "text/html").body(file.data)
            }
        });
    });

    // Create webview
    #[allow(unused_mut)]
    let mut webview_builder = WebviewBuilder::new()
        .title("BassieLight")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state()
        .theme(Theme::Dark)
        .background_color(0x1a1a1a)
        .load_url(&url);
    #[cfg(target_os = "macos")]
    {
        webview_builder =
            webview_builder.macos_titlebar_style(bwebview::MacosTitlebarStyle::Hidden);
    }
    let mut webview = webview_builder.build();

    #[cfg(target_os = "macos")]
    webview.add_user_script(
        format!(
            "document.documentElement.style.setProperty('--macos-titlebar-height', '{}px');",
            webview.macos_titlebar_size().height
        ),
        bwebview::InjectionTime::DocumentStart,
    );

    let event_loop_proxy = Arc::new(event_loop.create_proxy());
    event_loop.run(move |event| match event {
        // Window events
        Event::PageTitleChanged(title) => webview.set_title(title),
        #[cfg(target_os = "macos")]
        Event::MacosWindowFullscreenChanged(is_fullscreen) => {
            if is_fullscreen {
                webview.evaluate_script("document.body.classList.add('is-fullscreen');");
            } else {
                webview.evaluate_script("document.body.classList.remove('is-fullscreen');");
            }
        }

        // IPC events
        Event::PageLoadStarted => {
            IPC_CONNECTIONS
                .lock()
                .expect("Failed to lock IPC connections")
                .push(IpcConnection::WebviewIpc(event_loop_proxy.clone()));
        }
        Event::PageMessageReceived(message) => ipc_message_handler(
            IpcConnection::WebviewIpc(event_loop_proxy.clone()),
            &message,
        ),
        Event::UserEvent(data) => webview.send_ipc_message(&data),

        _ => {}
    });
}
