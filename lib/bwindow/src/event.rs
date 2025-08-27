/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::dpi::{LogicalPoint, LogicalSize};

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

    /// User event
    UserEvent(Vec<u8>),
}
