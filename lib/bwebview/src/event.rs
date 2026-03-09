/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::{LogicalPoint, LogicalSize};

/// Keyboard key code
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyCode {
    /// A key
    A,
    /// B key
    B,
    /// C key
    C,
    /// D key
    D,
    /// E key
    E,
    /// F key
    F,
    /// G key
    G,
    /// H key
    H,
    /// I key
    I,
    /// J key
    J,
    /// K key
    K,
    /// L key
    L,
    /// M key
    M,
    /// N key
    N,
    /// O key
    O,
    /// P key
    P,
    /// Q key
    Q,
    /// R key
    R,
    /// S key
    S,
    /// T key
    T,
    /// U key
    U,
    /// V key
    V,
    /// W key
    W,
    /// X key
    X,
    /// Y key
    Y,
    /// Z key
    Z,
    /// 0 digit key
    Key0,
    /// 1 digit key
    Key1,
    /// 2 digit key
    Key2,
    /// 3 digit key
    Key3,
    /// 4 digit key
    Key4,
    /// 5 digit key
    Key5,
    /// 6 digit key
    Key6,
    /// 7 digit key
    Key7,
    /// 8 digit key
    Key8,
    /// 9 digit key
    Key9,
    /// F1 function key
    F1,
    /// F2 function key
    F2,
    /// F3 function key
    F3,
    /// F4 function key
    F4,
    /// F5 function key
    F5,
    /// F6 function key
    F6,
    /// F7 function key
    F7,
    /// F8 function key
    F8,
    /// F9 function key
    F9,
    /// F10 function key
    F10,
    /// F11 function key
    F11,
    /// F12 function key
    F12,
    /// Backspace key
    Backspace,
    /// Tab key
    Tab,
    /// Enter / Return key
    Enter,
    /// Escape key
    Escape,
    /// Space key
    Space,
    /// Delete / Forward Delete key
    Delete,
    /// Insert key
    Insert,
    /// Left arrow key
    Left,
    /// Right arrow key
    Right,
    /// Up arrow key
    Up,
    /// Down arrow key
    Down,
    /// Home key
    Home,
    /// End key
    End,
    /// Page Up key
    PageUp,
    /// Page Down key
    PageDown,
    /// Shift key
    Shift,
    /// Control key
    Control,
    /// Alt / Option key
    Alt,
    /// Meta / Command / Windows key
    Meta,
    /// Caps Lock key
    CapsLock,
    /// Unknown key with platform-specific scan code
    Unknown(u32),
}

/// Active keyboard modifier keys
pub struct Modifiers {
    /// Shift key is held
    pub shift: bool,
    /// Control key is held
    pub ctrl: bool,
    /// Alt / Option key is held
    pub alt: bool,
    /// Meta / Command / Windows key is held
    pub meta: bool,
}

/// Mouse button
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MouseButton {
    /// Primary (left) button
    Left,
    /// Middle (scroll wheel click) button
    Middle,
    /// Secondary (right) button
    Right,
    /// Back side button (X1)
    Back,
    /// Forward side button (X2)
    Forward,
}

/// Window event
pub enum WindowEvent {
    /// Window create
    Create,
    /// Window move
    Move(LogicalPoint),
    /// Window resize
    Resize(LogicalSize),
    /// Window close
    Close,
    /// macOS window fullscreen change
    #[cfg(target_os = "macos")]
    MacosFullscreenChange(bool),
    /// Window gained keyboard focus
    Focus,
    /// Window lost keyboard focus
    Unfocus,
    /// Physical key pressed
    KeyDown {
        /// The key that was pressed
        key: KeyCode,
        /// Active modifier keys at time of press
        modifiers: Modifiers,
    },
    /// Physical key released
    KeyUp {
        /// The key that was released
        key: KeyCode,
        /// Active modifier keys at time of release
        modifiers: Modifiers,
    },
    /// Text character produced by a key press
    Char(char),
    /// Mouse button pressed
    MouseDown {
        /// The button that was pressed
        button: MouseButton,
        /// Cursor position in logical coordinates
        position: LogicalPoint,
    },
    /// Mouse button released
    MouseUp {
        /// The button that was released
        button: MouseButton,
        /// Cursor position in logical coordinates
        position: LogicalPoint,
    },
    /// Mouse cursor moved within the window
    MouseMove(LogicalPoint),
    /// Mouse cursor entered the window area
    MouseEnter,
    /// Mouse cursor left the window area
    MouseLeave,
    /// Mouse wheel or trackpad scroll; positive delta_y = down, positive delta_x = right
    MouseWheel {
        /// Horizontal scroll delta
        delta_x: f32,
        /// Vertical scroll delta
        delta_y: f32,
    },
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
}
