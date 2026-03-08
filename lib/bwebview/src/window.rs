/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::atomic::{AtomicU32, Ordering};

use crate::platforms::{PlatformMonitor, PlatformWindow};
use crate::{LogicalPoint, LogicalSize, WindowId};

static NEXT_WINDOW_ID: AtomicU32 = AtomicU32::new(0);

// MARK: Theme
/// Theme
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Theme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
}

// MARK: MacosTitlebarStyle
/// macOS titlebar style
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

// MARK: WindowBuilder
/// Window builder
pub struct WindowBuilder<'a> {
    pub(crate) title: String,
    pub(crate) position: Option<LogicalPoint>,
    pub(crate) size: LogicalSize,
    pub(crate) min_size: Option<LogicalSize>,
    pub(crate) resizable: bool,
    pub(crate) theme: Option<Theme>,
    pub(crate) background_color: Option<u32>,
    #[cfg(feature = "remember_window_state")]
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
            background_color: None,
            #[cfg(feature = "remember_window_state")]
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
    pub fn monitor(mut self, monitor: &'a crate::Monitor) -> Self {
        self.monitor = Some(&monitor.0);
        self
    }

    /// Center window on monitor
    pub fn center(mut self) -> Self {
        self.should_center = true;
        self
    }

    /// Set fullscreen
    pub fn fullscreen(mut self) -> Self {
        self.should_fullscreen = true;
        self
    }

    /// Set macOS titlebar style
    #[cfg(target_os = "macos")]
    pub fn macos_titlebar_style(mut self, style: MacosTitlebarStyle) -> Self {
        self.macos_titlebar_style = style;
        self
    }

    /// Build window
    pub fn build(self) -> Window {
        let id = WindowId(NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed));
        let platform = PlatformWindow::new(id, &self);
        Window { id, platform }
    }
}

// MARK: WindowInterface
pub(crate) trait WindowInterface {
    fn set_title(&mut self, title: impl AsRef<str>);
    fn position(&self) -> LogicalPoint;
    fn size(&self) -> LogicalSize;
    fn set_position(&mut self, point: LogicalPoint);
    fn set_size(&mut self, size: LogicalSize);
    fn set_min_size(&mut self, min_size: LogicalSize);
    fn set_resizable(&mut self, resizable: bool);
    fn set_theme(&mut self, theme: Theme);
    fn set_background_color(&mut self, color: u32);
    #[cfg(target_os = "macos")]
    fn macos_titlebar_size(&self) -> LogicalSize;
}

// MARK: Window
/// Window
pub struct Window {
    pub(crate) id: WindowId,
    pub(crate) platform: PlatformWindow,
}

impl Window {
    /// Get window ID
    pub fn id(&self) -> WindowId {
        self.id
    }

    /// Set title
    pub fn set_title(&mut self, title: impl AsRef<str>) {
        self.platform.set_title(title)
    }

    /// Get position
    pub fn position(&self) -> LogicalPoint {
        self.platform.position()
    }

    /// Get size
    pub fn size(&self) -> LogicalSize {
        self.platform.size()
    }

    /// Set position
    pub fn set_position(&mut self, point: LogicalPoint) {
        self.platform.set_position(point)
    }

    /// Set size
    pub fn set_size(&mut self, size: LogicalSize) {
        self.platform.set_size(size)
    }

    /// Set minimum size
    pub fn set_min_size(&mut self, min_size: LogicalSize) {
        self.platform.set_min_size(min_size)
    }

    /// Set resizable
    pub fn set_resizable(&mut self, resizable: bool) {
        self.platform.set_resizable(resizable)
    }

    /// Set theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.platform.set_theme(theme)
    }

    /// Set window background color
    pub fn set_background_color(&mut self, color: u32) {
        self.platform.set_background_color(color)
    }

    /// Get macOS titlebar size
    #[cfg(target_os = "macos")]
    pub fn macos_titlebar_size(&self) -> LogicalSize {
        self.platform.macos_titlebar_size()
    }
}
