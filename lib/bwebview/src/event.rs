/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::{LogicalPoint, LogicalSize};

/// Event
#[repr(C)]
pub enum Event {
    /// Window created
    WindowCreated,
    /// Window moved
    WindowMoved(LogicalPoint),
    /// Window resized
    WindowResized(LogicalSize),
    /// Window closed
    WindowClosed,
    /// macOS Window fullscreen changed
    #[cfg(target_os = "macos")]
    MacosWindowFullscreenChanged(bool),

    /// Page load started
    PageLoadStarted,
    /// Page load finished
    PageLoadFinished,
    /// Page title changed
    PageTitleChanged(String),
    /// Ipc message received
    PageMessageReceived(String),

    /// User event
    UserEvent(String),
}
