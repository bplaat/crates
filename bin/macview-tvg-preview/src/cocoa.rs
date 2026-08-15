/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_snake_case)]

use std::ffi::{c_char, c_void};

use objc2::runtime::AnyObject as Object;
use objc2::{class, msg_send};

pub(crate) const NS_UTF8_STRING_ENCODING: u64 = 4;
pub(crate) const NS_VIEW_HEIGHT_SIZABLE: u64 = 16;
pub(crate) const NS_VIEW_WIDTH_SIZABLE: u64 = 2;

#[link(name = "Cocoa", kind = "framework")]
unsafe extern "C" {}

#[link(name = "QuickLookUI", kind = "framework")]
unsafe extern "C" {}

#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    pub(crate) fn NSExtensionMain(argc: i32, argv: *const *const c_char) -> i32;
}

pub(crate) fn ns_string(value: &str) -> *mut Object {
    // SAFETY: NSString copies the valid UTF-8 bytes before the object is autoreleased.
    unsafe {
        let string: *mut Object = msg_send![class!(NSString), alloc];
        let string: *mut Object = msg_send![string,
            initWithBytes: value.as_ptr().cast::<c_void>(),
            length: value.len(),
            encoding: NS_UTF8_STRING_ENCODING
        ];
        msg_send![string, autorelease]
    }
}
