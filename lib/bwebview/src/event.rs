/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::{LogicalPoint, LogicalSize, Theme};

/// DOM-style mouse event data.
#[derive(Clone)]
pub struct MouseEvent {
    /// X coordinate relative to the window content.
    pub client_x: f32,
    /// Y coordinate relative to the window content.
    pub client_y: f32,
    /// X coordinate relative to the virtual desktop.
    pub screen_x: f32,
    /// Y coordinate relative to the virtual desktop.
    pub screen_y: f32,
    /// Horizontal movement since the previous mouse event.
    pub movement_x: f32,
    /// Vertical movement since the previous mouse event.
    pub movement_y: f32,
    /// Changed DOM mouse button (`0` left, `1` middle, `2` right).
    pub button: i16,
    /// DOM mouse button bit mask.
    pub buttons: u16,
    /// Click count.
    pub detail: u16,
    /// Whether Alt is pressed.
    pub alt_key: bool,
    /// Whether Control is pressed.
    pub ctrl_key: bool,
    /// Whether Meta/Command is pressed.
    pub meta_key: bool,
    /// Whether Shift is pressed.
    pub shift_key: bool,
}

/// DOM-style wheel event data.
#[derive(Clone)]
pub struct WheelEvent {
    /// Mouse data at the wheel event location.
    pub mouse: MouseEvent,
    /// Horizontal scroll delta.
    pub delta_x: f32,
    /// Vertical scroll delta.
    pub delta_y: f32,
    /// Depth scroll delta.
    pub delta_z: f32,
    /// DOM delta mode (`0` pixels, `1` lines, `2` pages).
    pub delta_mode: u32,
}

/// DOM-style keyboard event data.
#[derive(Clone)]
pub struct KeyboardEvent {
    /// Logical DOM key value, such as `a`, `Enter`, or `ArrowLeft`.
    pub key: String,
    /// Physical DOM key code, such as `KeyA` or `Digit1`.
    pub code: String,
    /// DOM key location.
    pub location: u32,
    /// Whether this is an automatic repeat event.
    pub repeat: bool,
    /// Whether an input method is composing text.
    pub is_composing: bool,
    /// Whether Alt is pressed.
    pub alt_key: bool,
    /// Whether Control is pressed.
    pub ctrl_key: bool,
    /// Whether Meta/Command is pressed.
    pub meta_key: bool,
    /// Whether Shift is pressed.
    pub shift_key: bool,
}

/// A close request that can be prevented by the event handler
#[derive(Clone)]
pub struct CloseRequest(Rc<Cell<bool>>);

impl CloseRequest {
    pub(crate) fn new() -> Self {
        Self(Rc::new(Cell::new(false)))
    }

    /// Prevent the window from closing
    pub fn prevent_close(&self) {
        self.0.set(true);
    }

    pub(crate) fn is_prevented(&self) -> bool {
        self.0.get()
    }
}

/// Window event
pub enum WindowEvent {
    /// Window create
    Create,
    /// Window moved. Requires [`crate::WindowEvents::MOVE`].
    Move(LogicalPoint),
    /// Window resized. Requires [`crate::WindowEvents::RESIZE`].
    Resize(LogicalSize),
    /// Window gained focus. Requires [`crate::WindowEvents::FOCUS`].
    Focus,
    /// Window lost focus. Requires [`crate::WindowEvents::FOCUS`].
    Blur,
    /// The effective light or dark system appearance changed. Requires
    /// [`crate::WindowEvents::THEME_CHANGE`].
    ThemeChange(Theme),
    /// Mouse button pressed. Requires [`crate::WindowEvents::MOUSE`].
    MouseDown(MouseEvent),
    /// Mouse button released. Requires [`crate::WindowEvents::MOUSE`].
    MouseUp(MouseEvent),
    /// Mouse moved. Requires [`crate::WindowEvents::MOUSE`].
    MouseMove(MouseEvent),
    /// Mouse entered the window. Requires [`crate::WindowEvents::MOUSE`].
    MouseEnter(MouseEvent),
    /// Mouse left the window. Requires [`crate::WindowEvents::MOUSE`].
    MouseLeave(MouseEvent),
    /// Primary-button click completed. Requires [`crate::WindowEvents::MOUSE`].
    Click(MouseEvent),
    /// Mouse wheel moved. Requires [`crate::WindowEvents::WHEEL`].
    Wheel(WheelEvent),
    /// Keyboard key pressed. Requires [`crate::WindowEvents::KEYBOARD`].
    KeyDown(KeyboardEvent),
    /// Keyboard key released. Requires [`crate::WindowEvents::KEYBOARD`].
    KeyUp(KeyboardEvent),
    /// Window close requested; closes normally unless prevented
    CloseRequested(CloseRequest),
    /// File dropped
    #[cfg(feature = "file_drop")]
    DroppedFile(PathBuf),
    /// macOS window fullscreen change
    #[cfg(target_os = "macos")]
    MacosFullscreenChange(bool),
}

/// Webview event
pub enum WebviewEvent {
    /// Page load start
    PageLoadStart,
    /// Page load finish
    PageLoadFinish,
    /// Page title change
    PageTitleChange(String),
    /// IPC message receive
    MessageReceive(String),
}

/// Event
pub enum Event {
    /// Window event
    Window(WindowEvent),
    /// Webview event
    Webview(WebviewEvent),
    /// User event
    UserEvent(String),
    /// Files opened through macOS Launch Services
    #[cfg(target_os = "macos")]
    MacosOpenFiles(Vec<PathBuf>),
    /// A custom macOS menu item was selected
    #[cfg(all(target_os = "macos", feature = "menu"))]
    MacosMenuItem(String),
}
