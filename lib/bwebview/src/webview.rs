/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::Window;
use crate::platforms::PlatformWebview;

// MARK: CustomProtocol
#[cfg(feature = "custom_protocol")]
pub(crate) struct CustomProtocol {
    pub scheme: String,
    pub handler: Box<dyn Fn(&small_http::Request) -> small_http::Response + Send + 'static>,
}

// MARK: InjectionTime
/// Injection time for user scripts
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum InjectionTime {
    /// Inject at document start
    DocumentStart,
    /// Inject at document loaded
    DocumentLoaded,
}

// MARK: WebviewBuilder
#[cfg(feature = "rust-embed")]
type EmbedCustomHandler =
    dyn Fn(&small_http::Request) -> Option<small_http::Response> + Send + 'static;

/// Webview builder
pub struct WebviewBuilder<'a> {
    pub(crate) window: &'a Window,
    pub(crate) should_load_url: Option<String>,
    pub(crate) should_load_html: Option<String>,
    #[cfg(feature = "custom_protocol")]
    pub(crate) custom_protocols: Vec<CustomProtocol>,
    #[cfg(feature = "rust-embed")]
    embed_assets_get: Option<fn(&str) -> Option<rust_embed::EmbeddedFile>>,
    #[cfg(feature = "rust-embed")]
    embed_custom_handler: Option<Box<EmbedCustomHandler>>,
}

impl<'a> WebviewBuilder<'a> {
    /// Create new webview builder
    pub fn new(window: &'a Window) -> Self {
        Self {
            window,
            should_load_url: None,
            should_load_html: None,
            #[cfg(feature = "custom_protocol")]
            custom_protocols: Vec::new(),
            #[cfg(feature = "rust-embed")]
            embed_assets_get: None,
            #[cfg(feature = "rust-embed")]
            embed_custom_handler: None,
        }
    }

    /// Add custom protocol
    #[cfg(feature = "custom_protocol")]
    pub fn with_custom_protocol(
        mut self,
        scheme: impl Into<String>,
        handler: impl Fn(&small_http::Request) -> small_http::Response + Send + 'static,
    ) -> Self {
        self.custom_protocols.push(CustomProtocol {
            scheme: scheme.into(),
            handler: Box::new(handler),
        });
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

    /// Load rust-embed folder with custom request handler
    #[cfg(feature = "rust-embed")]
    pub fn load_rust_embed_with_custom_handler<A: rust_embed::RustEmbed>(
        mut self,
        handler: impl Fn(&small_http::Request) -> Option<small_http::Response> + Send + 'static,
    ) -> Self {
        self.embed_assets_get = Some(A::get);
        self.embed_custom_handler = Some(Box::new(handler));
        self
    }

    /// Build webview
    #[allow(unused_mut)]
    pub fn build(mut self) -> Webview {
        #[cfg(feature = "rust-embed")]
        if let Some(assets_get) = self.embed_assets_get.take() {
            let handler = self.embed_custom_handler.take();
            self = self.with_custom_protocol("app", move |req| {
                let mut path = req.url.path().to_string();
                if path.ends_with('/') {
                    path = format!("{path}index.html");
                }

                if let Some(handle) = &handler
                    && let Some(response) = handle(req)
                {
                    return response;
                }

                if let Some(file) = assets_get(path.trim_start_matches('/')) {
                    let mime = mime_guess::from_path(&path).first_or_octet_stream();
                    small_http::Response::with_header("Content-Type", mime.to_string())
                        .body(file.data)
                } else if let Some(file) = assets_get("index.html") {
                    small_http::Response::with_header("Content-Type", "text/html").body(file.data)
                } else {
                    small_http::Response::with_status(small_http::Status::NotFound)
                        .body(b"404 Not Found".to_vec())
                }
            });
            self = self.load_url("app://index.html");
        }

        let mut platform = PlatformWebview::new(&self.window.platform);
        platform.init_webview(self);
        Webview { platform }
    }
}

// MARK: WebviewInterface
pub(crate) trait WebviewInterface {
    fn url(&self) -> Option<String>;
    fn load_url(&mut self, url: impl AsRef<str>);
    fn load_html(&mut self, html: impl AsRef<str>);
    fn evaluate_script(&mut self, script: impl AsRef<str>);
    fn add_user_script(&mut self, script: impl AsRef<str>, injection_time: InjectionTime);
    fn set_background_color(&mut self, color: u32);
}

// MARK: Webview
/// Webview
pub struct Webview {
    pub(crate) platform: PlatformWebview,
}

impl Webview {
    /// Get URL
    pub fn url(&self) -> Option<String> {
        self.platform.url()
    }

    /// Load URL
    pub fn load_url(&mut self, url: impl AsRef<str>) {
        self.platform.load_url(url)
    }

    /// Load HTML string
    pub fn load_html(&mut self, html: impl AsRef<str>) {
        self.platform.load_html(html)
    }

    /// Evaluate script
    pub fn evaluate_script(&mut self, script: impl AsRef<str>) {
        self.platform.evaluate_script(script)
    }

    /// Add user script, a script that runs on every page load at injection time
    pub fn add_user_script(&mut self, script: impl AsRef<str>, injection_time: InjectionTime) {
        self.platform.add_user_script(script, injection_time)
    }

    /// Set webview background color
    pub fn set_background_color(&mut self, color: u32) {
        self.platform.set_background_color(color)
    }

    /// Send IPC message
    pub fn send_ipc_message(&mut self, message: impl AsRef<str>) {
        let message = message.as_ref().replace('\\', "\\\\").replace('`', "\\`");
        self.evaluate_script(format!(
            "window.ipc.dispatchEvent(new MessageEvent('message',{{data:`{message}`}}));",
        ));
    }
}
