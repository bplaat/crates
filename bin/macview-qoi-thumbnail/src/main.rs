/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Quick Look thumbnail extension for QOI images.

#![allow(unsafe_code)]

mod cocoa;

use std::ffi::{CString, c_char, c_void};
use std::os::unix::ffi::OsStrExt;
use std::ptr::null_mut;

use block2::{Block, RcBlock};
use cocoa::*;
use macview_appkit::{Point, Rect, Size, load_image};
use objc2::runtime::AnyObject as Object;
use objc2::{class, define_class, msg_send};

define_class!(
    #[unsafe(super(QLThumbnailProvider))]
    #[name = "MacViewQoiThumbnailProvider"]
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

struct OwnedImage(*mut Object);

impl OwnedImage {
    const fn as_ptr(&self) -> *mut Object {
        self.0
    }
}

impl Drop for OwnedImage {
    fn drop(&mut self) {
        // SAFETY: OwnedImage contains the non-null, retained NSImage returned by load_image.
        unsafe {
            let _: () = msg_send![self.0, release];
        }
    }
}

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
    let (image, image_size) = match unsafe { load_image(url) } {
        Ok((image, image_size)) => (OwnedImage(image), image_size),
        Err(description) => {
            completion.call((null_mut(), make_error(&description)));
            return;
        }
    };

    let context_size = aspect_fit(image_size, maximum_size);
    let drawing_size = scaled(context_size, scale);
    let drawing = RcBlock::new_ret::<*mut c_void, bool>(move |context| {
        let rectangle = Rect {
            origin: Point { x: 0.0, y: 0.0 },
            size: drawing_size,
        };
        // SAFETY: Quick Look owns the drawing context for this call. The copied drawing block
        // owns image, and the returned CGImage remains valid for the duration of the draw.
        unsafe {
            let cg_image: *const c_void = msg_send![image.as_ptr(),
                CGImageForProposedRect: null_mut::<Rect>().cast::<c_void>(),
                context: null_mut::<Object>(),
                hints: null_mut::<Object>()
            ];
            if cg_image.is_null() {
                return false;
            }
            CGContextDrawImage(context, rectangle, cg_image);
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

fn aspect_fit(image: Size, bounds: Size) -> Size {
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
    fn image_fills_a_retina_context() {
        let maximum_size = Size {
            width: 256.0,
            height: 256.0,
        };
        let scale = 2.0;
        let context_size = aspect_fit(
            Size {
                width: 400.0,
                height: 200.0,
            },
            maximum_size,
        );
        let drawing_size = scaled(context_size, scale);
        assert_eq!(drawing_size.width, 512.0);
        assert_eq!(drawing_size.height, 256.0);
    }

    #[test]
    fn thumbnail_reply_is_an_object() {
        autoreleasepool(|_| {
            let drawing = RcBlock::new_ret::<*mut c_void, bool>(|_| true);
            // SAFETY: The block has the CGContext drawing signature required by QLThumbnailReply.
            unsafe {
                let reply: *mut Object = msg_send![class!(QLThumbnailReply),
                    replyWithContextSize: Size {
                        width: 32.0,
                        height: 32.0,
                    },
                    drawingBlock: &*drawing
                ];
                assert!(!reply.is_null());
                let _: *mut Object = msg_send![reply, retain];
                let _: () = msg_send![reply, release];
            }
        });
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
