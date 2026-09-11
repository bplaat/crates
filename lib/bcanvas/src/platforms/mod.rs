/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::{PlatformCanvas, PlatformCanvasContext, PlatformOffscreenCanvas};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) use windows::{PlatformCanvas, PlatformCanvasContext, PlatformOffscreenCanvas};

#[cfg(not(any(target_os = "macos", windows)))]
mod gtk;
#[cfg(not(any(target_os = "macos", windows)))]
pub(crate) use gtk::{PlatformCanvas, PlatformCanvasContext, PlatformOffscreenCanvas};
