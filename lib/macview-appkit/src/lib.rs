/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Shared AppKit image loading for MacView and its Quick Look extensions.

#![allow(unsafe_code)]

use std::ptr::null;

use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, msg_send};

mod cocoa;
mod tinyvg_renderer;

use cocoa::*;
pub use cocoa::{Point, Rect, Size};
pub use tinyvg_renderer::{create_tinyvg_view, fill_white_background, render_tinyvg};

/// Loads an image from a file URL into an owned `NSImage`.
///
/// The caller owns the returned object and must send it `release`.
///
/// # Safety
///
/// `url` must point to a valid `NSURL` for the duration of this call.
pub unsafe fn load_image(url: *mut Object) -> Result<(*mut Object, Size), String> {
    // SAFETY: The caller supplies a valid file URL and decode_image consumes data synchronously.
    unsafe { load_file(url, "Could not read the image", decode_image) }
}

/// Loads and parses a TinyVG document from a file URL.
///
/// # Safety
///
/// `url` must point to a valid `NSURL` for the duration of this call.
pub unsafe fn load_tinyvg(url: *mut Object) -> Result<tinyvg::Document, String> {
    // SAFETY: The caller supplies a valid file URL and decode_tinyvg consumes data synchronously.
    unsafe { load_file(url, "Could not read the TinyVG image", decode_tinyvg) }
}

unsafe fn load_file<T>(
    url: *mut Object,
    read_error: &str,
    decode: unsafe fn(*mut Object) -> Result<T, String>,
) -> Result<T, String> {
    // SAFETY: url is valid. Quick Look passes security-scoped URLs, and ordinary URLs simply
    // return false without changing their access state.
    let scoped: Bool = unsafe { msg_send![url, startAccessingSecurityScopedResource] };
    // SAFETY: url is valid and NSData reads the file synchronously.
    let data: *mut Object = unsafe { msg_send![class!(NSData), dataWithContentsOfURL: url] };
    let result = if data.is_null() {
        Err(String::from(read_error))
    } else {
        // SAFETY: data is a live NSData for the duration of this call.
        unsafe { decode(data) }
    };
    if scoped.as_bool() {
        // SAFETY: This balances the successful start call above.
        unsafe {
            let _: () = msg_send![url, stopAccessingSecurityScopedResource];
        }
    }
    result
}

/// Parses a TinyVG document from an `NSData` object.
///
/// # Safety
///
/// `data` must point to a valid `NSData` for the duration of this call.
pub unsafe fn decode_tinyvg(data: *mut Object) -> Result<tinyvg::Document, String> {
    // SAFETY: NSData keeps its immutable byte buffer alive for the duration of this function.
    let bytes = unsafe {
        let length: usize = msg_send![data, length];
        let bytes: *const std::ffi::c_void = msg_send![data, bytes];
        std::slice::from_raw_parts(bytes.cast::<u8>(), length)
    };
    tinyvg::parse_auto(bytes).map_err(|error| error.to_string())
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
