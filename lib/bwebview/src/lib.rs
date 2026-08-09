/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![allow(unused)]
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

#[cfg(feature = "canvas")]
pub use canvas::*;
#[cfg(feature = "dialog")]
pub use dialog::*;
pub use event::*;
pub use event_loop::*;
#[cfg(all(target_os = "macos", feature = "menu"))]
pub use menu::*;
pub use sizes::*;
#[cfg(feature = "webview")]
pub use webview::*;
pub use window::*;

#[cfg(feature = "canvas")]
mod canvas;
#[cfg(feature = "dialog")]
mod dialog;
mod event;
mod event_loop;
#[cfg(all(target_os = "macos", feature = "menu"))]
mod menu;
mod platforms;
mod sizes;
#[cfg(feature = "webview")]
mod webview;
mod window;
