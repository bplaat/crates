/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use raw_window_handle::RawWindowHandle;

use crate::dpi::{LogicalPoint, LogicalSize};
use crate::monitor::Monitor;
use crate::platforms::{PlatformMonitor, PlatformWindow};

/// Theme
#[derive(PartialEq, Eq)]
pub enum Theme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
}

/// MacOS Titlebar style
#[cfg(target_os = "macos")]
#[derive(PartialEq, Eq)]
pub enum MacosTitlebarStyle {
    /// Default titlebar style
    Default,
    /// Transparent titlebar
    Transparent,
    /// Hidden titlebar
    Hidden,
}

/// Window builder
pub struct WindowBuilder<'a> {
    pub(crate) title: String,
    pub(crate) position: Option<LogicalPoint>,
    pub(crate) size: LogicalSize,
    pub(crate) min_size: Option<LogicalSize>,
    pub(crate) resizable: bool,
    pub(crate) theme: Option<Theme>,
    pub(crate) remember_window_state: bool,
    pub(crate) monitor: Option<&'a PlatformMonitor>,
    pub(crate) should_center: bool,
    pub(crate) should_fullscreen: bool,
    #[cfg(target_os = "macos")]
    pub(crate) macos_titlebar_style: MacosTitlebarStyle,
}

impl<'a> Default for WindowBuilder<'a> {
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
            remember_window_state: false,
            monitor: None,
            should_center: false,
            should_fullscreen: false,
            #[cfg(target_os = "macos")]
            macos_titlebar_style: MacosTitlebarStyle::Default,
        }
    }
}

impl<'a> WindowBuilder<'a> {
    /// Create new window builder
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

    /// Set remember window state
    pub fn remember_window_state(mut self) -> Self {
        self.remember_window_state = true;
        self
    }

    /// Set monitor
    pub fn monitor(mut self, monitor: &'a Monitor) -> Self {
        self.monitor = Some(monitor.inner());
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

    /// Set macOS title transparent
    #[cfg(target_os = "macos")]
    pub fn macos_titlebar_style(mut self, style: MacosTitlebarStyle) -> Self {
        self.macos_titlebar_style = style;
        self
    }

    /// Build window
    #[allow(unused_mut)]
    pub fn build(mut self) -> Window {
        Window::new(PlatformWindow::new(self))
    }
}

// MARK: Window
/// Window
pub struct Window(PlatformWindow);

impl Window {
    fn new(window: PlatformWindow) -> Self {
        Self(window)
    }

    /// Get raw window handle
    pub fn raw_window_handle(&self) -> RawWindowHandle {
        self.0.raw_window_handle()
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
}
