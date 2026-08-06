/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(feature = "dialog")]
mod dialog;
mod event_loop;
mod webview;
mod webview2;
mod win32;
mod window;

#[cfg(feature = "dialog")]
pub(crate) use dialog::{PlatformFileDialog, PlatformMessageDialog};
pub(crate) use event_loop::{PlatformEventLoop, PlatformEventLoopProxy, PlatformMonitor};
pub(crate) use webview::PlatformWebview;
pub(crate) use window::PlatformWindow;
