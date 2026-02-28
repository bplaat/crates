/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![allow(unused)]

pub use event::*;
pub use sizes::*;

#[cfg(feature = "file_dialog")]
use crate::platforms::PlatformFileDialog;
use crate::platforms::{
    PlatformEventLoop, PlatformEventLoopProxy, PlatformMonitor, PlatformWebview,
};

mod event;
mod platforms;
mod sizes;

// MARK: EventLoopBuilder
pub(crate) struct AppId {
    pub qualifier: String,
    pub organization: String,
    pub application: String,
}

/// EventLoop builder
#[derive(Default)]
pub struct EventLoopBuilder {
    app_id: Option<AppId>,
}

impl EventLoopBuilder {
    /// Create new webview builder
    pub fn new() -> Self {
        Self::default()
    }

    /// App id used for storing window state and other platform specific features
    pub fn app_id(
        mut self,
        qualifier: impl AsRef<str>,
        organization: impl AsRef<str>,
        application: impl AsRef<str>,
    ) -> Self {
        self.app_id = Some(AppId {
            qualifier: qualifier.as_ref().to_string(),
            organization: organization.as_ref().to_string(),
            application: application.as_ref().to_string(),
        });
        self
    }

    /// Create new event loop
    pub fn build(self) -> EventLoop {
        EventLoop::from_platform(PlatformEventLoop::new(self))
    }
}

// MARK: EventLoop
pub(crate) trait EventLoopInterface {
    fn primary_monitor(&self) -> PlatformMonitor;
    fn available_monitors(&self) -> Vec<PlatformMonitor>;
    fn create_proxy(&self) -> PlatformEventLoopProxy;
    fn run(self, event_handler: impl FnMut(Event) + 'static) -> !;
}

/// Event loop
pub struct EventLoop(PlatformEventLoop);

impl EventLoop {
    fn from_platform(event_loop: PlatformEventLoop) -> Self {
        Self(event_loop)
    }

    /// Create new event loop
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        EventLoopBuilder::new().build()
    }

    /// Get primary monitor
    pub fn primary_monitor(&self) -> Monitor {
        Monitor::new(self.0.primary_monitor())
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
    fn is_primary(&self) -> bool;
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
    ///
    /// Primary monitor is 0x0 position all other monitors are relative to the primary monitor.
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

    /// Get if monitor is primary
    pub fn is_primary(&self) -> bool {
        self.0.is_primary()
    }
}

// MARK: WebviewBuilder
/// Theme
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Theme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
}

/// macOS Titlebar style
#[cfg(target_os = "macos")]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MacosTitlebarStyle {
    /// Default titlebar style
    Default,
    /// Transparent titlebar
    Transparent,
    /// Hidden titlebar
    Hidden,
}

#[cfg(feature = "custom_protocol")]
struct CustomProtocol {
    scheme: String,
    handler: Box<dyn Fn(&small_http::Request) -> small_http::Response + Send + 'static>,
}

#[cfg(feature = "rust-embed")]
type EmbedCustomHandler =
    dyn Fn(&small_http::Request) -> Option<small_http::Response> + Send + 'static;

/// Webview builder
pub struct WebviewBuilder<'a> {
    title: String,
    position: Option<LogicalPoint>,
    size: LogicalSize,
    min_size: Option<LogicalSize>,
    resizable: bool,
    theme: Option<Theme>,
    background_color: Option<u32>,
    #[cfg(feature = "remember_window_state")]
    remember_window_state: bool,
    monitor: Option<&'a PlatformMonitor>,
    should_center: bool,
    should_fullscreen: bool,
    should_load_url: Option<String>,
    should_load_html: Option<String>,
    #[cfg(feature = "custom_protocol")]
    custom_protocols: Vec<CustomProtocol>,
    #[cfg(feature = "rust-embed")]
    embed_assets_get: Option<fn(&str) -> Option<rust_embed::EmbeddedFile>>,
    #[cfg(feature = "rust-embed")]
    embed_custom_handler: Option<Box<EmbedCustomHandler>>,
    #[cfg(target_os = "macos")]
    macos_titlebar_style: MacosTitlebarStyle,
}

impl<'a> Default for WebviewBuilder<'a> {
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
            theme: None,
            background_color: None,
            #[cfg(feature = "remember_window_state")]
            remember_window_state: false,
            monitor: None,
            should_center: false,
            should_fullscreen: false,
            should_load_url: None,
            should_load_html: None,
            #[cfg(feature = "custom_protocol")]
            custom_protocols: Vec::new(),
            #[cfg(feature = "rust-embed")]
            embed_assets_get: None,
            #[cfg(feature = "rust-embed")]
            embed_custom_handler: None,
            #[cfg(target_os = "macos")]
            macos_titlebar_style: MacosTitlebarStyle::Default,
        }
    }
}

impl<'a> WebviewBuilder<'a> {
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

    /// Set theme
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Set window background color
    pub fn background_color(mut self, color: u32) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set remember window state
    #[cfg(feature = "remember_window_state")]
    pub fn remember_window_state(mut self) -> Self {
        self.remember_window_state = true;
        self
    }

    /// Set monitor
    pub fn monitor(mut self, monitor: &'a Monitor) -> Self {
        self.monitor = Some(&monitor.0);
        self
    }

    /// Center window
    pub fn center(mut self) -> Self {
        self.should_center = true;
        self
    }

