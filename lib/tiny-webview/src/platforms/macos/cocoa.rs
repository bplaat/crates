/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{c_char, c_void};
use std::fmt::{self, Display, Formatter};

use objc2::runtime::AnyObject as Object;
use objc2::{Encode, Encoding, class, msg_send};

#[link(name = "Cocoa", kind = "framework")]
unsafe extern "C" {
    pub(crate) static NSApp: *mut Object;
    pub(crate) static NSAppearanceNameAqua: *const Object;
    pub(crate) static NSAppearanceNameDarkAqua: *const Object;
}

#[repr(C)]
pub(crate) struct CGPoint {
    pub(crate) x: f64,
    pub(crate) y: f64,
}
impl CGPoint {
    pub(crate) fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
unsafe impl Encode for CGPoint {
    const ENCODING: Encoding = Encoding::Struct("CGPoint", &[f64::ENCODING, f64::ENCODING]);
}
pub(crate) type NSPoint = CGPoint;

#[repr(C)]
pub(crate) struct CGSize {
    pub(crate) width: f64,
    pub(crate) height: f64,
}
impl CGSize {
    pub(crate) fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}
unsafe impl Encode for CGSize {
    const ENCODING: Encoding = Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
}
pub(crate) type NSSize = CGSize;

#[repr(C)]
pub(crate) struct CGRect {
    pub(crate) origin: CGPoint,
    pub(crate) size: CGSize,
}
impl CGRect {
    pub(crate) fn new(origin: CGPoint, size: CGSize) -> Self {
        Self { origin, size }
    }
}
unsafe impl Encode for CGRect {
    const ENCODING: Encoding = Encoding::Struct("CGRect", &[CGPoint::ENCODING, CGSize::ENCODING]);
}
pub(crate) type NSRect = CGRect;

pub(crate) const NS_APPLICATION_ACTIVATION_POLICY_REGULAR: i64 = 0;

pub(crate) const NS_UTF8_STRING_ENCODING: u64 = 4;

pub(crate) const NS_WINDOW_STYLE_MASK_TITLED: u64 = 1 << 0;
pub(crate) const NS_WINDOW_STYLE_MASK_CLOSABLE: u64 = 1 << 1;
pub(crate) const NS_WINDOW_STYLE_MASK_MINIATURIZABLE: u64 = 1 << 2;
pub(crate) const NS_WINDOW_STYLE_MASK_RESIZABLE: u64 = 1 << 3;

pub(crate) const NS_BACKING_STORE_BUFFERED: u64 = 2;

pub(crate) const NS_WINDOW_TITLE_VISIBILITY_HIDDEN: i64 = 1;

#[repr(transparent)]
pub(crate) struct NSString(*mut Object);

unsafe impl Encode for NSString {
    const ENCODING: Encoding = Encoding::Object;
}

impl NSString {
    pub(crate) fn from_str(str: impl AsRef<str>) -> Self {
        let str = str.as_ref();
        unsafe {
            let ns_string: *mut Object = msg_send![class!(NSString), alloc];
            let ns_string: *mut Object = msg_send![ns_string, initWithBytes:str.as_ptr().cast::<c_void>(), length:str.len(), encoding:NS_UTF8_STRING_ENCODING];
            msg_send![ns_string, autorelease]
        }
    }
}
impl Display for NSString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe {
            let bytes: *const c_char = msg_send![self.0, UTF8String];
            let len: usize = msg_send![self.0, lengthOfBytesUsingEncoding:NS_UTF8_STRING_ENCODING];
            String::from_utf8_lossy(std::slice::from_raw_parts(bytes as *const u8, len))
        })
    }
}
