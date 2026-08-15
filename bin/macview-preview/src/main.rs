/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Quick Look preview extension for the image formats MacView displays.

#![allow(unsafe_code)]

mod cocoa;

use std::ptr::null_mut;

use block2::Block;
use macview_appkit::{
    Media, NS_VIEW_HEIGHT_SIZABLE, NS_VIEW_WIDTH_SIZABLE, Point, Rect, Size, create_image_view,
    create_tinyvg_view, extension_main, load_media, make_error, ns_string,
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
        let domain = ns_string!("nl.bplaat.MacView.Preview");
        // SAFETY: Quick Look supplied url as a valid file NSURL for this callback.
        let media = match unsafe { load_media(url) } {
            Ok(media) => media,
            Err(description) => {
                // SAFETY: The error domain is a constant string.
                let error = unsafe { make_error(domain, &description) };
                completion.call((error,));
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

/// The margin the media is drawn inside.
const MARGIN: f64 = 16.0;

/// Creates an owned view that draws `media` inside `frame`.
///
/// The caller owns the returned view and must send it `release`.
fn create_media_view(frame: Rect, media: Media) -> *mut Object {
    match media {
        Media::TinyVg(document) => create_tinyvg_view(frame, Box::new(document), MARGIN),
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
