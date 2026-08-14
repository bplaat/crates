/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Shared AppKit image conversion for the QOI viewer and Quick Look extensions.

#![allow(unsafe_code)]

use std::ptr::null;

use objc2::runtime::AnyObject as Object;
use objc2::{class, msg_send};

mod cocoa;

use cocoa::*;
pub use cocoa::{Point, Rect, Size};

/// Loads and decodes a QOI image from a file URL into an owned `NSImage`.
///
/// The caller owns the returned object and must send it `release`.
pub fn load_image(url: *mut Object) -> Result<(*mut Object, Size), String> {
    // SAFETY: The caller supplies a valid file URL. NSData owns its bytes for this scope.
    let data: *mut Object = unsafe { msg_send![class!(NSData), dataWithContentsOfURL: url] };
    if data.is_null() {
        return Err(String::from("Could not read the QOI image"));
    }
    // SAFETY: NSData keeps its immutable byte buffer alive for the duration of this function.
    let bytes = unsafe {
        let length: usize = msg_send![data, length];
        let bytes: *const std::ffi::c_void = msg_send![data, bytes];
        std::slice::from_raw_parts(bytes.cast::<u8>(), length)
    };
    let decoded = qoi::decode(bytes).map_err(|error| error.to_string())?;
    let size = Size {
        width: f64::from(decoded.width()),
        height: f64::from(decoded.height()),
    };
    let image = make_image(&decoded).ok_or_else(|| String::from("Could not create the image"))?;
    Ok((image, size))
}

/// Creates an owned `NSImage` from a decoded QOI image.
///
/// The caller owns the returned object and must send it `release`.
pub fn make_image(image: &qoi::Image) -> Option<*mut Object> {
    // SAFETY: Core Foundation copies the decoded bytes. Each create call is checked, and every
    // owned Core Foundation/Core Graphics object is released after ownership is transferred.
    unsafe {
        let data = CFDataCreate(
            null(),
            image.pixels().as_ptr(),
            image.pixels().len() as isize,
        );
        if data.is_null() {
            return None;
        }

        let provider = CGDataProviderCreateWithCFData(data);
        CFRelease(data);
        if provider.is_null() {
            return None;
        }

        let color_space_name = match image.color_space() {
            qoi::ColorSpace::Srgb => kCGColorSpaceSRGB,
            qoi::ColorSpace::Linear => kCGColorSpaceLinearSRGB,
        };
        let color_space = CGColorSpaceCreateWithName(color_space_name);
        if color_space.is_null() {
            CGDataProviderRelease(provider);
            return None;
        }

        // kCGImageAlphaLast is 3: the source pixels are straight-alpha RGBA.
        let cg_image = CGImageCreate(
            image.width() as usize,
            image.height() as usize,
            8,
            32,
            image.width() as usize * 4,
            color_space,
            3,
            provider,
            null(),
            true,
            0,
        );
        CGColorSpaceRelease(color_space);
        CGDataProviderRelease(provider);
        if cg_image.is_null() {
            return None;
        }

        let native_image: *mut Object = msg_send![class!(NSImage), alloc];
        let native_image: *mut Object = msg_send![native_image,
            initWithCGImage: cg_image,
            size: Size {
                width: f64::from(image.width()),
                height: f64::from(image.height()),
            }
        ];
        CGImageRelease(cg_image);
        (!native_image.is_null()).then_some(native_image)
    }
}
