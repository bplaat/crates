/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Shared AppKit support for MacView and its Quick Look extensions.

#![allow(unsafe_code)]

use std::ffi::{CStr, CString, OsStr, c_char, c_void};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr::null;
use std::sync::Arc;

use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, msg_send, sel};

mod cocoa;
mod tinyvg_renderer;

use cocoa::*;
pub use cocoa::{
    __CFConstantStringClassReference, CFConstString, CGContextFillRect, CGContextSetRGBFillColor,
    NS_VIEW_HEIGHT_SIZABLE, NS_VIEW_WIDTH_SIZABLE, Point, Rect, Size, ns_string,
};
pub use tinyvg_renderer::{create_tinyvg_view, fill_white_background, render_tinyvg};

/// An owned immutable `NSString` that can be transferred between queues.
pub struct OwnedString {
    string: Retained<Object>,
}

impl OwnedString {
    /// Creates a native string by copying `value` once.
    pub fn new(value: &str) -> Self {
        // SAFETY: NSString copies the valid UTF-8 bytes and returns an owned immutable object.
        let string: Retained<Object> = unsafe {
            let string: Allocated<Object> = msg_send![class!(NSString), alloc];
            msg_send![string,
                initWithBytes: value.as_ptr().cast::<c_void>(),
                length: value.len(),
                encoding: NS_UTF8_STRING_ENCODING
            ]
        };
        Self { string }
    }

    /// Returns the string pointer, which remains valid while this value is alive.
    pub const fn as_ptr(&self) -> *mut Object {
        self.string.as_ptr()
    }

    /// Retains an existing immutable string.
    ///
    /// # Safety
    ///
    /// `string` must point to a valid `NSString`.
    pub unsafe fn retain(string: *mut Object) -> Self {
        // SAFETY: The caller guarantees string is a live NSString.
        let string = unsafe { Retained::retain(string) }.expect("cannot retain a null NSString");
        Self { string }
    }
}

// SAFETY: NSString is immutable and may be transferred between threads.
unsafe impl Send for OwnedString {}
// SAFETY: NSString is immutable and may be read concurrently from multiple threads.
unsafe impl Sync for OwnedString {}

/// An owned `NSImage` that is released when it is dropped.
pub struct Image {
    image: Retained<Object>,
    size: Size,
    _backing: ImageBacking,
}

enum ImageBacking {
    None,
    Bytes { _bytes: Box<[u8]> },
    Data { _data: Retained<Object> },
}

impl Image {
    /// Returns the image, which stays alive for as long as this value does.
    pub const fn as_ptr(&self) -> *mut Object {
        self.image.as_ptr()
    }

    /// Returns the natural image size in points.
    pub const fn size(&self) -> Size {
        self.size
    }
}

// SAFETY: MacView only transfers ownership of a fully initialized image between queues. It does
// not access the image from both queues at once, and drawing remains on the queue chosen by
// AppKit or Quick Look.
unsafe impl Send for Image {}

/// An image in one of the formats the viewer and its Quick Look extensions display.
pub enum Media {
    /// An image AppKit can draw, which covers QOI, SVG and every built-in format.
    Image(Image),
    /// A parsed TinyVG document.
    TinyVg(Arc<tinyvg::Document>),
}

impl Media {
    /// Returns the natural size of the image in points.
    pub fn size(&self) -> Size {
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

/// Runs `work` asynchronously on a user-initiated global GCD queue.
pub fn dispatch_async<F>(work: F)
where
    F: FnOnce() + Send + 'static,
{
    // SAFETY: The system user-initiated global queue exists for the process lifetime.
    let queue = unsafe { dispatch_get_global_queue(QOS_CLASS_USER_INITIATED, 0) };
    dispatch_to(queue, work);
}

/// Runs `work` asynchronously on the main GCD queue.
pub fn dispatch_async_main<F>(work: F)
where
    F: FnOnce() + Send + 'static,
{
    // SAFETY: The exported system main queue exists for the process lifetime. Apple's
    // dispatch_get_main_queue is an inline C function, so bind its backing object directly.
    let queue = std::ptr::addr_of_mut!(DISPATCH_MAIN_QUEUE).cast::<c_void>();
    dispatch_to(queue, work);
}

fn dispatch_to<F>(queue: *mut c_void, work: F)
where
    F: FnOnce() + Send + 'static,
{
    extern "C" fn invoke<F>(context: *mut c_void)
    where
        F: FnOnce() + Send + 'static,
    {
        // SAFETY: dispatch_to created this allocation for this one GCD invocation.
        let work = unsafe { Box::from_raw(context.cast::<Option<F>>()) }
            .take()
            .expect("GCD work was already invoked");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            objc2::rc::autoreleasepool(|_| work());
        }));
        if result.is_err() {
            std::process::abort();
        }
    }

    let context = Box::into_raw(Box::new(Some(work))).cast();
    // SAFETY: The context stays allocated until GCD invokes the function exactly once.
    unsafe {
        dispatch_async_f(queue, context, invoke::<F>);
    }
}

