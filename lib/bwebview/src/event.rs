/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::{LogicalPoint, LogicalSize};

/// Window identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub(crate) u32);

impl WindowId {
    /// Get raw ID value
    pub fn id(&self) -> u32 {
        self.0
    }
}

/// Window event
pub enum WindowEvent {
    /// Window created
    Created,
    /// Window moved
    Moved(LogicalPoint),
    /// Window resized
    Resized(LogicalSize),
    /// Window closed
    Closed,
    /// macOS window fullscreen changed
    #[cfg(target_os = "macos")]
    MacosFullscreenChanged(bool),
}

/// Webview event
pub enum WebviewEvent {
    /// Page load started
    PageLoadStarted,
    /// Page load finished
    PageLoadFinished,
    /// Page title changed
    PageTitleChanged(String),
    /// IPC message received
    MessageReceived(String),
}

/// Event
pub enum Event {
    /// Window event
    Window(WindowId, WindowEvent),
    /// Webview event
    Webview(WindowId, WebviewEvent),
    /// User event
    UserEvent(String),
}
