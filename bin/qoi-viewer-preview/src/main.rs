/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Quick Look preview extension for QOI images.

#![allow(unsafe_code)]

mod cocoa;

use std::ffi::{CString, c_char};
use std::os::unix::ffi::OsStrExt;
use std::ptr::null_mut;

use block2::Block;
use cocoa::*;
use objc2::ffi::class_addProtocol;
use objc2::runtime::{AnyClass, AnyObject as Object, AnyProtocol};
use objc2::{class, define_class, msg_send};
use qoi_appkit::{Point, Rect, Size, load_image};

define_class!(
    #[unsafe(super(NSViewController))]
    #[name = "QoiPreviewViewController"]
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
        // SAFETY: The controller owns the view after setView:, and the selectors match AppKit.
        unsafe {
            let frame = Rect {
                origin: Point { x: 0.0, y: 0.0 },
                size: Size {
                    width: 640.0,
                    height: 480.0,
                },
            };
            let image_view: *mut Object = msg_send![class!(NSImageView), alloc];
            let image_view: *mut Object = msg_send![image_view, initWithFrame: frame];
            let _: () = msg_send![image_view,
                setImageScaling: NS_IMAGE_SCALE_PROPORTIONALLY_UP_OR_DOWN
            ];
            let this = self as *const Self as *mut Object;
            let _: () = msg_send![this, setView: image_view];
            let _: () = msg_send![image_view, release];
        }
    }

    fn prepare_preview(&self, url: *mut Object, completion: &Block<dyn Fn(*mut Object)>) {
        match load_image(url) {
            Ok((image, image_size)) => {
                let size = Size {
                    width: image_size.width.clamp(320.0, 1200.0),
                    height: image_size.height.clamp(240.0, 800.0),
                };
                // SAFETY: self is a live NSViewController and its view is the NSImageView
                // installed in loadView. NSImageView retains image during setImage:.
                unsafe {
                    let this = self as *const Self as *mut Object;
                    let view: *mut Object = msg_send![this, view];
                    let _: () = msg_send![view, setImage: image];
                    let _: () = msg_send![this, setPreferredContentSize: size];
                    let _: () = msg_send![image, release];
                }
                completion.call((null_mut(),));
            }
            Err(description) => completion.call((make_error(&description),)),
        }
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
            errorWithDomain: ns_string("nl.bplaat.QOIViewer.Preview"),
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
