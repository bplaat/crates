/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_snake_case, non_upper_case_globals)]

use std::ffi::c_void;

use objc2::{Encode, Encoding};

#[link(name = "Cocoa", kind = "framework")]
unsafe extern "C" {}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    pub(crate) fn CFDataCreate(
        allocator: *const c_void,
        bytes: *const u8,
        length: isize,
    ) -> *const c_void;
    pub(crate) fn CFRelease(object: *const c_void);
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    pub(crate) static kCGColorSpaceSRGB: *const c_void;
    pub(crate) static kCGColorSpaceLinearSRGB: *const c_void;

    pub(crate) fn CGColorSpaceCreateWithName(name: *const c_void) -> *const c_void;
    pub(crate) fn CGColorSpaceRelease(color_space: *const c_void);
    pub(crate) fn CGDataProviderCreateWithCFData(data: *const c_void) -> *const c_void;
    pub(crate) fn CGDataProviderRelease(provider: *const c_void);
    pub(crate) fn CGImageCreate(
        width: usize,
        height: usize,
        bits_per_component: usize,
        bits_per_pixel: usize,
        bytes_per_row: usize,
        color_space: *const c_void,
        bitmap_info: u32,
        provider: *const c_void,
        decode: *const f64,
        should_interpolate: bool,
        rendering_intent: i32,
    ) -> *const c_void;
    pub(crate) fn CGImageRelease(image: *const c_void);
}

/// A Core Graphics point.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Point {
    /// The horizontal coordinate.
    pub x: f64,
    /// The vertical coordinate.
    pub y: f64,
}

// SAFETY: Point matches the Core Graphics CGPoint ABI.
unsafe impl Encode for Point {
    const ENCODING: Encoding = Encoding::Struct("CGPoint", &[f64::ENCODING, f64::ENCODING]);
}

/// A Core Graphics size.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Size {
    /// The width.
    pub width: f64,
    /// The height.
    pub height: f64,
}

// SAFETY: Size matches the Core Graphics CGSize ABI.
unsafe impl Encode for Size {
    const ENCODING: Encoding = Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
}

/// A Core Graphics rectangle.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Rect {
    /// The origin.
    pub origin: Point,
    /// The size.
    pub size: Size,
}

// SAFETY: Rect matches the Core Graphics CGRect ABI.
unsafe impl Encode for Rect {
    const ENCODING: Encoding = Encoding::Struct("CGRect", &[Point::ENCODING, Size::ENCODING]);
}
