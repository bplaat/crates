/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple webview library

#[cfg(target_os = "macos")]
pub use macos::Webview;
#[cfg(not(target_os = "macos"))]
pub use stub::Webview;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod stub;

// MARK: Event
/// Event
#[repr(C)]
pub enum Event {
    /// Page loaded
    PageLoaded,
    /// Ipc message received
    IpcMessageReceived(String),
}

// MARK: Size
/// Size
#[allow(dead_code)]
struct Size {
    /// Width
    pub width: i32,
    /// Height
    pub height: i32,
}

// MARK: WebviewBuilder
/// Webview builder
pub struct WebviewBuilder {
    title: String,
    size: Size,
    min_size: Option<Size>,
    remember_window_state: bool,
    enable_ipc: bool,
    url: Option<String>,
    html: Option<String>,
}

impl Default for WebviewBuilder {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            size: Size {
                width: 800,
                height: 600,
            },
            min_size: None,
            remember_window_state: false,
            enable_ipc: false,
            url: None,
            html: None,
        }
    }
}

impl WebviewBuilder {
    /// Create new webview builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set title
    pub fn title(mut self, title: impl AsRef<str>) -> Self {
        self.title = title.as_ref().to_string();
        self
    }

    /// Set size
    pub fn size(mut self, width: i32, height: i32) -> Self {
        self.size = Size { width, height };
        self
    }

    /// Set minimum size
    pub fn min_size(mut self, width: i32, height: i32) -> Self {
        self.min_size = Some(Size { width, height });
        self
    }

    /// Set remember window state
    pub fn remember_window_state(mut self, remember_window_state: bool) -> Self {
        self.remember_window_state = remember_window_state;
        self
    }

    /// Set enable ipc
    pub fn enable_ipc(mut self, enable_ipc: bool) -> Self {
        self.enable_ipc = enable_ipc;
        self
    }

    /// Set URL
    pub fn url(mut self, url: impl AsRef<str>) -> Self {
        self.url = Some(url.as_ref().to_string());
        self
    }

    /// Set HTML
    pub fn html(mut self, html: impl AsRef<str>) -> Self {
        self.html = Some(html.as_ref().to_string());
        self
    }

    /// Build webview
    pub fn build(self) -> Webview {
        Webview::new(self)
    }
}
