/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![allow(unused)]
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

#[cfg(feature = "dialog")]
pub use dialog::*;
pub use event::*;
pub use event_loop::*;
pub use sizes::*;
pub use webview::*;
pub use window::*;

#[cfg(feature = "dialog")]
mod dialog;
mod event;
mod event_loop;
mod platforms;
mod sizes;
mod webview;
mod window;
