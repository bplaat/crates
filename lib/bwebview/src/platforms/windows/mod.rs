/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

mod event_loop;
mod file_dialog;
mod webview;
mod webview2;
mod win32;

pub(crate) use event_loop::{PlatformEventLoop, PlatformEventLoopProxy, PlatformMonitor};
#[cfg(feature = "file_dialog")]
pub(crate) use file_dialog::PlatformFileDialog;
pub(crate) use webview::PlatformWebview;
