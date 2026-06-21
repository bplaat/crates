/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! BImg Quick Look preview extension - renders QOI images in the Quick Look panel.

#![no_main]
#![allow(unsafe_code)]
#![cfg(target_os = "macos")]

use std::cell::Cell;
use std::ffi::{c_char, c_void};
use std::ptr::null_mut;

use block2::Block;
use objc2::runtime::AnyObject as Object;
use objc2::{class, define_class, extern_class, msg_send};

use bimg::{NSPoint, NSRect, NSSize, create_nsimage};

// MARK: Frameworks

#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    fn NSExtensionMain(argc: i32, argv: *const *const c_char) -> i32;
}

#[link(name = "QuickLookUI", kind = "framework")]
unsafe extern "C" {}

// MARK: Constants

const NS_IMAGE_SCALING_PROPORTIONALLY_UP_OR_DOWN: u64 = 3;
const NS_VIEW_WIDTH_SIZABLE: u64 = 2;
const NS_VIEW_HEIGHT_SIZABLE: u64 = 16;

// MARK: Extern class declarations

extern_class!(
    #[unsafe(super(NSObject))]
    #[name = "NSViewController"]
    struct NSViewController;
);

// MARK: PreviewViewController

struct PreviewViewControllerIvars {
    image_view: Cell<*mut Object>,
}

define_class!(
    #[unsafe(super(NSViewController))]
    #[name = "PreviewViewController"]
    #[ivars = PreviewViewControllerIvars]
    struct PreviewViewController;

    impl PreviewViewController {
        #[unsafe(method(loadView))]
        fn _load_view(&self) {
            self.load_view();
        }

        // QLPreviewingController protocol method: load the QOI file and display it.
        #[unsafe(method(preparePreviewOfFileAtURL:completionHandler:))]
        fn _prepare_preview(
            &self,
            url: *mut Object,
            handler: &Block<dyn Fn(*mut Object)>,
        ) {
            self.prepare_preview(url, handler);
        }
    }
);

impl PreviewViewController {
    fn load_view(&self) {
        // SAFETY: NSImageView is a valid AppKit class; initWithFrame: is the designated initializer.
        // setView: transfers ownership of the view to the view controller.
        unsafe {
            let zero_frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1.0, 1.0));
            let image_view: *mut Object = msg_send![class!(NSImageView), alloc];
            let image_view: *mut Object = msg_send![image_view, initWithFrame: zero_frame];
            let _: () = msg_send![image_view,
                setImageScaling: NS_IMAGE_SCALING_PROPORTIONALLY_UP_OR_DOWN
            ];
            let _: () = msg_send![image_view,
                setAutoresizingMask: NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE
            ];
            self.ivars().image_view.set(image_view);
            let this = self as *const Self as *mut Object;
            let _: () = msg_send![this, setView: image_view];
            let _: () = msg_send![image_view, release];
        }
    }

    fn prepare_preview(&self, url: *mut Object, handler: &Block<dyn Fn(*mut Object)>) {
        let image_view = self.ivars().image_view.get();
        if image_view.is_null() {
            handler.call((null_mut(),));
            return;
        }

        // SAFETY: NSData is a valid Foundation class; dataWithContentsOfURL: is a standard API.
        let data: *mut Object = unsafe { msg_send![class!(NSData), dataWithContentsOfURL: url] };
        if data.is_null() {
            handler.call((null_mut(),));
            return;
        }

        // SAFETY: bytes/length are standard NSData accessors that return valid C data.
        let (pixels, width, height) = unsafe {
            let bytes: *const c_void = msg_send![data, bytes];
            let length: usize = msg_send![data, length];
            let slice = std::slice::from_raw_parts(bytes as *const u8, length);
            match qoi::decode_to_vec(slice) {
                Err(_) => {
                    handler.call((null_mut(),));
                    return;
                }
                Ok((header, raw)) => {
                    let px = if header.channels == qoi::Channels::Rgb {
                        raw.chunks_exact(3)
                            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                            .collect::<Vec<_>>()
                    } else {
                        raw
                    };
                    (px, header.width, header.height)
                }
            }
        };

        let ns_image = create_nsimage(&pixels, width, height);

        // SAFETY: image_view is the NSImageView created in load_view; setImage: is a standard
        // property setter that retains the image. setPreferredContentSize: is on NSViewController.
        unsafe {
            let _: () = msg_send![image_view, setImage: ns_image];
            let _: () = msg_send![ns_image, release];

            let this = self as *const Self as *mut Object;
            let _: () = msg_send![this,
                setPreferredContentSize: NSSize::new(width as f64, height as f64)
            ];
        }

        handler.call((null_mut(),));
    }
}

// MARK: Main

#[unsafe(no_mangle)]
extern "C" fn main(argc: i32, argv: *const *const c_char) -> i32 {
    // Pre-register the custom ObjC class before NSExtensionMain starts the extension runtime.
    let _ = PreviewViewController::class();
    // SAFETY: NSExtensionMain is the standard app extension entry point provided by Foundation.
    unsafe { NSExtensionMain(argc, argv) }
}