const QOS_CLASS_USER_INITIATED: isize = 0x19;

#[link(name = "System")]
unsafe extern "C" {
    fn dispatch_get_global_queue(identifier: isize, flags: usize) -> *mut c_void;
    fn dispatch_async_f(queue: *mut c_void, context: *mut c_void, work: extern "C" fn(*mut c_void));
    #[link_name = "_dispatch_main_q"]
    static mut DISPATCH_MAIN_QUEUE: u8;
}

/// Loads a supported image from a file URL.
///
/// # Safety
///
/// `url` must point to a valid `NSURL` for the duration of this call.
pub unsafe fn load_media(url: *mut Object) -> Result<Media, String> {
    // SAFETY: url is valid. Quick Look passes security-scoped URLs, and ordinary URLs simply
    // return false without changing their access state.
    let scoped: Bool = unsafe { msg_send![url, startAccessingSecurityScopedResource] };
    // SAFETY: The caller supplies a live file URL.
    let path = unsafe { file_path(url) };
    let result = path
        .ok_or_else(|| String::from("Could not read the image"))
        .and_then(|path| std::fs::read(path).map_err(|_| String::from("Could not read the image")))
        .and_then(decode_media);
    if scoped.as_bool() {
        // SAFETY: This balances the successful start call above.
        unsafe {
            let _: () = msg_send![url, stopAccessingSecurityScopedResource];
        }
    }
    result
}

fn decode_media(bytes: Vec<u8>) -> Result<Media, String> {
    if tinyvg::is_tinyvg(&bytes) {
        return decode_tinyvg(&bytes).map(Arc::new).map(Media::TinyVg);
    }
    decode_image(bytes).map(Media::Image)
}

/// Returns the Rust path represented by a file URL.
///
/// # Safety
///
/// `url` must point to a valid file `NSURL` for this call.
unsafe fn file_path(url: *mut Object) -> Option<PathBuf> {
    // SAFETY: NSURL owns the representation, which is copied into the Rust path before return.
    let representation: *const c_char = unsafe { msg_send![url, fileSystemRepresentation] };
    if representation.is_null() {
        return None;
    }
    // SAFETY: NSURL returns a null-terminated file-system representation.
    let bytes = unsafe { CStr::from_ptr(representation) }.to_bytes();
    Some(PathBuf::from(OsStr::from_bytes(bytes)))
}

/// Parses a TinyVG document from bytes.
pub fn decode_tinyvg(bytes: &[u8]) -> Result<tinyvg::Document, String> {
    tinyvg::parse_auto(bytes).map_err(|error| error.to_string())
}

/// Decodes QOI or an AppKit-supported image format from an owned Rust buffer.
pub fn decode_image(bytes: Vec<u8>) -> Result<Image, String> {
    if bytes.starts_with(b"qoif") {
        let decoded = qoi::decode(&bytes).map_err(|error| error.to_string())?;
        let image =
            make_image(&decoded).ok_or_else(|| String::from("Could not create the image"))?;
        // SAFETY: make_image returned an owned, initialized NSImage.
        return unsafe { finish_image(image, ImageBacking::None) };
    }

    let bytes = bytes.into_boxed_slice();
    // SAFETY: NSData borrows the stable boxed allocation, which ImageBacking keeps alive until
    // after NSImage is released. NSData must not free Rust's allocation.
    let data: *mut Object = unsafe {
        msg_send![class!(NSData),
            dataWithBytesNoCopy: bytes.as_ptr().cast::<c_void>(),
            length: bytes.len(),
            freeWhenDone: Bool::NO
        ]
    };
    // SAFETY: data remains valid through the owned backing buffer.
    unsafe { decode_native_image(data, ImageBacking::Bytes { _bytes: bytes }) }
}

