/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! BImg Quick Look thumbnail extension - provides Finder thumbnails for QOI files.

#![no_main]
#![allow(unsafe_code)]
#![cfg(target_os = "macos")]

use std::ffi::{c_char, c_void};
use std::ptr::null_mut;

use block2::{Block, RcBlock};
use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, define_class, extern_class, msg_send};

use bimg::{NSPoint, NSRect, NSSize, create_nsimage};

// MARK: Frameworks

#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    fn NSExtensionMain(argc: i32, argv: *const *const c_char) -> i32;
}

#[link(name = "QuickLookThumbnailing", kind = "framework")]
unsafe extern "C" {}

// MARK: Extern class declarations

extern_class!(
    #[unsafe(super(NSObject))]
    #[name = "QLThumbnailProvider"]
    struct QLThumbnailProvider;
);

// MARK: ThumbnailProvider

define_class!(
    #[unsafe(super(QLThumbnailProvider))]
    #[name = "ThumbnailProvider"]
    struct ThumbnailProvider;

    impl ThumbnailProvider {
        // QLThumbnailProvider override: decode the QOI file and return a drawing-block reply.
        #[unsafe(method(provideThumbnailForFileRequest:completionHandler:))]
        fn _provide_thumbnail(
            &self,
            request: *mut Object,
            handler: &Block<dyn Fn(*mut Object, *mut Object)>,
        ) {
            self.provide_thumbnail(request, handler);
        }
    }
);

impl ThumbnailProvider {
    fn provide_thumbnail(
        &self,
        request: *mut Object,
        handler: &Block<dyn Fn(*mut Object, *mut Object)>,
    ) {
        // SAFETY: request is a valid QLFileThumbnailRequest; fileURL/maximumSize are safe accessors.
        let (url, max_size) = unsafe {
            let url: *mut Object = msg_send![request, fileURL];
            let max_size: NSSize = msg_send![request, maximumSize];
            (url, max_size)
        };

        // SAFETY: NSData dataWithContentsOfURL: is a standard Foundation method.
        let data: *mut Object =
            unsafe { msg_send![class!(NSData), dataWithContentsOfURL: url] };
        if data.is_null() {
            handler.call((null_mut(), null_mut()));
            return;
        }

        // SAFETY: bytes/length are standard NSData accessors returning valid C data.
        let (pixels, width, height) = unsafe {
            let bytes: *const c_void = msg_send![data, bytes];
            let length: usize = msg_send![data, length];
            let slice = std::slice::from_raw_parts(bytes as *const u8, length);
            match qoi::decode_to_vec(slice) {
                Err(_) => {
                    handler.call((null_mut(), null_mut()));
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

        // Compute a proportional context size that fits within maximumSize.
        let scale = (max_size.width / width as f64).min(max_size.height / height as f64);
        let context_size = NSSize::new(width as f64 * scale, height as f64 * scale);

        // Build the drawing block: draws the decoded image into the QL-provided Core Graphics context.
        let drawing_block = RcBlock::new0_ret::<Bool>(move || {
            // SAFETY: ns_image is valid until we release it below; drawInRect: is a standard
            // NSImage method that draws into the current NSGraphicsContext.
            unsafe {
                let dest_rect = NSRect::new(NSPoint::new(0.0, 0.0), context_size);
                let _: () = msg_send![ns_image, drawInRect: dest_rect];
            }
            Bool::YES
        });

        // SAFETY: QLThumbnailReply is available from QuickLookThumbnailing.framework (macOS 10.15+).
        // replyWithContextSize:currentContextDrawingBlock: is the standard synchronous thumbnail API.
        let reply: *mut Object = unsafe {
            msg_send![class!(QLThumbnailReply),
                replyWithContextSize: context_size,
                currentContextDrawingBlock: &*drawing_block
            ]
        };

        // The drawing block has been passed to the reply; QL calls it synchronously before the
        // handler returns, so ns_image is still valid when the block executes.
        // SAFETY: ns_image is a valid +1-retained NSImage returned by create_nsimage; release
        // balances the +1 after passing the image to the drawing block.
        unsafe { let _: () = msg_send![ns_image, release]; }

        handler.call((reply, null_mut()));
    }
}

// MARK: Main

#[unsafe(no_mangle)]
extern "C" fn main(argc: i32, argv: *const *const c_char) -> i32 {
    // Pre-register the custom ObjC class before NSExtensionMain starts the extension runtime.
    let _ = ThumbnailProvider::class();
    // SAFETY: NSExtensionMain is the standard app extension entry point provided by Foundation.
    unsafe { NSExtensionMain(argc, argv) }
}
