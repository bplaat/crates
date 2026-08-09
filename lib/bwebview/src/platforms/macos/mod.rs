/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(feature = "canvas")]
mod canvas;
mod cocoa;
#[cfg(feature = "dialog")]
mod dialog;
mod event_loop;
#[cfg(feature = "file_drop")]
mod file_drop;
mod menu;
#[cfg(feature = "webview")]
mod webkit;
#[cfg(feature = "webview")]
mod webview;
mod window;

#[cfg(feature = "canvas")]
pub(crate) use canvas::{PlatformCanvas, PlatformCanvasContext};
#[cfg(feature = "dialog")]
pub(crate) use dialog::{PlatformFileDialog, PlatformMessageDialog};
pub(crate) use event_loop::{PlatformEventLoop, PlatformEventLoopProxy, PlatformMonitor};
#[cfg(feature = "webview")]
pub(crate) use webview::PlatformWebview;
pub(crate) use window::PlatformWindow;
