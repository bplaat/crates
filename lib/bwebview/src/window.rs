/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::atomic::{AtomicU32, Ordering};

use crate::platforms::{PlatformMonitor, PlatformWindow};
use crate::{Key, LogicalPoint, LogicalSize, Modifiers, MouseButton};

// MARK: WindowId
/// Window identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub(crate) u32);

impl WindowId {
    /// Get raw ID value
    pub fn id(&self) -> u32 {
        self.0
    }
}

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

// MARK: WindowHandler
/// Window event handler trait
pub trait WindowHandler {
    /// Called when the user requests to close the window. Return `true` to allow close, `false` to prevent it.
    fn on_close(&mut self, window: &mut Window) -> bool {
        true
    }
    /// Called when the window moves
    fn on_move(&mut self, window: &mut Window, x: i32, y: i32) {
        let _ = (window, x, y);
    }
    /// Called when the window is resized
    fn on_resize(&mut self, window: &mut Window, width: u32, height: u32) {
        let _ = (window, width, height);
    }
    /// Called when the window gains keyboard focus
    fn on_focus(&mut self, window: &mut Window) {
        let _ = window;
    }
    /// Called when the window loses keyboard focus
    fn on_blur(&mut self, window: &mut Window) {
        let _ = window;
    }
    /// Called when a key is pressed
    fn on_key_down(&mut self, window: &mut Window, key: Key, mods: Modifiers) {
        let _ = (window, key, mods);
    }
    /// Called when a key is released
    fn on_key_up(&mut self, window: &mut Window, key: Key, mods: Modifiers) {
        let _ = (window, key, mods);
    }
    /// Called when the mouse moves over the window
    fn on_mouse_move(&mut self, window: &mut Window, x: f64, y: f64) {
        let _ = (window, x, y);
    }
    /// Called when a mouse button is pressed
    fn on_mouse_down(&mut self, window: &mut Window, button: MouseButton, x: f64, y: f64) {
        let _ = (window, button, x, y);
    }
    /// Called when a mouse button is released
    fn on_mouse_up(&mut self, window: &mut Window, button: MouseButton, x: f64, y: f64) {
        let _ = (window, button, x, y);
    }
    /// Called when the scroll wheel is used
    fn on_wheel(&mut self, window: &mut Window, delta_x: f64, delta_y: f64) {
        let _ = (window, delta_x, delta_y);
    }
    /// Called when the macOS window enters or exits fullscreen
    #[cfg(target_os = "macos")]
    fn on_fullscreen_change(&mut self, window: &mut Window, is_fullscreen: bool) {
        let _ = (window, is_fullscreen);
    }
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
    pub(crate) remember_window_state: bool,
    pub(crate) monitor: Option<&'a PlatformMonitor>,
    pub(crate) should_center: bool,
    pub(crate) should_fullscreen: bool,
    #[cfg(target_os = "macos")]
    pub(crate) macos_titlebar_style: MacosTitlebarStyle,
    pub(crate) window_handler: Option<*mut dyn WindowHandler>,
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
            remember_window_state: false,
            monitor: None,
            should_center: false,
            should_fullscreen: false,
            #[cfg(target_os = "macos")]
            macos_titlebar_style: MacosTitlebarStyle::Default,
            window_handler: None,
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

    /// Enable remember window state
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

    /// Set window event handler
    pub fn handler<H: WindowHandler + 'static>(mut self, handler: &mut H) -> Self {
        self.window_handler = Some(handler as *mut dyn WindowHandler);
        self
    }

    /// Build window
    pub fn build(self) -> Window {
        let id = WindowId(NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed));
        let window_handler = self.window_handler;
        let platform = PlatformWindow::new(id, &self);
        Window {
            id,
            platform,
            window_handler,
        }
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
    pub(crate) window_handler: Option<*mut dyn WindowHandler>,
}

impl Window {
    /// Construct a temporary non-owning Window from raw parts for use in callbacks.
    ///
    /// # Safety
    /// The caller must ensure `platform_data` remains valid for the duration of the returned value,
    /// and must call `std::mem::forget` on the returned Window to prevent double-free.
    pub(crate) unsafe fn from_raw(
        id: WindowId,
        platform: PlatformWindow,
        window_handler: Option<*mut dyn WindowHandler>,
    ) -> std::mem::ManuallyDrop<Window> {
        std::mem::ManuallyDrop::new(Window {
            id,
            platform,
            window_handler,
        })
    }

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