    /// Set fullscreen
    pub fn fullscreen(mut self) -> Self {
        self.should_fullscreen = true;
        self
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

    /// Load rust-embed folder with options
    #[cfg(feature = "rust-embed")]
    pub fn load_rust_embed_with_custom_handler<A: rust_embed::RustEmbed>(
        mut self,
        handler: impl Fn(&small_http::Request) -> Option<small_http::Response> + Send + 'static,
    ) -> Self {
        self.embed_assets_get = Some(A::get);
        self.embed_custom_handler = Some(Box::new(handler));
        self
    }

    /// Set macOS title transparent
    #[cfg(target_os = "macos")]
    pub fn macos_titlebar_style(mut self, style: MacosTitlebarStyle) -> Self {
        self.macos_titlebar_style = style;
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
        Webview::new(PlatformWebview::new(self))
    }
}

// MARK: Webview
/// Injection time for user scripts
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum InjectionTime {
    /// Inject at document start
    DocumentStart,
    /// Inject at document loaded
    DocumentLoaded,
}

pub(crate) trait WebviewInterface {
    fn set_title(&mut self, title: impl AsRef<str>);
    fn position(&self) -> LogicalPoint;
    fn size(&self) -> LogicalSize;
    fn set_position(&mut self, point: LogicalPoint);
    fn set_size(&mut self, size: LogicalSize);
    fn set_min_size(&mut self, min_size: LogicalSize);
    fn set_resizable(&mut self, resizable: bool);
    fn set_theme(&mut self, theme: Theme);
    fn set_background_color(&mut self, color: u32);
    fn url(&self) -> Option<String>;
    fn load_url(&mut self, url: impl AsRef<str>);
    fn load_html(&mut self, html: impl AsRef<str>);
    fn evaluate_script(&mut self, script: impl AsRef<str>);
    fn add_user_script(&mut self, script: impl AsRef<str>, injection_time: InjectionTime);
    #[cfg(target_os = "macos")]
    fn macos_titlebar_size(&self) -> LogicalSize;
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

    /// Set theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.0.set_theme(theme)
    }

    /// Set window background color
    pub fn set_background_color(&mut self, color: u32) {
        self.0.set_background_color(color)
    }

    /// Get URL
    pub fn url(&self) -> Option<String> {
        self.0.url()
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

    /// Add user script, a script that runs on every page load at injection time.
    pub fn add_user_script(&mut self, script: impl AsRef<str>, injection_time: InjectionTime) {
        self.0.add_user_script(script, injection_time)
    }

    /// Send IPC message
    pub fn send_ipc_message(&mut self, message: impl AsRef<str>) {
        let message = message.as_ref().replace('\\', "\\\\").replace('`', "\\`");
        self.evaluate_script(format!(
            "window.ipc.dispatchEvent(new MessageEvent('message',{{data:`{message}`}}));",
        ));
    }

    /// Get macOS titlebar size
    #[cfg(target_os = "macos")]
    pub fn macos_titlebar_size(&self) -> LogicalSize {
        self.0.macos_titlebar_size()
    }
}

// MARK: FileDialog
#[cfg(feature = "file_dialog")]
pub(crate) struct FileDialogFilter {
    pub(crate) name: String,
    pub(crate) extensions: Vec<String>,
}

#[cfg(feature = "file_dialog")]
pub(crate) trait FileDialogInterface {
    fn pick_file(dialog: FileDialog) -> Option<std::path::PathBuf>;
    fn pick_files(dialog: FileDialog) -> Option<Vec<std::path::PathBuf>>;
    fn save_file(dialog: FileDialog) -> Option<std::path::PathBuf>;
}

/// File dialog builder
#[cfg(feature = "file_dialog")]
#[derive(Default)]
pub struct FileDialog {
    pub(crate) title: Option<String>,
    pub(crate) directory: Option<std::path::PathBuf>,
    pub(crate) filename: Option<String>,
    pub(crate) filters: Vec<FileDialogFilter>,
}

#[cfg(feature = "file_dialog")]
impl FileDialog {
    /// Create new file dialog
    pub fn new() -> Self {
        Self::default()
    }

    /// Set dialog title
    pub fn title(mut self, title: impl AsRef<str>) -> Self {
        self.title = Some(title.as_ref().to_string());
        self
    }

    /// Set starting directory
    pub fn set_directory(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.directory = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set default file name (used for save dialogs)
    pub fn set_file_name(mut self, filename: impl AsRef<str>) -> Self {
        self.filename = Some(filename.as_ref().to_string());
        self
    }

    /// Add a file filter (name + list of extensions without dot)
    pub fn add_filter(mut self, name: impl AsRef<str>, extensions: &[impl AsRef<str>]) -> Self {
        self.filters.push(FileDialogFilter {
            name: name.as_ref().to_string(),
            extensions: extensions.iter().map(|e| e.as_ref().to_string()).collect(),
        });
        self
    }

    /// Open a single-file picker dialog
    pub fn pick_file(self) -> Option<std::path::PathBuf> {
        PlatformFileDialog::pick_file(self)
    }

    /// Open a multi-file picker dialog
    pub fn pick_files(self) -> Option<Vec<std::path::PathBuf>> {
        PlatformFileDialog::pick_files(self)
    }

    /// Open a save-file dialog
    pub fn save_file(self) -> Option<std::path::PathBuf> {
        PlatformFileDialog::save_file(self)
    }
}
