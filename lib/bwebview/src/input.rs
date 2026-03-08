/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// MARK: Key
/// Keyboard key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// The `A` key
    A,
    /// The `B` key
    B,
    /// The `C` key
    C,
    /// The `D` key
    D,
    /// The `E` key
    E,
    /// The `F` key
    F,
    /// The `G` key
    G,
    /// The `H` key
    H,
    /// The `I` key
    I,
    /// The `J` key
    J,
    /// The `K` key
    K,
    /// The `L` key
    L,
    /// The `M` key
    M,
    /// The `N` key
    N,
    /// The `O` key
    O,
    /// The `P` key
    P,
    /// The `Q` key
    Q,
    /// The `R` key
    R,
    /// The `S` key
    S,
    /// The `T` key
    T,
    /// The `U` key
    U,
    /// The `V` key
    V,
    /// The `W` key
    W,
    /// The `X` key
    X,
    /// The `Y` key
    Y,
    /// The `Z` key
    Z,
    /// The `0` digit key
    Digit0,
    /// The `1` digit key
    Digit1,
    /// The `2` digit key
    Digit2,
    /// The `3` digit key
    Digit3,
    /// The `4` digit key
    Digit4,
    /// The `5` digit key
    Digit5,
    /// The `6` digit key
    Digit6,
    /// The `7` digit key
    Digit7,
    /// The `8` digit key
    Digit8,
    /// The `9` digit key
    Digit9,
    /// The `F1` function key
    F1,
    /// The `F2` function key
    F2,
    /// The `F3` function key
    F3,
    /// The `F4` function key
    F4,
    /// The `F5` function key
    F5,
    /// The `F6` function key
    F6,
    /// The `F7` function key
    F7,
    /// The `F8` function key
    F8,
    /// The `F9` function key
    F9,
    /// The `F10` function key
    F10,
    /// The `F11` function key
    F11,
    /// The `F12` function key
    F12,
    /// The `Escape` key
    Escape,
    /// The `Enter` / `Return` key
    Enter,
    /// The `Backspace` key
    Backspace,
    /// The `Tab` key
    Tab,
    /// The `Space` bar
    Space,
    /// The `Delete` (forward-delete) key
    Delete,
    /// The `Insert` key
    Insert,
    /// The `ArrowUp` key
    ArrowUp,
    /// The `ArrowDown` key
    ArrowDown,
    /// The `ArrowLeft` key
    ArrowLeft,
    /// The `ArrowRight` key
    ArrowRight,
    /// The `Home` key
    Home,
    /// The `End` key
    End,
    /// The `PageUp` key
    PageUp,
    /// The `PageDown` key
    PageDown,
    /// The `Shift` modifier key
    Shift,
    /// The `Control` modifier key
    Control,
    /// The `Alt` / `Option` modifier key
    Alt,
    /// The `Meta` / `Command` / `Windows` modifier key
    Meta,
    /// The `-` key
    Minus,
    /// The `=` key
    Equal,
    /// The `[` key
    BracketLeft,
    /// The `]` key
    BracketRight,
    /// The `\` key
    Backslash,
    /// The `;` key
    Semicolon,
    /// The `'` key
    Quote,
    /// The `,` key
    Comma,
    /// The `.` key
    Period,
    /// The `/` key
    Slash,
    /// The `` ` `` key
    Backtick,
    /// The numpad `0` key
    Numpad0,
    /// The numpad `1` key
    Numpad1,
    /// The numpad `2` key
    Numpad2,
    /// The numpad `3` key
    Numpad3,
    /// The numpad `4` key
    Numpad4,
    /// The numpad `5` key
    Numpad5,
    /// The numpad `6` key
    Numpad6,
    /// The numpad `7` key
    Numpad7,
    /// The numpad `8` key
    Numpad8,
    /// The numpad `9` key
    Numpad9,
    /// The numpad `+` key
    NumpadAdd,
    /// The numpad `-` key
    NumpadSubtract,
    /// The numpad `*` key
    NumpadMultiply,
    /// The numpad `/` key
    NumpadDivide,
    /// The numpad `Enter` key
    NumpadEnter,
    /// The numpad `.` (decimal) key
    NumpadDecimal,
    /// The `CapsLock` key
    CapsLock,
    /// The `PrintScreen` key
    PrintScreen,
    /// The `Pause` / `Break` key
    Pause,
    /// An unrecognised key
    Unknown,
}

// MARK: Modifiers
/// Keyboard modifier keys bitflags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers(u8);

impl Modifiers {
    /// Shift modifier
    pub const SHIFT: Modifiers = Modifiers(1);
    /// Control modifier
    pub const CTRL: Modifiers = Modifiers(2);
    /// Alt / Option modifier
    pub const ALT: Modifiers = Modifiers(4);
    /// Meta / Command / Windows modifier
    pub const META: Modifiers = Modifiers(8);

    /// Returns a Modifiers with no bits set
    pub fn empty() -> Self {
        Modifiers(0)
    }

    /// Returns true if no modifiers are active
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns true if all bits of `other` are set in self
    pub fn contains(self, other: Modifiers) -> bool {
        self.0 & other.0 != 0
    }
}

impl std::ops::BitOr for Modifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Modifiers(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Modifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

// MARK: MouseButton
/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Right mouse button
    Right,
    /// Middle mouse button (scroll wheel click)
    Middle,
    /// Back side button
    Back,
    /// Forward side button
    Forward,
    /// Other button by index
    Other(u8),
}
