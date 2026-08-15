/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_snake_case, non_upper_case_globals)]

use std::ffi::c_void;

use macview_appkit::Rect;
use objc2::runtime::AnyObject as Object;
use objc2::{class, msg_send};

pub(crate) const NS_APPLICATION_ACTIVATION_POLICY_REGULAR: i64 = 0;
pub(crate) const NS_BACKING_STORE_BUFFERED: u64 = 2;
pub(crate) const NS_EVENT_MODIFIER_FLAG_SHIFT: u64 = 1 << 17;
pub(crate) const NS_EVENT_MODIFIER_FLAG_OPTION: u64 = 1 << 19;
pub(crate) const NS_EVENT_MODIFIER_FLAG_COMMAND: u64 = 1 << 20;
pub(crate) const NS_IMAGE_SCALE_PROPORTIONALLY_UP_OR_DOWN: u64 = 3;
pub(crate) const NS_NO_BORDER: u64 = 0;
pub(crate) const NS_PRINTING_PAGINATION_MODE_FIT: u64 = 1;
pub(crate) const NS_UTF8_STRING_ENCODING: u64 = 4;
pub(crate) const NS_VIEW_HEIGHT_SIZABLE: u64 = 16;
pub(crate) const NS_VIEW_WIDTH_SIZABLE: u64 = 2;
pub(crate) const NS_WINDOW_STYLE_MASK_CLOSABLE: u64 = 1 << 1;
pub(crate) const NS_WINDOW_STYLE_MASK_MINIATURIZABLE: u64 = 1 << 2;
pub(crate) const NS_WINDOW_STYLE_MASK_RESIZABLE: u64 = 1 << 3;
pub(crate) const NS_WINDOW_STYLE_MASK_TITLED: u64 = 1;

#[link(name = "Cocoa", kind = "framework")]
unsafe extern "C" {}

#[link(name = "WebKit", kind = "framework")]
unsafe extern "C" {}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {
    pub(crate) static NSAppearanceNameAqua: *mut Object;
    pub(crate) static NSAppearanceNameDarkAqua: *mut Object;
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    pub(crate) fn CGContextFillRect(context: *mut c_void, rectangle: Rect);
    pub(crate) fn CGContextSetRGBFillColor(
        context: *mut c_void,
        red: f64,
        green: f64,
        blue: f64,
        alpha: f64,
    );
}

pub(crate) fn ns_string(value: &str) -> *mut Object {
    // SAFETY: NSString copies the valid UTF-8 bytes before the returned object is autoreleased.
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
