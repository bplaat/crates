/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [raw-window-handle](https://crates.io/crates/raw-window-handle) crate

use std::ffi::c_void;
use std::ptr::NonNull;

/// Raw window handle for AppKit
pub struct AppKitWindowHandle {
    /// A pointer to an `NSView` object.
    pub ns_view: NonNull<c_void>,
}

/// A window handle for a particular windowing system.
#[non_exhaustive]
pub enum RawWindowHandle {
    /// A raw window handle for AppKit.
    AppKit(AppKitWindowHandle),
}
