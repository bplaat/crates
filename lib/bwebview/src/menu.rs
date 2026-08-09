/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ops::{BitOr, BitOrAssign};

/// macOS menu shortcut modifiers
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Modifiers(u8);

impl Modifiers {
    /// No modifiers
    pub const NONE: Self = Self(0);
    /// Command key
    pub const COMMAND: Self = Self(1 << 0);
    /// Control key
    pub const CONTROL: Self = Self(1 << 1);
    /// Option key
    pub const OPTION: Self = Self(1 << 2);
    /// Alt key (alias for Option)
    pub const ALT: Self = Self::OPTION;
    /// Super/Meta key (alias for Command on macOS)
    pub const SUPER: Self = Self::COMMAND;
    /// Shift key
    pub const SHIFT: Self = Self(1 << 3);

    /// Returns whether every modifier in `other` is set
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for Modifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Modifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Physical key used by a macOS menu accelerator
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// A key
    KeyA,
    /// B key
    KeyB,
    /// C key
    KeyC,
    /// D key
    KeyD,
    /// E key
    KeyE,
    /// F key
    KeyF,
    /// G key
    KeyG,
    /// H key
    KeyH,
    /// I key
    KeyI,
    /// J key
    KeyJ,
    /// K key
    KeyK,
    /// L key
    KeyL,
    /// M key
    KeyM,
    /// N key
    KeyN,
    /// O key
    KeyO,
    /// P key
    KeyP,
    /// Q key
    KeyQ,
    /// R key
    KeyR,
    /// S key
    KeyS,
    /// T key
    KeyT,
    /// U key
    KeyU,
    /// V key
    KeyV,
    /// W key
    KeyW,
    /// X key
    KeyX,
    /// Y key
    KeyY,
    /// Z key
    KeyZ,
    /// Top-row 0 key
    Digit0,
    /// Top-row 1 key
    Digit1,
    /// Top-row 2 key
    Digit2,
    /// Top-row 3 key
    Digit3,
    /// Top-row 4 key
    Digit4,
    /// Top-row 5 key
    Digit5,
    /// Top-row 6 key
    Digit6,
    /// Top-row 7 key
    Digit7,
    /// Top-row 8 key
    Digit8,
    /// Top-row 9 key
    Digit9,
    /// F1 key
    F1,
    /// F2 key
    F2,
    /// F3 key
    F3,
    /// F4 key
    F4,
    /// F5 key
    F5,
    /// F6 key
    F6,
    /// F7 key
    F7,
    /// F8 key
    F8,
    /// F9 key
    F9,
    /// F10 key
    F10,
    /// F11 key
    F11,
    /// F12 key
    F12,
    /// F13 key
    F13,
    /// F14 key
    F14,
    /// F15 key
    F15,
    /// F16 key
    F16,
    /// F17 key
    F17,
    /// F18 key
    F18,
    /// F19 key
    F19,
    /// F20 key
    F20,
    /// F21 key
    F21,
    /// F22 key
    F22,
    /// F23 key
    F23,
    /// F24 key
    F24,
    /// Backquote key
    Backquote,
    /// Backslash key
    Backslash,
    /// Left bracket key
    BracketLeft,
    /// Right bracket key
    BracketRight,
    /// Comma key
    Comma,
    /// Equal key
    Equal,
    /// Minus key
    Minus,
    /// Period key
    Period,
    /// Quote key
    Quote,
    /// Semicolon key
    Semicolon,
    /// Slash key
    Slash,
    /// Space key
    Space,
    /// Tab key
    Tab,
    /// Backspace key
    Backspace,
    /// Forward-delete key
    Delete,
    /// Enter/Return key
    Enter,
    /// Escape key
    Escape,
    /// Up arrow key
    ArrowUp,
    /// Down arrow key
    ArrowDown,
    /// Left arrow key
    ArrowLeft,
    /// Right arrow key
    ArrowRight,
    /// Home key
    Home,
    /// End key
    End,
    /// Page Up key
    PageUp,
    /// Page Down key
    PageDown,
}

/// Typed macOS menu keyboard accelerator
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Accelerator {
    pub(crate) modifiers: Modifiers,
    pub(crate) key: KeyCode,
}

impl Accelerator {
    /// Create an accelerator from modifiers and a physical key code
    pub const fn new(modifiers: Modifiers, key: KeyCode) -> Self {
        Self { modifiers, key }
    }
}

/// macOS menu item builder
pub struct MenuItem {
    pub(crate) title: String,
    pub(crate) action: String,
    pub(crate) accelerator: Option<Accelerator>,
}

/// A native menu command routed through the macOS responder chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItemRole {
    /// Show the application About panel.
    About,
    /// Hide the application.
    Hide,
    /// Hide other applications.
    HideOthers,
    /// Show all applications.
    ShowAll,
    /// Quit the application.
    Quit,
    /// Close the active window.
    Close,
    /// Undo in the active responder.
    Undo,
    /// Redo in the active responder.
    Redo,
    /// Cut the selection.
    Cut,
    /// Copy the selection.
    Copy,
    /// Paste from the clipboard.
    Paste,
    /// Delete the selection.
    Delete,
    /// Select all content.
    SelectAll,
    /// Minimize the active window.
    Minimize,
    /// Toggle the active window zoom state.
    Zoom,
}

impl MenuItem {
    /// Create a menu item with a title and stable action identifier
    pub fn new(title: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            action: action.into(),
            accelerator: None,
        }
    }

    /// Set the item's keyboard accelerator
    pub const fn accelerator(mut self, accelerator: Accelerator) -> Self {
        self.accelerator = Some(accelerator);
        self
    }
}

pub(crate) enum MenuBuilderEntry {
    Item(MenuItem),
    Role(MenuItemRole),
    Separator,
}

/// Builder for one macOS menu
pub struct MenuBuilder {
    pub(crate) title: String,
    pub(crate) entries: Vec<MenuBuilderEntry>,
}

impl MenuBuilder {
    /// Create an empty named menu
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            entries: Vec::new(),
        }
    }

    /// Add a menu item
    pub fn item(mut self, item: MenuItem) -> Self {
        self.entries.push(MenuBuilderEntry::Item(item));
        self
    }

    /// Add a standard native item with its conventional action and shortcut.
    pub fn role(mut self, role: MenuItemRole) -> Self {
        self.entries.push(MenuBuilderEntry::Role(role));
        self
    }

    /// Add a separator
    pub fn separator(mut self) -> Self {
        self.entries.push(MenuBuilderEntry::Separator);
        self
    }
}

/// Builder for custom macOS menu bar entries
///
/// Menus with standard names such as `File` and `Edit` are merged into bwebview's
/// complete default menu bar. Same-titled items are overridden in place and
/// custom shortcuts take precedence over defaults.
#[derive(Default)]
pub struct MenuBarBuilder {
    pub(crate) menus: Vec<MenuBuilder>,
}

impl MenuBarBuilder {
    /// Create an empty menu bar builder
    pub const fn new() -> Self {
        Self { menus: Vec::new() }
    }

    /// Add or extend a menu
    pub fn menu(mut self, menu: MenuBuilder) -> Self {
        self.menus.push(menu);
        self
    }
}
