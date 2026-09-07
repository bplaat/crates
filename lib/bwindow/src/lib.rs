/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![allow(unused)]
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

pub use content::*;
#[cfg(feature = "dialog")]
pub use dialog::*;
pub use event::*;
pub use event_loop::*;
pub use input::*;
#[cfg(all(target_os = "macos", feature = "menu"))]
pub use menu::*;
pub use sizes::*;
pub use window::*;

mod content;
#[cfg(feature = "dialog")]
mod dialog;
mod dispatch;
mod event;
mod event_loop;
mod input;
mod mailbox;
#[cfg(all(target_os = "macos", feature = "menu"))]
mod menu;
mod platforms;
mod sizes;
mod window;

/// Native ABI declarations for implementing content backends.
///
/// These unsafe bindings are platform-specific and are not a portable window API.
#[doc(hidden)]
#[allow(missing_docs)]
pub mod ffi {
    #[cfg(all(not(any(target_os = "macos", windows)), feature = "file_drop"))]
    pub use crate::platforms::gtk::file_drop::*;
    #[cfg(not(any(target_os = "macos", windows)))]
    pub use crate::platforms::gtk::input::*;
    #[cfg(all(target_os = "macos", feature = "file_drop"))]
    pub use crate::platforms::macos::file_drop::{perform_file_drop, register_dragged_types};
    #[cfg(target_os = "macos")]
    pub use crate::platforms::macos::input::*;
    pub use crate::platforms::native_headers::*;
    #[cfg(windows)]
    pub use crate::platforms::windows::input::*;
}
