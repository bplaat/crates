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

use bwebview::{
    EventLoop, EventLoopBuilder, EventLoopHandler, EventLoopProxy, LogicalSize, Theme,
    WebviewBuilder, WebviewHandler, Window, Webview, WindowBuilder, WindowHandler,
};
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

// MARK: App
struct App {
    proxy: Option<Arc<EventLoopProxy>>,
    window: Option<Window>,
    webview: Option<Webview>,
    url: String,
}

impl App {
    fn new(url: String) -> Self {
        Self {
            proxy: None,
            window: None,
            webview: None,
            url,
        }
    }
}

impl EventLoopHandler for App {
    fn on_init(&mut self) {
        #[allow(unused_mut)]
        let mut window_builder = WindowBuilder::new()
            .title("BassieLight")
            .size(LogicalSize::new(1024.0, 768.0))
            .min_size(LogicalSize::new(640.0, 480.0))
            .center()
            .remember_window_state()
            .theme(Theme::Dark)
            .background_color(0x1a1a1a)
            .handler(self);
        #[cfg(target_os = "macos")]
        {
            window_builder =
                window_builder.macos_titlebar_style(bwebview::MacosTitlebarStyle::Hidden);
        }
        let window = window_builder.build();

        #[allow(unused_mut)]
        let mut webview = WebviewBuilder::new(&window)
            .load_url(&self.url)
            .handler(self)
            .build();

        #[cfg(target_os = "macos")]
        webview.add_user_script(
            format!(
                "document.documentElement.style.setProperty('--macos-titlebar-height', '{}px');",
                window.macos_titlebar_size().height
            ),
            bwebview::InjectionTime::DocumentStart,
        );

        self.window = Some(window);
        self.webview = Some(webview);
    }

    fn on_user_event(&mut self, data: String) {
        if let Some(webview) = self.webview.as_mut() {
            webview.send_ipc_message(&data);
        }
    }
}

impl WindowHandler for App {
    fn on_close(&mut self, _window: &mut Window) -> bool {
        EventLoop::quit();
        true
    }

    #[cfg(target_os = "macos")]
    fn on_fullscreen_change(&mut self, _window: &mut Window, is_fullscreen: bool) {
        if let Some(webview) = self.webview.as_mut() {
            if is_fullscreen {
                webview.evaluate_script("document.body.classList.add('is-fullscreen');");
            } else {
                webview.evaluate_script("document.body.classList.remove('is-fullscreen');");
            }
        }
    }
}

impl WebviewHandler for App {
    fn on_title_change(&mut self, _webview: &mut Webview, title: String) {
        if let Some(window) = self.window.as_mut() {
            window.set_title(title);
        }
    }

    fn on_load_start(&mut self, _webview: &mut Webview) {
        if let Some(proxy) = self.proxy.clone() {
            IPC_CONNECTIONS
                .lock()
                .expect("Failed to lock IPC connections")
                .push(IpcConnection::WebviewIpc(proxy));
        }
    }

    fn on_message(&mut self, _webview: &mut Webview, message: String) {
        if let Some(proxy) = self.proxy.clone() {
            ipc_message_handler(IpcConnection::WebviewIpc(proxy), &message);
        }
    }
}

// MARK: Main
fn main() {
    // Init logger
    simple_logger::init_with_level(if cfg!(debug_assertions) {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    })
    .expect("Failed to init logger");

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

    // Build event loop and create proxy before starting
    let mut app = App::new(url.clone());
    let event_loop = EventLoopBuilder::new()
        .app_id("nl", "bplaat", "BassieLight")
        .handler(&mut app)
        .build();

    let event_loop_proxy = Arc::new(event_loop.create_proxy());
    app.proxy = Some(event_loop_proxy.clone());

    // Start internal HTTP server thread
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
                let file = WebAssets::get("index.html").expect("index.html not found");
                Response::with_header("Content-Type", "text/html").body(file.data)
            }
        });
    });

    event_loop.run();
}
