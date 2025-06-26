/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

pub use event::*;
pub use sizes::*;

use crate::platforms::{
    PlatformEventLoop, PlatformEventLoopProxy, PlatformMonitor, PlatformWebview,
};

mod event;
mod platforms;
mod sizes;

// MARK: EventLoopBuilder
/// EventLoop builder
pub struct EventLoopBuilder;

impl EventLoopBuilder {
    /// Create new event loop
    pub fn build() -> EventLoop {
        EventLoop::new(PlatformEventLoop::new())
    }
}

// MARK: EventLoop
pub(crate) trait EventLoopInterface {
    fn available_monitors(&self) -> Vec<PlatformMonitor>;
    fn create_proxy(&self) -> PlatformEventLoopProxy;
    fn run(self, event_handler: impl FnMut(Event) + 'static) -> !;
}

/// Event loop
pub struct EventLoop(PlatformEventLoop);

impl EventLoop {
    fn new(event_loop: PlatformEventLoop) -> Self {
        Self(event_loop)
    }

    /// List available monitors
    pub fn available_monitors(&self) -> Vec<Monitor> {
        self.0
            .available_monitors()
            .into_iter()
            .map(Monitor::new)
            .collect()
    }

    /// Create new event loop proxy
    pub fn create_proxy(&self) -> EventLoopProxy {
        EventLoopProxy::new(self.0.create_proxy())
    }

    /// Run the event loop
    pub fn run(self, event_handler: impl FnMut(Event) + 'static) -> ! {
        self.0.run(event_handler)
    }
}

// MARK: EventLoopProxy
/// Event loop proxy
pub struct EventLoopProxy(PlatformEventLoopProxy);

pub(crate) trait EventLoopProxyInterface {
    fn send_user_event(&self, data: String);
}

impl EventLoopProxy {
    fn new(proxy: PlatformEventLoopProxy) -> Self {
        Self(proxy)
    }

    /// Send user event to the event loop
    pub fn send_user_event(&self, data: String) {
        self.0.send_user_event(data);
    }
}

// MARK: Monitor
/// Monitor
pub struct Monitor(PlatformMonitor);

pub(crate) trait MonitorInterface {
    fn name(&self) -> String;
    fn position(&self) -> LogicalPoint;
    fn size(&self) -> LogicalSize;
    fn scale_factor(&self) -> f32;
}

impl Monitor {
    fn new(monitor: PlatformMonitor) -> Self {
        Self(monitor)
    }

    /// Get monitor name
    pub fn name(&self) -> String {
        self.0.name()
    }

    /// Get monitor position
    pub fn position(&self) -> LogicalPoint {
        self.0.position()
    }

    /// Get monitor size
    pub fn size(&self) -> LogicalSize {
        self.0.size()
    }

    /// Get monitor scale factor
    pub fn scale_factor(&self) -> f32 {
        self.0.scale_factor()
    }
}

// MARK: WebviewBuilder
/// Webview builder
pub struct WebviewBuilder {
    title: String,
    position: Option<LogicalPoint>,
    size: LogicalSize,
    min_size: Option<LogicalSize>,
    resizable: bool,
    fullscreen: bool,
    #[cfg(feature = "remember_window_state")]
    remember_window_state: bool,
    should_force_dark_mode: bool,
    should_center: bool,
    should_load_url: Option<String>,
    should_load_html: Option<String>,
    #[cfg(feature = "rust-embed")]
    embed_assets_get: Option<fn(&str) -> Option<rust_embed::EmbeddedFile>>,
    #[cfg(feature = "rust-embed")]
    internal_http_server_expose: bool,
    #[cfg(feature = "rust-embed")]
    internal_http_server_handle: Option<fn(&small_http::Request) -> Option<small_http::Response>>,
    #[cfg(target_os = "macos")]
    macos_titlebar_hidden: bool,
}

impl Default for WebviewBuilder {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            position: None,
            size: LogicalSize {
                width: 1024.0,
                height: 768.0,
            },
            min_size: None,
            resizable: true,
            fullscreen: false,
            #[cfg(feature = "remember_window_state")]
            remember_window_state: false,
            should_force_dark_mode: false,
            should_center: false,
            should_load_url: None,
            should_load_html: None,
            #[cfg(feature = "rust-embed")]
            embed_assets_get: None,
            #[cfg(feature = "rust-embed")]
            internal_http_server_expose: false,
            #[cfg(feature = "rust-embed")]
            internal_http_server_handle: None,
            #[cfg(target_os = "macos")]
            macos_titlebar_hidden: false,
        }
    }
}

impl WebviewBuilder {
    /// Create new webview builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set position
    pub fn position(mut self, position: LogicalPoint) -> Self {
        self.position = Some(position);
        self
    }

    /// Set size
    pub fn size(mut self, size: LogicalSize) -> Self {
        self.size = size;
        self
    }

    /// Set minimum size
    pub fn min_size(mut self, min_size: LogicalSize) -> Self {
        self.min_size = Some(min_size);
        self
    }

    /// Set resizable
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set fullscreen
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Set remember window state
    #[cfg(feature = "remember_window_state")]
    pub fn remember_window_state(mut self, remember_window_state: bool) -> Self {
        self.remember_window_state = remember_window_state;
        self
    }

    /// Force dark mode
    pub fn force_dark_mode(mut self, force_dark_mode: bool) -> Self {
        self.should_force_dark_mode = force_dark_mode;
        self
    }

    /// Center window
    pub fn center(mut self) -> Self {
        self.should_center = true;
        self
    }

