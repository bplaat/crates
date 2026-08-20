/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Quick Look preview extension for the image formats MacView displays.

#![allow(unsafe_code)]

mod cocoa;

use std::ptr::null_mut;

use block2::{Block, RcBlock};
use macview_appkit::{
    Media, NS_VIEW_HEIGHT_SIZABLE, NS_VIEW_WIDTH_SIZABLE, Point, Rect, Size, create_image_view,
    create_tinyvg_view, dispatch_async, dispatch_async_main, extension_main, load_media,
    make_error, ns_string, preferred_content_size,
};
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

struct PreviewCompletion(RcBlock<dyn Fn(*mut Object)>);

// SAFETY: Quick Look completion blocks are escaping blocks intended for asynchronous use. This
// wrapper moves its owned copy to the main queue and invokes it there exactly once.
unsafe impl Send for PreviewCompletion {}

impl PreviewCompletion {
    fn call(&self, error: *mut Object) {
        self.0.call((error,));
    }
}

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
        // SAFETY: Quick Look supplied live objects. Copies keep them alive until the asynchronous
        // load and main-queue UI continuation have finished.
        let (this, url, completion) = unsafe {
            let this = self as *const Self as *mut Object;
            let this: *mut Object = msg_send![this, retain];
            let url: *mut Object = msg_send![url, retain];
            let completion = PreviewCompletion(completion.copy());
            (this as usize, url as usize, completion)
        };

        dispatch_async(move || {
            // SAFETY: url is retained until this load completes.
            let media = unsafe { load_media(url as *mut Object) };
            // SAFETY: This balances the retain before dispatching the load.
            unsafe {
                let _: () = msg_send![url as *mut Object, release];
            }
            dispatch_async_main(move || {
                // SAFETY: this is retained and all AppKit view work happens on the main queue.
                unsafe {
                    let this = this as *mut Object;
                    match media {
                        Ok(media) => {
                            let size = preferred_content_size(media.size());
                            let root: *mut Object = msg_send![this, view];
                            let bounds: Rect = msg_send![root, bounds];
                            let view = create_media_view(bounds, media);
                            let _: () = msg_send![view,
                                setAutoresizingMask: NS_VIEW_WIDTH_SIZABLE
                                    | NS_VIEW_HEIGHT_SIZABLE
                            ];
                            let _: () = msg_send![root, addSubview: view];
                            let _: () = msg_send![this, setPreferredContentSize: size];
                            let _: () = msg_send![view, release];
                            completion.call(null_mut());
                        }
                        Err(description) => {
                            let error =
                                make_error(ns_string!("nl.bplaat.MacView.Preview"), &description);
                            completion.call(error);
                        }
                    }
                    let _: () = msg_send![this, release];
                }
            });
        });
    }
}

/// Creates an owned view that draws `media` inside `frame`.
///
/// The caller owns the returned view and must send it `release`.
fn create_media_view(frame: Rect, media: Media) -> *mut Object {
    match media {
        Media::TinyVg(document) => create_tinyvg_view(frame, document),
        // SAFETY: NSImageView retains the image, which stays alive until this function returns.
        Media::Image(image) => unsafe { create_image_view(frame, image.as_ptr()) },
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
    extension_main();
}
