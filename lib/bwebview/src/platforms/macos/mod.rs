/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

mod cocoa;
mod event_loop;
mod file_dialog;
#[cfg(feature = "webview")]
mod webkit;
#[cfg(feature = "webview")]
mod webview;
mod window;

pub(crate) use event_loop::{PlatformEventLoop, PlatformEventLoopProxy, PlatformMonitor};
#[cfg(feature = "file_dialog")]
pub(crate) use file_dialog::PlatformFileDialog;
#[cfg(feature = "webview")]
pub(crate) use webview::PlatformWebview;
pub(crate) use window::PlatformWindow;
