/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(feature = "dialog")]
mod dialog;
mod event_loop;
#[cfg(feature = "file_drop")]
mod file_drop;
mod headers;
#[cfg(feature = "progress_bar")]
mod progress_bar;
mod webview;
mod window;
#[cfg(feature = "remember_window_state")]
mod window_state;

#[cfg(feature = "dialog")]
pub(crate) use dialog::{PlatformFileDialog, PlatformMessageDialog};
pub(crate) use event_loop::{PlatformEventLoop, PlatformEventLoopProxy, PlatformMonitor};
pub(crate) use webview::PlatformWebview;
pub(crate) use window::PlatformWindow;
