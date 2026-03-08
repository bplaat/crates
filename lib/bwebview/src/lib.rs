/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![allow(unused)]

pub use event_loop::*;
#[cfg(feature = "file_dialog")]
pub use file_dialog::*;
pub use input::*;
pub use sizes::*;
#[cfg(feature = "webview")]
pub use webview::*;
pub use window::*;

mod event_loop;
#[cfg(feature = "file_dialog")]
mod file_dialog;
mod input;
mod platforms;
mod sizes;
#[cfg(feature = "webview")]
mod webview;
mod window;
