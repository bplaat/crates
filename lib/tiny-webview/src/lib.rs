/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

pub use event::*;
pub use sizes::*;

mod event;
mod platforms;
mod sizes;

// MARK: EventLoop trait
/// EventLoop trait
pub trait EventLoop {
    /// Start event loop
    fn run(&mut self, event_handler: impl FnMut(Event) + 'static) -> !;
}

// MARK: EventLoopBuilder
/// EventLoop builder
pub struct EventLoopBuilder;

impl EventLoopBuilder {
    /// Create new event loop builder
    pub fn build() -> impl EventLoop {
        platforms::EventLoop::new()
    }
}

// MARK: Webview trait
/// Webview trait
pub trait Webview {
    /// Set title
    fn set_title(&mut self, title: impl AsRef<str>);
    /// Get position
    fn position(&self) -> LogicalPoint;
    /// Get size
    fn size(&self) -> LogicalSize;
    /// Set position
    fn set_position(&mut self, point: LogicalPoint);
    /// Set size
    fn set_size(&mut self, size: LogicalSize);
    /// Set minimum size
    fn set_min_size(&mut self, min_size: LogicalSize);
    /// Set resizable
    fn set_resizable(&mut self, resizable: bool);

    /// Load URL
    fn load_url(&mut self, url: impl AsRef<str>);
    /// Load HTML string
    fn load_html(&mut self, html: impl AsRef<str>);
    /// Evaluate script
    fn evaluate_script(&mut self, script: impl AsRef<str>);

    /// Send IPC message
    #[cfg(feature = "ipc")]
    fn send_ipc_message(&mut self, message: impl AsRef<str>) {
        self.evaluate_script(format!(
            "window.ipc.dispatchEvent(new MessageEvent('message',{{data:`{}`}}));",
            message.as_ref()
        ));
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
    #[cfg(feature = "remember_window_state")]
    remember_window_state: bool,
    should_force_dark_mode: bool,
    should_center: bool,
    should_load_url: Option<String>,
    should_load_html: Option<String>,
    #[cfg(feature = "rust-embed")]
    embed_assets_get: Option<fn(&str) -> Option<rust_embed::EmbeddedFile>>,
    #[cfg(feature = "rust-embed")]
    internal_http_serve_handle: Option<fn(&small_http::Request) -> Option<small_http::Response>>,
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
            #[cfg(feature = "remember_window_state")]
            remember_window_state: false,
            should_force_dark_mode: false,
            should_center: false,
            should_load_url: None,
            should_load_html: None,
            #[cfg(feature = "rust-embed")]
            embed_assets_get: None,
            #[cfg(feature = "rust-embed")]
            internal_http_serve_handle: None,
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

    /// Set internal http server handler
    #[cfg(feature = "rust-embed")]
    pub fn internal_http_serve_handle(
        mut self,
        handle: fn(&small_http::Request) -> Option<small_http::Response>,
    ) -> Self {
        self.internal_http_serve_handle = Some(handle);
        self
    }

    /// Build webview
    #[allow(unused_mut)]
    pub fn build(mut self) -> impl Webview {
        // Spawn a local http server when assets_get is set
        #[cfg(feature = "rust-embed")]
        if let Some(assets_get) = self.embed_assets_get.take() {
            let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
                .unwrap_or_else(|_| panic!("Can't start local http server"));
            let local_addr = listener
                .local_addr()
                .expect("Can't start local http server");
            println!("Local HTTP server started at http://{}/", local_addr);
            std::thread::spawn(move || {
                small_http::serve_single_threaded(listener, move |req| {
                    let mut path = req.url.path().to_string();
                    if path.ends_with('/') {
                        path = format!("{}index.html", path);
                    }

                    if let Some(handle) = self.internal_http_serve_handle {
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
            self.should_load_url = Some(format!("http://{}/", local_addr));
        }

        platforms::Webview::new(self)
    }
}
