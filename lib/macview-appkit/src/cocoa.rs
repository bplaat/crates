/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_snake_case, non_upper_case_globals)]

use std::ffi::c_void;

use objc2::{Encode, Encoding};

pub(crate) const DRAW_PATH_EVEN_ODD_FILL: i32 = 1;
pub(crate) const DRAW_PATH_STROKE: i32 = 2;
pub(crate) const GRADIENT_DRAWS_AFTER_END: u32 = 2;
pub(crate) const GRADIENT_DRAWS_BEFORE_START: u32 = 1;
pub(crate) const LINE_CAP_ROUND: i32 = 1;
pub(crate) const LINE_JOIN_ROUND: i32 = 1;

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
    #[cfg(test)]
    pub(crate) fn CGColorSpaceCreateDeviceRGB() -> *const c_void;
    pub(crate) fn CGColorSpaceRelease(color_space: *const c_void);
    #[cfg(test)]
    pub(crate) fn CGBitmapContextCreate(
        data: *mut c_void,
        width: usize,
        height: usize,
        bits_per_component: usize,
        bytes_per_row: usize,
        color_space: *const c_void,
        bitmap_info: u32,
    ) -> *mut c_void;
    pub(crate) fn CGContextAddCurveToPoint(
        context: *mut c_void,
        control_0_x: f64,
        control_0_y: f64,
        control_1_x: f64,
        control_1_y: f64,
        x: f64,
        y: f64,
    );
    pub(crate) fn CGContextAddLineToPoint(context: *mut c_void, x: f64, y: f64);
    pub(crate) fn CGContextAddQuadCurveToPoint(
        context: *mut c_void,
        control_x: f64,
        control_y: f64,
        x: f64,
        y: f64,
    );
    pub(crate) fn CGContextBeginPath(context: *mut c_void);
    pub(crate) fn CGContextClip(context: *mut c_void);
    pub(crate) fn CGContextClosePath(context: *mut c_void);
    pub(crate) fn CGContextDrawLinearGradient(
        context: *mut c_void,
        gradient: *const c_void,
        start: TinyVgPoint,
        end: TinyVgPoint,
        options: u32,
    );
    pub(crate) fn CGContextDrawPath(context: *mut c_void, mode: i32);
    pub(crate) fn CGContextDrawRadialGradient(
        context: *mut c_void,
        gradient: *const c_void,
        start_center: TinyVgPoint,
        start_radius: f64,
        end_center: TinyVgPoint,
        end_radius: f64,
        options: u32,
    );
    pub(crate) fn CGContextEOClip(context: *mut c_void);
    pub(crate) fn CGContextFillRect(context: *mut c_void, rectangle: Rect);
    pub(crate) fn CGContextMoveToPoint(context: *mut c_void, x: f64, y: f64);
    #[cfg(test)]
    pub(crate) fn CGContextRelease(context: *mut c_void);
    pub(crate) fn CGContextReplacePathWithStrokedPath(context: *mut c_void);
    pub(crate) fn CGContextRestoreGState(context: *mut c_void);
    pub(crate) fn CGContextSaveGState(context: *mut c_void);
    pub(crate) fn CGContextScaleCTM(context: *mut c_void, x: f64, y: f64);
    pub(crate) fn CGContextSetLineCap(context: *mut c_void, line_cap: i32);
    pub(crate) fn CGContextSetLineJoin(context: *mut c_void, line_join: i32);
    pub(crate) fn CGContextSetLineWidth(context: *mut c_void, width: f64);
    pub(crate) fn CGContextSetRGBFillColor(
        context: *mut c_void,
        red: f64,
        green: f64,
        blue: f64,
        alpha: f64,
    );
    pub(crate) fn CGContextSetRGBStrokeColor(
        context: *mut c_void,
        red: f64,
        green: f64,
        blue: f64,
        alpha: f64,
    );
    pub(crate) fn CGContextTranslateCTM(context: *mut c_void, x: f64, y: f64);
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
    pub(crate) fn CGGradientCreateWithColorComponents(
        color_space: *const c_void,
        components: *const f64,
        locations: *const f64,
        count: usize,
    ) -> *const c_void;
    pub(crate) fn CGGradientRelease(gradient: *const c_void);
}

type TinyVgPoint = tinyvg::Point;

/// A Core Graphics point.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Point {
    /// The horizontal coordinate.
    pub x: f64,
    /// The vertical coordinate.
    pub y: f64,
}

pub(crate) type CgPoint = Point;

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
