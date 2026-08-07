/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::platforms::{PlatformMonitor, PlatformWindow};
use crate::{LogicalPoint, LogicalSize};

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

// MARK: WindowsProgressBarState
/// Windows taskbar progress state
#[cfg(all(windows, feature = "progress_bar"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowsProgressBarState {
    /// Show normal progress
    #[default]
    Normal,
    /// Show failed progress
    Error,
    /// Show paused progress
    Paused,
    /// Show progress without a known completion percentage
    Indeterminate,
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
    #[cfg(feature = "drag_drop")]
    pub(crate) allow_file_drop: bool,
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
            #[cfg(feature = "drag_drop")]
            allow_file_drop: false,
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
    pub const fn position(mut self, position: LogicalPoint) -> Self {
        self.position = Some(position);
        self
    }

    /// Set size
    pub const fn size(mut self, size: LogicalSize) -> Self {
        self.size = size;
        self
    }

    /// Set minimum size
    pub const fn min_size(mut self, min_size: LogicalSize) -> Self {
        self.min_size = Some(min_size);
        self
    }

    /// Set resizable
    pub const fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set theme
    pub const fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Set window background color
    pub const fn background_color(mut self, color: u32) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set remember window state
    #[cfg(feature = "remember_window_state")]
    pub const fn remember_window_state(mut self) -> Self {
        self.remember_window_state = true;
        self
    }

    /// Allow files to be dropped onto the window
    #[cfg(feature = "drag_drop")]
    pub const fn allow_file_drop(mut self, allow: bool) -> Self {
        self.allow_file_drop = allow;
        self
    }

    /// Set monitor
    pub const fn monitor(mut self, monitor: &'a crate::Monitor) -> Self {
        self.monitor = Some(&monitor.0);
        self
    }

    /// Center window on monitor
    pub const fn center(mut self) -> Self {
        self.should_center = true;
        self
    }

    /// Set fullscreen
    pub const fn fullscreen(mut self) -> Self {
        self.should_fullscreen = true;
        self
    }

    /// Set macOS titlebar style
    #[cfg(target_os = "macos")]
    pub const fn macos_titlebar_style(mut self, style: MacosTitlebarStyle) -> Self {
        self.macos_titlebar_style = style;
        self
    }

    /// Build window
    pub fn build(self) -> Window {
        let platform = PlatformWindow::new(&self);
        Window { platform }
    }
}

// MARK: WindowInterface
pub(crate) trait WindowInterface {
    fn close(&mut self);
    fn set_title(&mut self, title: impl AsRef<str>);
    fn position(&self) -> LogicalPoint;
    fn size(&self) -> LogicalSize;
    fn set_position(&mut self, point: LogicalPoint);
    fn set_size(&mut self, size: LogicalSize);
    fn set_min_size(&mut self, min_size: LogicalSize);
    fn set_resizable(&mut self, resizable: bool);
    fn set_theme(&mut self, theme: Theme);
    fn set_background_color(&mut self, color: u32);
    #[cfg(all(
        feature = "progress_bar",
        any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd",
        )
    ))]
    fn gtk_set_progress_bar(&mut self, progress: Option<f32>);
    #[cfg(target_os = "macos")]
    fn macos_titlebar_size(&self) -> LogicalSize;
    #[cfg(target_os = "macos")]
    fn macos_set_document_edited(&mut self, edited: bool);
    #[cfg(all(windows, feature = "progress_bar"))]
    fn windows_set_progress_bar(&mut self, progress: Option<f32>, state: WindowsProgressBarState);
}

// MARK: Window
/// Window
pub struct Window {
    pub(crate) platform: PlatformWindow,
}

impl Window {
    /// Close the window
    pub fn close(&mut self) {
        self.platform.close()
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

    /// Set GTK application launcher progress, use a value above `1.0` for indeterminate progress,
    /// or hide it with `None`
    #[cfg(all(
        feature = "progress_bar",
        any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd"
        )
    ))]
    pub fn gtk_set_progress_bar(&mut self, progress: Option<f32>) {
        self.platform.gtk_set_progress_bar(progress)
    }

    /// Get macOS titlebar size
    #[cfg(target_os = "macos")]
    pub fn macos_titlebar_size(&self) -> LogicalSize {
        self.platform.macos_titlebar_size()
    }

    /// Set whether the macOS window represents a document with unsaved changes
    #[cfg(target_os = "macos")]
    pub fn macos_set_document_edited(&mut self, edited: bool) {
        self.platform.macos_set_document_edited(edited)
    }

    /// Set Windows taskbar progress for this window, or hide it with `None`
    #[cfg(all(windows, feature = "progress_bar"))]
    pub fn windows_set_progress_bar(
        &mut self,
        progress: Option<f32>,
        state: WindowsProgressBarState,
    ) {
        self.platform.windows_set_progress_bar(progress, state)
    }
}
