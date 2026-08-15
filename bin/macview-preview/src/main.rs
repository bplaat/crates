/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Quick Look preview extension for the image formats MacView displays.

#![allow(unsafe_code)]

mod cocoa;

use std::ffi::{CString, c_char};
use std::os::unix::ffi::OsStrExt;
use std::ptr::null_mut;

use block2::Block;
use cocoa::*;
use macview_appkit::{Media, Point, Rect, Size, create_tinyvg_view, load_media};
use objc2::ffi::class_addProtocol;
use objc2::runtime::{AnyClass, AnyObject as Object, AnyProtocol};
use objc2::{class, define_class, msg_send};

define_class!(
    #[unsafe(super(NSViewController))]
    #[name = "MacViewPreviewViewController"]
    struct PreviewViewController;

    impl PreviewViewController {
        #[unsafe(method(loadView))]
        fn _load_view(&self) {
            self.load_view();
        }

        #[unsafe(method(preparePreviewOfFileAtURL:completionHandler:))]
        fn _prepare_preview(
            &self,
            url: *mut Object,
            completion: &Block<dyn Fn(*mut Object)>,
        ) {
            self.prepare_preview(url, completion);
        }
    }
);

impl PreviewViewController {
    fn load_view(&self) {
        // SAFETY: The controller owns the placeholder view after setView:.
        unsafe {
            let view: *mut Object = msg_send![class!(NSView), alloc];
            let view: *mut Object = msg_send![view,
                initWithFrame: Rect {
                    origin: Point { x: 0.0, y: 0.0 },
                    size: Size { width: 640.0, height: 480.0 },
                }
            ];
            let this = self as *const Self as *mut Object;
            let _: () = msg_send![this, setView: view];
            let _: () = msg_send![view, release];
        }
    }

    fn prepare_preview(&self, url: *mut Object, completion: &Block<dyn Fn(*mut Object)>) {
        // SAFETY: Quick Look supplied url as a valid file NSURL for this callback.
        let media = match unsafe { load_media(url) } {
            Ok(media) => media,
            Err(description) => {
                completion.call((make_error(&description),));
                return;
            }
        };

        let media_size = media.size();
        let size = Size {
            width: media_size.width.clamp(320.0, 1200.0),
            height: media_size.height.clamp(240.0, 800.0),
        };
        // SAFETY: self and its root view are live. Keeping the loadView root in place is required
        // because replacing it after ViewBridge connects tears down the service.
        unsafe {
            let this = self as *const Self as *mut Object;
            let root: *mut Object = msg_send![this, view];
            let bounds: Rect = msg_send![root, bounds];
            let view = create_media_view(bounds, media);
            let _: () = msg_send![view,
                setAutoresizingMask: NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE
            ];
            let _: () = msg_send![root, addSubview: view];
            let _: () = msg_send![this, setPreferredContentSize: size];
            let _: () = msg_send![view, release];
        }
        completion.call((null_mut(),));
    }
}

/// Creates an owned view that draws `media` inside `frame`.
///
/// The caller owns the returned view and must send it `release`.
fn create_media_view(frame: Rect, media: Media) -> *mut Object {
    match media {
        Media::TinyVg(document) => create_tinyvg_view(frame, Box::new(document)),
        // SAFETY: NSImageView retains the image, which stays alive until this function returns.
        Media::Image(image) => unsafe {
            let view: *mut Object = msg_send![class!(NSImageView), alloc];
            let view: *mut Object = msg_send![view, initWithFrame: frame];
            let _: () = msg_send![view, setImage: image.as_ptr()];
            let _: () = msg_send![view,
                setImageScaling: NS_IMAGE_SCALE_PROPORTIONALLY_UP_OR_DOWN
            ];
            view
        },
    }
}

fn make_error(description: &str) -> *mut Object {
    // SAFETY: The Foundation convenience constructors return autoreleased objects that remain
    // valid while Quick Look receives the completion callback.
    unsafe {
        let description_key = ns_string("NSLocalizedDescription");
        let user_info: *mut Object = msg_send![class!(NSDictionary),
            dictionaryWithObject: ns_string(description),
            forKey: description_key
        ];
        msg_send![class!(NSError),
            errorWithDomain: ns_string("nl.bplaat.MacView.Preview"),
            code: 1isize,
            userInfo: user_info
        ]
    }
}

fn main() {
    let preview_class = PreviewViewController::class();
    // SAFETY: QuickLookUI provides the process-lifetime QLPreviewingController protocol, and
    // preview_class is the registered Objective-C class used as the extension principal class.
    unsafe {
        let protocol = AnyProtocol::get(c"QLPreviewingController")
            .expect("QLPreviewingController is unavailable");
        assert!(
            class_addProtocol(preview_class.cast::<AnyClass>(), protocol).as_bool(),
            "failed to adopt QLPreviewingController"
        );
    }
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
