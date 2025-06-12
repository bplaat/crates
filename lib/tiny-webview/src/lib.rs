/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple webview library

pub use event::*;
pub use sizes::*;

mod event;
mod platforms;
mod sizes;

/// Webview
pub trait Webview {
    /// Start event loop
    fn run(&mut self, _event_handler: fn(&mut Self, Event)) -> !;

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
    fn send_ipc_message(&mut self, message: impl AsRef<str>);
}

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

    /// Build webview
    pub fn build(self) -> impl Webview {
        platforms::Webview::new(self)
    }
}
