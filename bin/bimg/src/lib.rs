/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! BImg - shared Cocoa bindings for QOI image rendering on macOS.

#![cfg(target_os = "macos")]
#![allow(unsafe_code)]

use std::ffi::c_void;
use std::ptr::null_mut;

use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{Encode, Encoding, class, msg_send};

// MARK: ns_string! support

// Apple's __CFConstantStringClassReference is the isa pointer for CFConstant NSString literals.
#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    #[doc(hidden)]
    pub static __CFConstantStringClassReference: Object;
}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}

/// Internal struct matching Apple's `__CFConstantString` ABI. Used only by `ns_string!`.
#[doc(hidden)]
#[repr(C)]
pub struct CFConstString {
    /// isa pointer to the NSString constant class.
    pub isa: *const c_void,
    /// CFString info flags (0x07C8 for immutable ASCII literal).
    pub cfinfo: u32,
    #[cfg(target_pointer_width = "64")]
    /// Reference count placeholder (0 for static strings).
    pub _rc: u32,
    /// Pointer to the null-terminated UTF-8 string bytes.
    pub data: *const u8,
    /// Number of bytes in the string (excluding the null terminator).
    pub len: usize,
}

// SAFETY: CFConstString is an immutable static literal; it is safe to share across threads.
unsafe impl Send for CFConstString {}
// SAFETY: CFConstString is an immutable static literal; it is safe to share across threads.
unsafe impl Sync for CFConstString {}

/// Creates a zero-cost `NSString` literal from an ASCII string constant.
///
/// The argument must be a string literal containing only ASCII characters and no NUL bytes.
/// Do not call inside closures - hoist to enclosing function scope (rustc bug madsmtm/objc2#258).
#[macro_export]
macro_rules! ns_string {
    ($s:expr) => {{
        const INPUT: &str = $s;
        const BYTES: &[u8] = INPUT.as_bytes();
        const _: () = {
            let mut i = 0usize;
            while i < BYTES.len() {
                if !BYTES[i].is_ascii() || BYTES[i] == b'\0' {
                    panic!("ns_string! only supports ASCII strings without NUL bytes");
                }
                i += 1;
            }
        };
        #[unsafe(link_section = "__TEXT,__cstring,cstring_literals")]
        static DATA: [u8; BYTES.len() + 1] = {
            let mut arr = [0u8; BYTES.len() + 1];
            let mut i = 0usize;
            while i < BYTES.len() {
                arr[i] = BYTES[i];
                i += 1;
            }
            arr
        };
        #[unsafe(link_section = "__DATA,__cfstring")]
        static CFSTRING: $crate::CFConstString = unsafe {
            $crate::CFConstString {
                isa: &$crate::__CFConstantStringClassReference
                    as *const ::objc2::runtime::AnyObject
                    as *const ::std::ffi::c_void,
                cfinfo: 0x07C8,
                #[cfg(target_pointer_width = "64")]
                _rc: 0,
                data: DATA.as_ptr(),
                len: BYTES.len(),
            }
        };
        &CFSTRING as *const $crate::CFConstString as *mut ::objc2::runtime::AnyObject
    }};
}

// MARK: NSString helper (private)

const NS_UTF8_STRING_ENCODING: u64 = 4;

// Returns an autoreleased NSString* for the given Rust string slice.
fn nsstring_from_str(s: impl AsRef<str>) -> *mut Object {
    let s = s.as_ref();
    // SAFETY: NSString is a valid Foundation class; initWithBytes:length:encoding: is a
    // standard initializer for creating UTF-8 strings from raw bytes.
    unsafe {
        let obj: *mut Object = msg_send![class!(NSString), alloc];
        let obj: *mut Object = msg_send![obj,
            initWithBytes: s.as_ptr() as *const c_void,
            length: s.len() as u64,
            encoding: NS_UTF8_STRING_ENCODING
        ];
        msg_send![obj, autorelease]
    }
}

// MARK: Geometry types

/// A 2D point matching the `NSPoint` / `CGPoint` struct layout.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct NSPoint {
    /// Horizontal coordinate.
    pub x: f64,
    /// Vertical coordinate.
    pub y: f64,
}

impl NSPoint {
    /// Creates a new `NSPoint` at `(x, y)`.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

// SAFETY: NSPoint/CGPoint is a C struct with two f64 fields; ObjC encoding is {CGPoint=dd}.
unsafe impl Encode for NSPoint {
    const ENCODING: Encoding = Encoding::Struct("CGPoint", &[f64::ENCODING, f64::ENCODING]);
}

/// A 2D size matching the `NSSize` / `CGSize` struct layout.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct NSSize {
    /// Width in points.
    pub width: f64,
    /// Height in points.
    pub height: f64,
}

impl NSSize {
    /// Creates a new `NSSize` from `width` and `height`.
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

// SAFETY: NSSize/CGSize is a C struct with two f64 fields; ObjC encoding is {CGSize=dd}.
unsafe impl Encode for NSSize {
    const ENCODING: Encoding = Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
}

/// A 2D rectangle matching the `NSRect` / `CGRect` struct layout.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct NSRect {
    /// The origin (top-left corner on macOS with flipped coordinates).
    pub origin: NSPoint,
    /// The size of the rectangle.
    pub size: NSSize,
}

impl NSRect {
    /// Creates a new `NSRect` from an `origin` and a `size`.
    pub fn new(origin: NSPoint, size: NSSize) -> Self {
        Self { origin, size }
    }
}

// SAFETY: NSRect/CGRect is {CGPoint, CGSize}; ObjC encoding is {CGRect={CGPoint=dd}{CGSize=dd}}.
unsafe impl Encode for NSRect {
    const ENCODING: Encoding =
        Encoding::Struct("CGRect", &[NSPoint::ENCODING, NSSize::ENCODING]);
}

// MARK: Image creation

/// Creates an `NSImage` from raw RGBA pixel data and returns a `+1`-retained pointer.
///
/// `pixels` must contain exactly `width * height * 4` bytes in RGBA order.
/// The caller is responsible for releasing the returned object.
pub fn create_nsimage(pixels: &[u8], width: u32, height: u32) -> *mut Object {
    // SAFETY: NSBitmapImageRep is a valid AppKit class. initWithBitmapDataPlanes:NULL lets AppKit
    // allocate the pixel buffer; we then copy our decoded pixels in via bitmapData.
    unsafe {
        let rep: *mut Object = msg_send![class!(NSBitmapImageRep), alloc];
        let rep: *mut Object = msg_send![rep,
            initWithBitmapDataPlanes: null_mut::<c_void>(),
            pixelsWide: width as i64,
            pixelsHigh: height as i64,
            bitsPerSample: 8i64,
            samplesPerPixel: 4i64,
            hasAlpha: Bool::YES,
            isPlanar: Bool::NO,
            colorSpaceName: nsstring_from_str("NSCalibratedRGBColorSpace"),
            bytesPerRow: (width as i64) * 4,
            bitsPerPixel: 32i64
        ];
        let bitmap_data: *mut c_void = msg_send![rep, bitmapData];
        std::ptr::copy_nonoverlapping(pixels.as_ptr(), bitmap_data as *mut u8, pixels.len());

        let image: *mut Object = msg_send![class!(NSImage), alloc];
        let image: *mut Object =
            msg_send![image, initWithSize: NSSize::new(width as f64, height as f64)];
        let _: () = msg_send![image, addRepresentation: rep];
        let _: () = msg_send![rep, release];
        image
    }
}
