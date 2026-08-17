/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Shared AppKit support for MacView and its Quick Look extensions.

#![allow(unsafe_code)]

use std::ffi::{CString, c_char};
use std::os::unix::ffi::OsStrExt;
use std::ptr::null;

use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, msg_send};

mod cocoa;
mod tinyvg_renderer;

use cocoa::*;
pub use cocoa::{
    __CFConstantStringClassReference, CFConstString, CGContextFillRect, CGContextSetRGBFillColor,
    NS_VIEW_HEIGHT_SIZABLE, NS_VIEW_WIDTH_SIZABLE, Point, Rect, Size, ns_string,
};
pub use tinyvg_renderer::{create_tinyvg_view, fill_white_background, render_tinyvg};

/// An owned `NSImage` that is released when it is dropped.
pub struct Image {
    image: *mut Object,
    size: Size,
}

impl Image {
    /// Returns the image, which stays alive for as long as this value does.
    pub const fn as_ptr(&self) -> *mut Object {
        self.image
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        // SAFETY: The value owns the retained NSImage returned by decode_image.
        unsafe {
            let _: () = msg_send![self.image, release];
        }
    }
}

/// An image in one of the formats the viewer and its Quick Look extensions display.
pub enum Media {
    /// An image AppKit can draw, which covers QOI, SVG and every built-in format.
    Image(Image),
    /// A parsed TinyVG document.
    TinyVg(tinyvg::Document),
}

impl Media {
    /// Returns the natural size of the image in points.
    pub const fn size(&self) -> Size {
        match self {
            Self::Image(image) => image.size,
            Self::TinyVg(document) => Size {
                width: document.size.width,
                height: document.size.height,
            },
        }
    }
}

/// The size media is shown at, which is the media itself when it lies between these bounds.
const MINIMUM_CONTENT_SIZE: Size = Size {
    width: 320.0,
    height: 240.0,
};
const MAXIMUM_CONTENT_SIZE: Size = Size {
    width: 1200.0,
    height: 800.0,
};

/// Returns the size of the window or preview panel that media is shown in.
///
/// Media larger than the bounds is shrunk with its shape kept, so that what shows it has the shape
/// of the media and the media fills it. Clamping the width and the height on their own would give
/// some other shape, which leaves a band of background along one pair of edges.
pub fn preferred_content_size(media_size: Size) -> Size {
    let scale = (MAXIMUM_CONTENT_SIZE.width / media_size.width)
        .min(MAXIMUM_CONTENT_SIZE.height / media_size.height)
        .min(1.0);
    Size {
        width: (media_size.width * scale).max(MINIMUM_CONTENT_SIZE.width),
        height: (media_size.height * scale).max(MINIMUM_CONTENT_SIZE.height),
    }
}

/// Loads a supported image from a file URL.
///
/// # Safety
///
/// `url` must point to a valid `NSURL` for the duration of this call.
pub unsafe fn load_media(url: *mut Object) -> Result<Media, String> {
    // SAFETY: The caller supplies a valid file URL and decoding consumes data synchronously.
    unsafe { load_file(url, "Could not read the image", decode_media) }
}