    /// Load URL
    pub fn load_url(mut self, url: impl Into<String>) -> Self {
        self.should_load_url = Some(url.into());
        self
    }

    /// Load HTML string
    pub fn load_html(mut self, html: impl Into<String>) -> Self {
        self.should_load_html = Some(html.into());
        self
    }

    /// Load rust-embed folder
    #[cfg(feature = "rust-embed")]
    pub fn load_rust_embed<A: rust_embed::RustEmbed>(mut self) -> Self {
        self.embed_assets_get = Some(A::get);
        self
    }

    /// Expose internal http server to other devices in the network
    #[cfg(feature = "rust-embed")]
    pub fn internal_http_serve_expose(mut self, expose: bool) -> Self {
        self.internal_http_server_expose = expose;
        self
    }

    /// Set internal http server handler
    #[cfg(feature = "rust-embed")]
    pub fn internal_http_server_handle(
        mut self,
        handle: fn(&small_http::Request) -> Option<small_http::Response>,
    ) -> Self {
        self.internal_http_server_handle = Some(handle);
        self
    }

    /// Set macOS titlebar hidden
    #[cfg(target_os = "macos")]
    pub fn macos_titlebar_hidden(mut self, hidden: bool) -> Self {
        self.macos_titlebar_hidden = hidden;
        self
    }

    /// Build webview
    #[allow(unused_mut)]
    pub fn build(mut self) -> Webview {
        // Spawn a local http server when assets_get is set
        #[cfg(feature = "rust-embed")]
        if let Some(assets_get) = self.embed_assets_get.take() {
            let listener = if self.internal_http_server_expose {
                // Start an exposed HTTP server
                let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, 0))
                    .unwrap_or_else(|_| panic!("Can't start local http server"));
                self.should_load_url = Some(format!(
                    "http://{}:{}{}",
                    local_ip_address::local_ip().expect("Can't get local ip addr"),
                    listener
                        .local_addr()
                        .expect("Can't get local http server port")
                        .port(),
                    self.should_load_url.as_deref().unwrap_or("/")
                ));
                listener
            } else {
                // Start a local HTTP server
                let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
                    .unwrap_or_else(|_| panic!("Can't start local http server"));
                self.should_load_url = Some(format!(
                    "http://{}{}",
                    listener
                        .local_addr()
                        .expect("Can't get local http server addr"),
                    self.should_load_url.as_deref().unwrap_or("/")
                ));
                listener
            };

            std::thread::spawn(move || {
                small_http::serve_single_threaded(listener, move |req| {
                    let mut path = req.url.path().to_string();
                    if path.ends_with('/') {
                        path = format!("{}index.html", path);
                    }

                    if let Some(handle) = self.internal_http_server_handle {
                        if let Some(response) = handle(req) {
                            return response;
                        }
                    }

                    if let Some(file) = assets_get(path.trim_start_matches('/')) {
                        let mime = mime_guess::from_path(&path).first_or_octet_stream();
                        small_http::Response::with_header("Content-Type", mime.to_string())
                            .body(file.data)
                    } else {
                        small_http::Response::with_status(small_http::Status::NotFound)
                            .body(b"404 Not Found".to_vec())
                    }
                });
            });
        }

        Webview::new(PlatformWebview::new(self))
    }
}

// MARK: Webview
pub(crate) trait WebviewInterface {
    fn set_title(&mut self, title: impl AsRef<str>);
    fn position(&self) -> LogicalPoint;
    fn size(&self) -> LogicalSize;
    fn set_position(&mut self, point: LogicalPoint);
    fn set_size(&mut self, size: LogicalSize);
    fn set_min_size(&mut self, min_size: LogicalSize);
    fn set_resizable(&mut self, resizable: bool);

    fn load_url(&mut self, url: impl AsRef<str>);
    fn load_html(&mut self, html: impl AsRef<str>);
    fn evaluate_script(&mut self, script: impl AsRef<str>);
}

/// Webview
pub struct Webview(PlatformWebview);

impl Webview {
    fn new(webview: PlatformWebview) -> Self {
        Self(webview)
    }

    /// Set title
    pub fn set_title(&mut self, title: impl AsRef<str>) {
        self.0.set_title(title)
    }

    /// Get position
    pub fn position(&self) -> LogicalPoint {
        self.0.position()
    }

    /// Get size
    pub fn size(&self) -> LogicalSize {
        self.0.size()
    }

    /// Set position
    pub fn set_position(&mut self, point: LogicalPoint) {
        self.0.set_position(point)
    }

    /// Set size
    pub fn set_size(&mut self, size: LogicalSize) {
        self.0.set_size(size)
    }

    /// Set minimum size
    pub fn set_min_size(&mut self, min_size: LogicalSize) {
        self.0.set_min_size(min_size)
    }

    /// Set resizable
    pub fn set_resizable(&mut self, resizable: bool) {
        self.0.set_resizable(resizable)
    }

    /// Load URL
    pub fn load_url(&mut self, url: impl AsRef<str>) {
        self.0.load_url(url)
    }

    /// Load HTML string
    pub fn load_html(&mut self, html: impl AsRef<str>) {
        self.0.load_html(html)
    }

    /// Evaluate script
    pub fn evaluate_script(&mut self, script: impl AsRef<str>) {
        self.0.evaluate_script(script)
    }

    /// Send IPC message
    pub fn send_ipc_message(&mut self, message: impl AsRef<str>) {
        self.evaluate_script(format!(
            "window.ipc.dispatchEvent(new MessageEvent('message',{{data:`{}`}}));",
            message.as_ref()
        ));
    }
}