/// Decodes an AppKit-supported image while retaining its existing native data buffer.
///
/// # Safety
///
/// `data` must point to a valid `NSData` for this call.
pub unsafe fn decode_image_data(data: *mut Object) -> Result<Image, String> {
    // SAFETY: data remains valid for this call and exposes immutable bytes.
    let bytes = unsafe {
        let length: usize = msg_send![data, length];
        let bytes: *const c_void = msg_send![data, bytes];
        if length == 0 {
            &[]
        } else {
            assert!(!bytes.is_null(), "non-empty NSData returned null bytes");
            std::slice::from_raw_parts(bytes.cast::<u8>(), length)
        }
    };
    if bytes.starts_with(b"qoif") {
        let decoded = qoi::decode(bytes).map_err(|error| error.to_string())?;
        let image =
            make_image(&decoded).ok_or_else(|| String::from("Could not create the image"))?;
        // SAFETY: make_image returned an owned, initialized NSImage.
        return unsafe { finish_image(image, ImageBacking::None) };
    }
    // SAFETY: Retaining data keeps lazy AppKit access valid for the image lifetime.
    let retained = unsafe { Retained::retain(data) }.expect("cannot retain a null NSData");
    // SAFETY: data is live and retained as the image backing.
    unsafe { decode_native_image(data, ImageBacking::Data { _data: retained }) }
}

unsafe fn decode_native_image(data: *mut Object, backing: ImageBacking) -> Result<Image, String> {
    // SAFETY: data is valid and its backing outlives the returned image.
    let image: Option<Retained<Object>> = unsafe {
        let image: Allocated<Object> = msg_send![class!(NSImage), alloc];
        msg_send![image, initWithData: data]
    };
    let Some(image) = image else {
        return Err(String::from("Unsupported or invalid image"));
    };
    // SAFETY: image is owned and initialized.
    unsafe { finish_image(image, backing) }
}

unsafe fn finish_image(image: Retained<Object>, backing: ImageBacking) -> Result<Image, String> {
    // SAFETY: image is a valid, initialized NSImage.
    let size = unsafe { msg_send![&*image, size] };
    // SAFETY: image owns its representations. Asking representations that expose CGImage for it
    // realizes their existing pixel storage on this worker without creating an application-owned
    // copy. Other representation types are left lazy.
    unsafe {
        let representations: *mut Object = msg_send![&*image, representations];
        let count: usize = msg_send![representations, count];
        for index in 0..count {
            let representation: *mut Object = msg_send![representations, objectAtIndex: index];
            let exposes_cg_image: Bool =
                msg_send![representation, respondsToSelector: sel!(CGImage)];
            if exposes_cg_image.as_bool() {
                let _: *const c_void = msg_send![representation, CGImage];
            }
        }
    }
    Ok(Image {
        image,
        size,
        _backing: backing,
    })
}

/// Creates an owned `NSImageView` that scales `image` to fit `frame`.
///
/// An image that holds more than one frame, such as an animated GIF or APNG, plays instead of
/// showing its first frame. The returned view owns one retain count.
///
/// # Safety
///
/// `image` must point to a valid `NSImage` for the duration of this call.
pub unsafe fn create_image_view(frame: Rect, image: *mut Object) -> Retained<Object> {
    // SAFETY: The caller guarantees the image is valid, and NSImageView retains it.
    unsafe {
        let view: Allocated<Object> = msg_send![class!(NSImageView), alloc];
        let view: Retained<Object> = msg_send![view, initWithFrame: frame];
        let _: () = msg_send![&*view, setImage: image];
        let _: () = msg_send![&*view, setImageScaling: NS_IMAGE_SCALE_PROPORTIONALLY_UP_OR_DOWN];
        let _: () = msg_send![&*view, setAnimates: Bool::YES];
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
/// The returned object owns one retain count.
fn make_image(image: &qoi::Image) -> Option<Retained<Object>> {
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

        let native_image: Allocated<Object> = msg_send![class!(NSImage), alloc];
        let native_image: Option<Retained<Object>> = msg_send![native_image,
            initWithCGImage: cg_image,
            size: Size {
                width: f64::from(image.width()),
                height: f64::from(image.height()),
            }
        ];
        CGImageRelease(cg_image);
        native_image
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use objc2::rc::autoreleasepool;

    use super::*;

    #[test]
    fn gcd_worker_runs_dispatched_work() {
        let (sender, receiver) = mpsc::sync_channel(1);
        dispatch_async(move || sender.send(()).expect("receiver should remain alive"));
        receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("GCD work should run");
    }

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
            let image = decode_image(PNG.to_vec()).expect("PNG should be decoded by AppKit");
            assert_eq!(image.size().width, 1.0);
            assert_eq!(image.size().height, 1.0);
        });
    }

    #[test]
    fn rejects_empty_nsdata_without_dereferencing_its_null_bytes() {
        autoreleasepool(|_| {
            // SAFETY: NSData's data constructor returns a live empty data object.
            let result = unsafe {
                let data: *mut Object = msg_send![class!(NSData), data];
                decode_image_data(data)
            };
            assert!(result.is_err());
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
