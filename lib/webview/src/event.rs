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

    /// Page load started
    PageLoadStarted,
    /// Page load finished
    PageLoadFinished,
    /// Ipc message received
    #[cfg(feature = "ipc")]
    PageMessageReceived(String),
}
