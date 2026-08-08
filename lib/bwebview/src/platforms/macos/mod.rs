/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

mod cocoa;
#[cfg(feature = "dialog")]
mod dialog;
mod event_loop;
#[cfg(feature = "file_drop")]
mod file_drop;
mod menu;
mod webkit;
mod webview;
mod window;

#[cfg(feature = "dialog")]
pub(crate) use dialog::{PlatformFileDialog, PlatformMessageDialog};
pub(crate) use event_loop::{PlatformEventLoop, PlatformEventLoopProxy, PlatformMonitor};
pub(crate) use webview::PlatformWebview;
pub(crate) use window::PlatformWindow;
