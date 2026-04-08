/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![allow(unused)]
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

pub use event::*;
pub use event_loop::*;
#[cfg(feature = "file_dialog")]
pub use file_dialog::*;
pub use sizes::*;
pub use webview::*;
pub use window::*;

mod event;
mod event_loop;
#[cfg(feature = "file_dialog")]
mod file_dialog;
mod platforms;
mod sizes;
mod webview;
mod window;
