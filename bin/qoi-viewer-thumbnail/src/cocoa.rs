/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_snake_case)]

use std::ffi::{c_char, c_void};

use qoi_appkit::Rect;

#[link(name = "Cocoa", kind = "framework")]
unsafe extern "C" {}

#[link(name = "QuickLookThumbnailing", kind = "framework")]
unsafe extern "C" {}

#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    pub(crate) fn NSExtensionMain(argc: i32, argv: *const *const c_char) -> i32;
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    pub(crate) fn CGContextDrawImage(context: *mut c_void, rectangle: Rect, image: *const c_void);
}
