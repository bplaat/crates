/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_char;
use std::fmt::{self, Display, Formatter};

use super::objc::Object;
use crate::{class, msg_send};

#[link(name = "Cocoa", kind = "framework")]
unsafe extern "C" {
    pub(crate) static NSApp: *mut Object;
}

#[repr(C)]
pub(crate) struct NSPoint {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
pub(crate) struct NSSize {
    pub width: f64,
    pub height: f64,
}

#[repr(C)]
pub(crate) struct NSRect {
    pub origin: NSPoint,
    pub size: NSSize,
}

pub(crate) const NS_APPLICATION_ACTIVATION_POLICY_REGULAR: i32 = 0;

pub(crate) const NS_UTF8_STRING_ENCODING: i32 = 4;

pub(crate) const NS_WINDOW_STYLE_MASK_TITLED: i32 = 1;
pub(crate) const NS_WINDOW_STYLE_MASK_CLOSABLE: i32 = 2;
pub(crate) const NS_WINDOW_STYLE_MASK_MINIATURIZABLE: i32 = 4;
pub(crate) const NS_WINDOW_STYLE_MASK_RESIZABLE: i32 = 8;

pub(crate) const NS_BACKING_STORE_BUFFERED: i32 = 2;

pub(crate) struct NSString(pub(crate) *mut Object);

impl NSString {
    pub(crate) fn from_str(str: impl AsRef<str>) -> Self {
        let str = str.as_ref();
        unsafe {
            msg_send![
                msg_send![msg_send![class!(NSString), alloc], initWithBytes:str.as_ptr() length:str.len() encoding:NS_UTF8_STRING_ENCODING],
                autorelease
            ]
        }
    }
}

impl Display for NSString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe {
            let bytes: *const c_char = msg_send![self.0, UTF8String];
            let len: usize = msg_send![self.0, lengthOfBytesUsingEncoding:NS_UTF8_STRING_ENCODING];
            String::from_utf8_lossy(std::slice::from_raw_parts(bytes as *const u8, len as usize))
        })
    }
}
