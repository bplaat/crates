/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Shared AppKit image loading for MacView and its Quick Look extensions.

#![allow(unsafe_code)]

use std::ptr::null;

use objc2::runtime::AnyObject as Object;
use objc2::{class, msg_send};

mod cocoa;

use cocoa::*;
pub use cocoa::{Point, Rect, Size};

/// Loads an image from a file URL into an owned `NSImage`.
///
/// The caller owns the returned object and must send it `release`.
///
/// # Safety
///
/// `url` must point to a valid `NSURL` for the duration of this call.
pub unsafe fn load_image(url: *mut Object) -> Result<(*mut Object, Size), String> {
    // SAFETY: The caller supplies a valid file URL. NSData owns its bytes for this scope.
    let data: *mut Object = unsafe { msg_send![class!(NSData), dataWithContentsOfURL: url] };
    if data.is_null() {
        return Err(String::from("Could not read the image"));
    }
    // SAFETY: data is the valid NSData object returned above and lives for this call.
    unsafe { decode_image(data) }
}

/// Decodes QOI or an AppKit-supported image format into an owned `NSImage`.
///
/// The caller owns the returned object and must send it `release`.
///
/// # Safety
///
/// `data` must point to a valid `NSData` for the duration of this call.
pub unsafe fn decode_image(data: *mut Object) -> Result<(*mut Object, Size), String> {
    // SAFETY: NSData keeps its immutable byte buffer alive for the duration of this function.
    let bytes = unsafe {
        let length: usize = msg_send![data, length];
        let bytes: *const std::ffi::c_void = msg_send![data, bytes];
        std::slice::from_raw_parts(bytes.cast::<u8>(), length)
    };

    let image = if bytes.starts_with(b"qoif") {
        let decoded = qoi::decode(bytes).map_err(|error| error.to_string())?;
        make_image(&decoded).ok_or_else(|| String::from("Could not create the image"))?
    } else {
        // SAFETY: data is a valid NSData instance. NSImage's initializer either returns an
        // owned image or null when AppKit does not support the contents.
        let image: *mut Object = unsafe {
            let image: *mut Object = msg_send![class!(NSImage), alloc];
            msg_send![image, initWithData: data]
        };
        if image.is_null() {
            return Err(String::from("Unsupported or invalid image"));
        }
        image
    };
    // SAFETY: image is a valid, initialized NSImage.
    let size = unsafe { msg_send![image, size] };
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

#[cfg(test)]
mod tests {
    use objc2::rc::autoreleasepool;

    use super::*;

    #[test]
    fn decodes_an_appkit_image_format() {
        const PNG: &[u8] = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x04, 0x00, 0x00,
            0x00, 0xb5, 0x1c, 0x0c, 0x02, 0x00, 0x00, 0x00, 0x0b, 0x49, 0x44, 0x41, 0x54, 0x78,
            0xda, 0x63, 0x64, 0xf8, 0x0f, 0x00, 0x01, 0x05, 0x01, 0x01, 0x27, 0x18, 0xe3, 0x66,
            0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];

        autoreleasepool(|_| {
            // SAFETY: NSData copies the bytes and remains alive while decode_image uses it.
            let data: *mut Object = unsafe {
                msg_send![class!(NSData),
                    dataWithBytes: PNG.as_ptr().cast::<std::ffi::c_void>(),
                    length: PNG.len()
                ]
            };
            // SAFETY: data is a valid NSData that lives for the duration of this call.
            let (image, size) =
                unsafe { decode_image(data) }.expect("PNG should be decoded by AppKit");
            assert_eq!(size.width, 1.0);
            assert_eq!(size.height, 1.0);
            // SAFETY: decode_image returns an owned NSImage.
            unsafe {
                let _: () = msg_send![image, release];
            }
        });
    }
}
