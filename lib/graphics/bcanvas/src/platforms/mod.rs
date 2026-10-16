/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(not(any(target_os = "macos", windows)))]
pub(crate) use crate::platforms::gtk::{
    PlatformCanvas, PlatformCanvasContext, PlatformOffscreenCanvas,
};
#[cfg(target_os = "macos")]
pub(crate) use crate::platforms::macos::{
    PlatformCanvas, PlatformCanvasContext, PlatformOffscreenCanvas,
};
#[cfg(windows)]
pub(crate) use crate::platforms::windows::{
    PlatformCanvas, PlatformCanvasContext, PlatformOffscreenCanvas,
};

#[cfg(target_os = "macos")]
mod macos;

#[cfg(windows)]
mod windows;

#[cfg(not(any(target_os = "macos", windows)))]
mod gtk;