/// Decodes a supported image from an `NSData` object.
///
/// # Safety
///
/// `data` must point to a valid `NSData` for the duration of this call.
unsafe fn decode_media(data: *mut Object) -> Result<Media, String> {
    // SAFETY: NSData keeps its immutable byte buffer alive for the duration of this function.
    let bytes = unsafe {
        let length: usize = msg_send![data, length];
        let bytes: *const std::ffi::c_void = msg_send![data, bytes];
        std::slice::from_raw_parts(bytes.cast::<u8>(), length)
    };
    if tinyvg::is_tinyvg(bytes) {
        // SAFETY: data is a live NSData for the duration of this call.
        return unsafe { decode_tinyvg(data) }.map(Media::TinyVg);
    }
    // SAFETY: data is a live NSData for the duration of this call.
    let (image, size) = unsafe { decode_image(data) }?;
    Ok(Media::Image(Image { image, size }))
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

/// Creates an owned `NSImageView` that scales `image` to fit `frame`.
///
/// An image that holds more than one frame, such as an animated GIF or APNG, plays instead of
/// showing its first frame. The caller owns the returned view and must send it `release`.
///
/// # Safety
///
/// `image` must point to a valid `NSImage` for the duration of this call.
pub unsafe fn create_image_view(frame: Rect, image: *mut Object) -> *mut Object {
    // SAFETY: The caller guarantees the image is valid, and NSImageView retains it.
    unsafe {
        let view: *mut Object = msg_send![class!(NSImageView), alloc];
        let view: *mut Object = msg_send![view, initWithFrame: frame];
        let _: () = msg_send![view, setImage: image];
        let _: () = msg_send![view, setImageScaling: NS_IMAGE_SCALE_PROPORTIONALLY_UP_OR_DOWN];
        let _: () = msg_send![view, setAnimates: Bool::YES];
        view
    }
}

/// Creates an autoreleased `NSError` that carries `description` as its localized description.
///
/// # Safety
///
/// `domain` must point to a valid `NSString`, which [`ns_string!`] guarantees.
pub unsafe fn make_error(domain: *mut Object, description: &str) -> *mut Object {
    // SAFETY: The Foundation convenience constructors return autoreleased objects, and the caller
    // guarantees the domain is a valid string.
    unsafe {
        let user_info: *mut Object = msg_send![class!(NSDictionary),
            dictionaryWithObject: ns_string(description),
            forKey: ns_string!("NSLocalizedDescription")
        ];
        msg_send![class!(NSError),
            errorWithDomain: domain,
            code: 1isize,
            userInfo: user_info
        ]
    }
}

/// Hands the process over to the app extension entry point, which does not return.
pub fn extension_main() -> ! {
    let arguments: Vec<CString> = std::env::args_os()
        .map(|argument| {
            CString::new(argument.as_os_str().as_bytes())
                .expect("process argument contains a null byte")
        })
        .collect();
    let argument_pointers: Vec<*const c_char> =
        arguments.iter().map(|argument| argument.as_ptr()).collect();
    // SAFETY: The argument pointers stay alive for the non-returning extension entry point.
    unsafe {
        NSExtensionMain(argument_pointers.len() as i32, argument_pointers.as_ptr());
    }
    unreachable!("NSExtensionMain does not return");
}

/// Creates an owned `NSImage` from a decoded QOI image.
///
/// The caller owns the returned object and must send it `release`.
fn make_image(image: &qoi::Image) -> Option<*mut Object> {
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

    /// Returns the magnification that shows all of the media inside a window of `content`.
    fn fit(media: Size, content: Size) -> f64 {
        (content.width / media.width).min(content.height / media.height)
    }

    #[test]
    fn a_window_opens_on_the_media_at_its_own_size() {
        let media = Size {
            width: 448.0,
            height: 220.0,
        };
        let content = preferred_content_size(media);
        assert_eq!(content.width, 448.0);
        // A window is never smaller than the size every window has room for.
        assert_eq!(content.height, MINIMUM_CONTENT_SIZE.height);
        assert_eq!(fit(media, content), 1.0);
    }

    #[test]
    fn a_window_of_shrunk_media_keeps_the_shape_of_the_media() {
        // Media that is taller than the bounds allow used to open in a window as wide as itself,
        // which left a band of background along the sides.
        for media in [
            Size {
                width: 1152.0,
                height: 858.0,
            },
            Size {
                width: 2000.0,
                height: 1500.0,
            },
        ] {
            let content = preferred_content_size(media);
            assert!(content.width <= MAXIMUM_CONTENT_SIZE.width);
            assert!(content.height <= MAXIMUM_CONTENT_SIZE.height);
            // The media fills the window it opens in, in both directions.
            let magnification = fit(media, content);
            assert!((media.width * magnification - content.width).abs() < 0.001);
            assert!((media.height * magnification - content.height).abs() < 0.001);
        }
    }

    #[test]
    fn media_that_is_shrunk_below_the_smallest_window_keeps_a_band_of_background() {
        // A panorama is 1200 points wide long before it is 240 points tall, and a window this
        // short is one no window gets, so the background shows above and below it.
        let content = preferred_content_size(Size {
            width: 4000.0,
            height: 500.0,
        });
        assert_eq!(content.width, MAXIMUM_CONTENT_SIZE.width);
        assert_eq!(content.height, MINIMUM_CONTENT_SIZE.height);
    }

    #[test]
    fn a_window_of_small_media_is_the_smallest_a_window_gets() {
        let content = preferred_content_size(Size {
            width: 24.0,
            height: 24.0,
        });
        assert_eq!(content.width, MINIMUM_CONTENT_SIZE.width);
        assert_eq!(content.height, MINIMUM_CONTENT_SIZE.height);
    }
}
