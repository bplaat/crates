/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![allow(unused)]
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use bwindow::{NativeWindowHandle, Window, WindowAttachment, WindowEvent, WindowId};
pub use event::*;
pub use webview::*;
mod event;
mod platforms;
mod webview;
