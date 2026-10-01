/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use bwebview::{WebviewBuilder, WebviewEvent};
use bwindow::{
    Event, EventLoopBuilder, FileDialog, LogicalSize, MessageButtons, MessageDialog, MessageLevel,
    Theme, Window, WindowBuilder,
};
use log::{info, warn};
use rust_embed::Embed;
use serde::Deserialize;
use small_http::Response;
use small_websocket::Message;

use crate::config::{CONFIG, Config};
use crate::ipc::{IPC_CONNECTIONS, IpcConnection, ipc_message_handler};
use crate::stage::{STAGE, Stage};

mod config;
mod dmx;
mod ipc;
mod stage;
mod usb;

// MARK: Internal HTTP server
const PORT: u16 = 39027;

#[derive(Embed)]
#[folder = "$OUT_DIR/web"]
struct WebAssets;

// MARK: Window messages
/// Centers the macOS window buttons in the 52px high header bar
#[cfg(target_os = "macos")]
const TRAFFIC_LIGHT_POSITION: bwindow::LogicalPoint = bwindow::LogicalPoint::new(19.0, 19.0);

/// IPC messages that need the native window, only accepted from the webview
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum WindowMessage {
    NewStage,
    OpenStage,
    SaveStageAs,
    #[cfg(target_os = "macos")]
    StartWindowDrag,
    #[cfg(target_os = "macos")]
    TitlebarDoubleClick,
}

fn stage_file_dialog<'a>(window: &'a Window, title: &str) -> FileDialog<'a> {
    FileDialog::new()
        .parent(window)
        .title(title)
        .directory(Config::dir())
        .add_filter("Stage files", &["json"])
}

fn show_error(window: &Window, title: &str, error: &str) {
    MessageDialog::new()
        .parent(window)
        .title(title)
        .description(error)
        .level(MessageLevel::Error)
        .buttons(MessageButtons::Ok)
        .show();
}

fn save_and_open_stage(window: &Window, path: PathBuf, stage: Stage) {
    match stage.save(&path) {
        Ok(()) => ipc::open_stage(path, stage),
        Err(error) => show_error(window, "Can't save stage", &error.to_string()),
    }
}

fn handle_window_message(window: &mut Window, message: WindowMessage) {
    match message {
        WindowMessage::NewStage => {
            if let Some(path) = stage_file_dialog(window, "New Stage")
                .file_name("stage.json")
                .save_file()
            {
                save_and_open_stage(window, path, Stage::default());
            }
        }
        WindowMessage::OpenStage => {
            if let Some(path) = stage_file_dialog(window, "Open Stage").pick_file() {
                match Stage::load(&path, config::dmx_length()) {
                    Ok(stage) => ipc::open_stage(path, stage),
                    Err(error) => show_error(window, "Can't open stage", &error),
                }
            }
        }
        WindowMessage::SaveStageAs => {
            let (file_name, stage) = {
                let open_stage = STAGE.lock().expect("Failed to lock stage");
                let open = open_stage.as_ref().expect("Stage not loaded");
                let file_name = open
                    .path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned());
                (file_name, open.stage.clone())
            };
            if let Some(path) = stage_file_dialog(window, "Save Stage As")
                .file_name(file_name.as_deref().unwrap_or("stage.json"))
                .save_file()
            {
                save_and_open_stage(window, path, stage);
            }
        }
        #[cfg(target_os = "macos")]
        WindowMessage::StartWindowDrag => window.macos_start_window_drag(),
        #[cfg(target_os = "macos")]
        WindowMessage::TitlebarDoubleClick => window.macos_perform_titlebar_double_click(),
    }
}

// MARK: Main
pub(crate) enum AppEvent {
    Webview(bwindow::WindowId, WebviewEvent),
    UserEvent(String),
}

