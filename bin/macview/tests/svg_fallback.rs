/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Main-thread WebKit failure test using an AppKit-readable TIFF as SVG input.

#![allow(unsafe_code)]
// This standalone harness includes the production module directly, so items used only by the app
// or by the module's libtest tests are intentionally unused here.
#[allow(dead_code, unused_imports)]
#[path = "../src/svg.rs"]
mod svg;

use std::time::{Duration, Instant};

use macview_appkit::{Point, Rect, Size, ns_string};
use objc2::rc::{Allocated, Retained, autoreleasepool};
use objc2::runtime::{AnyClass, AnyObject as Object, Bool};
use objc2::{class, msg_send};

#[link(name = "WebKit", kind = "framework")]
unsafe extern "C" {}

fn main() {
    // Expose this main-thread test to cargo-nextest's libtest discovery.
    if std::env::args().any(|arg| arg == "--list") {
        println!("svg_fallback: test");
        return;
    }
    autoreleasepool(|_| {
        // SAFETY: This test's main function owns all AppKit/WebKit objects on the
        // process main thread and pumps that thread's run loop for callbacks.
        unsafe {
            let _: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let frame = Rect {
                origin: Point { x: 0.0, y: 0.0 },
                size: Size {
                    width: 100.0,
                    height: 100.0,
                },
            };
            let document = svg::parse_svg(include_bytes!("fixtures/native.tiff"));
            let view = svg::create_svg_view(frame, &document);
            let window: Allocated<Object> = msg_send![class!(NSWindow), alloc];
            let window: Retained<Object> = msg_send![window, initWithContentRect: frame, styleMask: 0u64, backing: 2u64, defer: Bool::NO];
            let _: () = msg_send![&*window, setContentView: view.as_ptr()];
            let until = Instant::now() + Duration::from_secs(15);
            let mut found = false;
            while Instant::now() < until && !found {
                let run_loop: *mut Object = msg_send![class!(NSRunLoop), mainRunLoop];
                let date: *mut Object =
                    msg_send![class!(NSDate), dateWithTimeIntervalSinceNow: 0.01f64];
                let _: Bool = msg_send![run_loop, runMode: ns_string("kCFRunLoopDefaultMode"), beforeDate: date];
                let children: *mut Object = msg_send![&*view, subviews];
                let count: usize = msg_send![children, count];
                for i in 0..count {
                    let child: *mut Object = msg_send![children, objectAtIndex: i];
                    let image_view: Bool = msg_send![child, isKindOfClass: AnyClass::get(c"NSImageView").expect("NSImageView exists") as *const AnyClass];
                    if image_view.as_bool() {
                        let image: *mut Object = msg_send![child, image];
                        assert!(!image.is_null());
                        found = true;
                    }
                }
            }
            assert!(
                found,
                "WebKit image load failure must display the NSImage fallback"
            );
            println!("WebKit load failure -> NSImage fallback passed");
        }
    });
}
