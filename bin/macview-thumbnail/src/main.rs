/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Quick Look thumbnail extension for the image formats MacView displays.

#![allow(unsafe_code)]

mod cocoa;

use std::ffi::{CString, c_char, c_void};
use std::os::unix::ffi::OsStrExt;
use std::ptr::null_mut;

use block2::{Block, RcBlock};
use cocoa::*;
use macview_appkit::{Media, Point, Rect, Size, fill_white_background, load_media, render_tinyvg};
use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send};

define_class!(
    #[unsafe(super(QLThumbnailProvider))]
    #[name = "MacViewThumbnailProvider"]
    struct ThumbnailProvider;

    impl ThumbnailProvider {
        #[unsafe(method(provideThumbnailForFileRequest:completionHandler:))]
        fn _provide_thumbnail(
            &self,
            request: *mut Object,
            completion: &Block<dyn Fn(*mut Object, *mut Object)>,
        ) {
            provide_thumbnail(request, completion);
        }
    }
);

fn provide_thumbnail(request: *mut Object, completion: &Block<dyn Fn(*mut Object, *mut Object)>) {
    // SAFETY: Quick Look supplies a valid request with a file URL and maximum size.
    let (url, maximum_size, scale): (*mut Object, Size, f64) = unsafe {
        (
            msg_send![request, fileURL],
            msg_send![request, maximumSize],
            msg_send![request, scale],
        )
    };
    // SAFETY: Quick Look supplied url as a valid file NSURL for this callback.
    let media = match unsafe { load_media(url) } {
        Ok(media) => media,
        Err(description) => {
            completion.call((null_mut(), make_error(&description)));
            return;
        }
    };

    let context_size = aspect_fit(media.size(), maximum_size);
    let drawing_size = scaled(context_size, scale);
    let drawing = RcBlock::new_ret::<*mut c_void, bool>(move |context| {
        // SAFETY: Quick Look owns the drawing context for this call. The copied drawing block
        // owns the media and drawing is synchronous within the block invocation.
        unsafe {
            fill_white_background(context, drawing_size);
            match &media {
                Media::Image(image) => draw_image(context, image.as_ptr(), drawing_size),
                Media::TinyVg(document) => render_tinyvg(context, document, drawing_size, 1.0),
            }
        }
        true
    });

    // SAFETY: QLThumbnailReply copies the drawing block and invokes it while producing the reply.
    let reply: *mut Object = unsafe {
        msg_send![class!(QLThumbnailReply),
            replyWithContextSize: context_size,
            drawingBlock: &*drawing
        ]
    };
    completion.call((reply, null_mut()));
}

/// Draws an image over the whole thumbnail.
///
/// AppKit draws through the image itself instead of a fixed size bitmap, so vector formats like
/// SVG are rasterized at the size the thumbnail is actually requested in.
///
/// # Safety
///
/// `context` must be a valid `CGContext` and `image` a valid `NSImage` for this call.
unsafe fn draw_image(context: *mut c_void, image: *mut Object, size: Size) {
    // SAFETY: The caller guarantees both objects are valid, and the graphics state is restored
    // before returning.
    unsafe {
        let graphics_context: *mut Object = msg_send![class!(NSGraphicsContext),
            graphicsContextWithCGContext: context,
            flipped: Bool::NO
        ];
        let _: () = msg_send![class!(NSGraphicsContext), saveGraphicsState];
        let _: () = msg_send![class!(NSGraphicsContext), setCurrentContext: graphics_context];
        let _: () = msg_send![image,
            drawInRect: Rect {
                origin: Point { x: 0.0, y: 0.0 },
                size,
            }
        ];
        let _: () = msg_send![class!(NSGraphicsContext), restoreGraphicsState];
    }
}

/// Returns the size `image` gets when it is scaled to fit inside `bounds`.
///
/// A drawing without a size, which is what AppKit reports for an SVG that declares none, keeps
/// the requested bounds instead of scaling to nothing.
fn aspect_fit(image: Size, bounds: Size) -> Size {
    if image.width <= 0.0 || image.height <= 0.0 {
        return bounds;
    }
    let scale = (bounds.width / image.width).min(bounds.height / image.height);
    Size {
        width: image.width * scale,
        height: image.height * scale,
    }
}

fn scaled(size: Size, scale: f64) -> Size {
    Size {
        width: size.width * scale,
        height: size.height * scale,
    }
}

fn make_error(description: &str) -> *mut Object {
    // SAFETY: The Foundation convenience constructors return autoreleased objects that remain
    // valid while Quick Look receives the completion callback.
    unsafe {
        let description: *mut Object = msg_send![class!(NSString),
            stringWithUTF8String: CString::new(description)
                .expect("error description contains a null byte")
                .as_ptr()
        ];
        let description_key: *mut Object = msg_send![class!(NSString),
            stringWithUTF8String: c"NSLocalizedDescription".as_ptr()
        ];
        let user_info: *mut Object = msg_send![class!(NSDictionary),
            dictionaryWithObject: description,
            forKey: description_key
        ];
        let domain: *mut Object = msg_send![class!(NSString),
            stringWithUTF8String: c"nl.bplaat.MacView.Thumbnail".as_ptr()
        ];
        msg_send![class!(NSError),
            errorWithDomain: domain,
            code: 1isize,
            userInfo: user_info
        ]
    }
}

fn main() {
    let _ = ThumbnailProvider::class();
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
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use objc2::rc::autoreleasepool;

    use super::*;

    #[test]
    fn thumbnail_context_uses_the_image_aspect_ratio() {
        let size = aspect_fit(
            Size {
                width: 400.0,
                height: 200.0,
            },
            Size {
                width: 100.0,
                height: 100.0,
            },
        );
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 50.0);
    }

    #[test]
    fn media_without_a_size_keeps_the_requested_bounds() {
        let bounds = Size {
            width: 128.0,
            height: 96.0,
        };
        let size = aspect_fit(
            Size {
                width: 0.0,
                height: 0.0,
            },
            bounds,
        );
        assert_eq!(size.width, bounds.width);
        assert_eq!(size.height, bounds.height);
    }

    #[test]
    fn image_fills_a_retina_context() {
        let context_size = aspect_fit(
            Size {
                width: 400.0,
                height: 200.0,
            },
            Size {
                width: 256.0,
                height: 256.0,
            },
        );
        let drawing_size = scaled(context_size, 2.0);
        assert_eq!(drawing_size.width, 512.0);
        assert_eq!(drawing_size.height, 256.0);
    }

    #[test]
    fn thumbnail_reply_keeps_its_drawing_block_alive() {
        let dropped = Arc::new(AtomicBool::new(false));
        struct DropGuard(Arc<AtomicBool>);
        impl Drop for DropGuard {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        autoreleasepool(|_| {
            let guard = DropGuard(dropped.clone());
            let drawing = RcBlock::new_ret::<*mut c_void, bool>(move |_| {
                let _ = &guard;
                true
            });
            // SAFETY: The block has the CGContext drawing signature required by QLThumbnailReply.
            let reply: *mut Object = unsafe {
                msg_send![class!(QLThumbnailReply),
                    replyWithContextSize: Size {
                        width: 32.0,
                        height: 32.0,
                    },
                    drawingBlock: &*drawing
                ]
            };
            assert!(!reply.is_null());
            drop(drawing);
            assert!(
                !dropped.load(Ordering::SeqCst),
                "QLThumbnailReply must retain its escaping drawing block"
            );
        });
        assert!(dropped.load(Ordering::SeqCst));
    }
}