fn main() {
    // Init logger
    simple_logger::init_with_level(if cfg!(debug_assertions) {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    })
    .expect("Failed to init logger");

    // Create event loop
    let event_loop = EventLoopBuilder::new()
        .with_user_event::<AppEvent>()
        .app_id("nl", "bplaat", "BassieLight")
        .build();

    // Load config and the last stage
    let mut config = Config::load();
    let open_stage = config.load_stage();
    config.last_stage = Some(open_stage.path.clone());
    if let Err(error) = config.save() {
        warn!("Can't save config.json: {error}");
    }
    info!("Config: {config:?}");
    *CONFIG.lock().expect("Failed to lock config") = Some(config);
    *STAGE.lock().expect("Failed to lock stage") = Some(open_stage);

    // Start DMX thread
    thread::Builder::new()
        .name("dmx".to_string())
        .spawn(dmx::dmx_thread)
        .expect("Failed to spawn DMX thread");

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
    thread::Builder::new()
        .name("http-server".to_string())
        .spawn(move || {
            small_http::serve_single_threaded(listener, move |req| {
                let mut path = req.url.path().to_string();
                if path.ends_with('/') {
                    path = format!("{path}index.html");
                }

                if req.url.path() == "/ipc" {
                    return small_websocket::upgrade(req, |mut ws| {
                        if let Err(error) = ws.set_write_timeout(Some(Duration::from_millis(250))) {
                            warn!("Can't configure WebSocket write timeout: {error}");
                            return;
                        }
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
                                Err(err) => {
                                    warn!("WebSocket recv error: {err}");
                                    break;
                                }
                            };
                            match message {
                                Some(Message::Close(_, _)) => break,
                                Some(Message::Text(text))
                                    if !ipc_message_handler(
                                        IpcConnection::WebSocket(ws.clone()),
                                        &text,
                                    ) =>
                                {
                                    break;
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
                    let file = WebAssets::get("index.html").expect("index.html not found");
                    Response::with_header("Content-Type", "text/html").body(file.data)
                }
            });
        })
        .expect("Failed to spawn HTTP server thread");

    // Create webview
    #[allow(unused_mut)]
    let mut window_builder = WindowBuilder::new()
        .title("BassieLight")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state()
        .theme(Theme::Dark)
        .background_color(0x18181b);
    #[cfg(target_os = "macos")]
    {
        window_builder = window_builder
            .macos_titlebar_style(bwindow::MacosTitlebarStyle::Hidden)
            .macos_traffic_light_position(TRAFFIC_LIGHT_POSITION);
    }
    let mut window = window_builder.build();

    let mut webview = WebviewBuilder::new(&window)
        .on_event(event_loop.create_proxy(), AppEvent::Webview)
        .load_url(&url)
        .build();

    let event_loop_proxy = Arc::new(event_loop.create_proxy());
    event_loop.run(move |event| match event {
        // Window events
        Event::UserEvent(AppEvent::Webview(_window_id, WebviewEvent::PageTitleChange(title))) => {
            window.set_title(title)
        }
        #[cfg(target_os = "macos")]
        Event::Window(_, bwindow::WindowEvent::MacosFullscreenChange(is_fullscreen)) => {
            if is_fullscreen {
                webview.evaluate_script("document.body.classList.add('is-fullscreen');");
            } else {
                webview.evaluate_script("document.body.classList.remove('is-fullscreen');");
            }
        }

        // IPC events
        Event::UserEvent(AppEvent::Webview(_window_id, WebviewEvent::PageLoadStart)) => {
            IPC_CONNECTIONS
                .lock()
                .expect("Failed to lock IPC connections")
                .push(IpcConnection::WebviewIpc(event_loop_proxy.clone()));
        }
        Event::UserEvent(AppEvent::Webview(_window_id, WebviewEvent::MessageReceive(message))) => {
            if let Ok(message) = serde_json::from_str::<WindowMessage>(&message) {
                handle_window_message(&mut window, message);
            } else {
                ipc_message_handler(
                    IpcConnection::WebviewIpc(event_loop_proxy.clone()),
                    &message,
                );
            }
        }
        Event::UserEvent(AppEvent::UserEvent(data)) => webview.send_ipc_message(&data),

        _ => {}
    });
}
