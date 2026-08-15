/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;

use macview_appkit::{Point, Rect, Size};
use objc2::ffi::{objc_msgSendSuper, objc_super};
use objc2::runtime::{AnyClass, AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};

use crate::cocoa::*;

/// The magnification the media may be shrunk and enlarged to.
const MINIMUM_MAGNIFICATION: f64 = 0.02;
const MAXIMUM_MAGNIFICATION: f64 = 64.0;

/// The space kept free around the media when all of it is shown.
///
/// The margin belongs to the window and not to the media, so that every format is shown the same
/// way and switching between showing all of the media and showing it at its own size moves it by
/// the difference in size alone.
pub(crate) const MARGIN: f64 = 16.0;

define_class!(
    #[unsafe(super(NSClipView))]
    #[name = "MacViewClipView"]
    struct ClipView;

    impl ClipView {
        #[unsafe(method(constrainBoundsRect:))]
        fn _constrain_bounds_rect(&self, proposed: Rect) -> Rect {
            self.constrain_bounds_rect(proposed)
        }
    }
);

impl ClipView {
    /// Centers media that is smaller than the visible area instead of pinning it to a corner.
    fn constrain_bounds_rect(&self, proposed: Rect) -> Rect {
        // SAFETY: objc_msgSendSuper invokes NSClipView's own constraining with the exact
        // Objective-C ABI, and the document view is the one the scroll view installed.
        unsafe {
            let this = self as *const Self as *mut Object;
            let super_info = objc_super {
                receiver: this,
                super_class: class!(NSClipView).cast::<AnyClass>(),
            };
            let send: unsafe extern "C" fn(*const objc_super, *const c_void, Rect) -> Rect =
                std::mem::transmute(objc_msgSendSuper as *const c_void);
            let mut rect = send(&super_info, sel!(constrainBoundsRect:).0, proposed);

            let document: *mut Object = msg_send![this, documentView];
            if document.is_null() {
                return rect;
            }
            let frame: Rect = msg_send![document, frame];
            if rect.size.width > frame.size.width {
                rect.origin.x = (frame.size.width - rect.size.width) / 2.0;
            }
            if rect.size.height > frame.size.height {
                rect.origin.y = (frame.size.height - rect.size.height) / 2.0;
            }
            rect
        }
    }
}

define_class!(
    #[unsafe(super(NSScrollView))]
    #[name = "MacViewScrollView"]
    struct ScrollView;

    impl ScrollView {
        /// Shows the media at the given magnification, in the middle of the window.
        #[unsafe(method(setZoom:))]
        fn _set_zoom(&self, magnification: f64) {
            self.magnify(magnification);
            self.scroll_to_center();
        }

        /// Magnifies by a step, keeping the middle of the visible area in place.
        #[unsafe(method(zoomBy:))]
        fn _zoom_by(&self, factor: f64) {
            // SAFETY: The scroll view magnifies itself.
            let magnification: f64 =
                unsafe { msg_send![self as *const Self as *mut Object, magnification] };
            self.magnify_around_center(magnification * factor);
        }

        /// Shows all of the media, in the middle of the window.
        #[unsafe(method(zoomToFit))]
        fn _zoom_to_fit(&self) {
            self.magnify(self.fit_magnification());
            self.scroll_to_center();
        }
    }
);

impl ScrollView {
    /// Returns the magnification that shows all of the media inside its margin.
    fn fit_magnification(&self) -> f64 {
        // SAFETY: The scroll view keeps its document view alive, and its content size is the
        // visible area in points.
        unsafe {
            let this = self as *const Self as *mut Object;
            let document: *mut Object = msg_send![this, documentView];
            if document.is_null() {
                return 1.0;
            }
            let media: Rect = msg_send![document, frame];
            if media.size.width <= 0.0 || media.size.height <= 0.0 {
                return 1.0;
            }
            let content: Size = msg_send![this, contentSize];
            let width = (content.width - MARGIN * 2.0).max(1.0);
            let height = (content.height - MARGIN * 2.0).max(1.0);
            (width / media.size.width)
                .min(height / media.size.height)
                .clamp(MINIMUM_MAGNIFICATION, MAXIMUM_MAGNIFICATION)
        }
    }

    /// Magnifies the media, leaving the scroll position to the caller.
    fn magnify(&self, magnification: f64) {
        // SAFETY: The scroll view clamps the magnification to its own range.
        unsafe {
            let this = self as *const Self as *mut Object;
            let _: () = msg_send![this, setMagnification: magnification];
        }
    }

    /// Magnifies the media, keeping the middle of the visible area in place.
    fn magnify_around_center(&self, magnification: f64) {
        // SAFETY: The scroll view keeps its clip view alive and clamps the magnification.
        unsafe {
            let this = self as *const Self as *mut Object;
            let clip_view: *mut Object = msg_send![this, contentView];
            let bounds: Rect = msg_send![clip_view, bounds];
            let _: () = msg_send![this,
                setMagnification: magnification,
                centeredAtPoint: Point {
                    x: bounds.origin.x + bounds.size.width / 2.0,
                    y: bounds.origin.y + bounds.size.height / 2.0,
                }
            ];
        }
    }

    /// Scrolls back to the middle of the media, so that it cannot stay stuck in a corner.
    fn scroll_to_center(&self) {
        // SAFETY: The scroll view keeps its document and clip views alive, and the clip view
        // constrains the point it is scrolled to.
        unsafe {
            let this = self as *const Self as *mut Object;
            let document: *mut Object = msg_send![this, documentView];
            let clip_view: *mut Object = msg_send![this, contentView];
            if document.is_null() {
                return;
            }
            let media: Rect = msg_send![document, frame];
            let bounds: Rect = msg_send![clip_view, bounds];
            let _: () = msg_send![clip_view,
                scrollToPoint: Point {
                    x: (media.size.width - bounds.size.width) / 2.0,
                    y: (media.size.height - bounds.size.height) / 2.0,
                }
            ];
            let _: () = msg_send![this, reflectScrolledClipView: clip_view];
        }
    }
}

/// Creates an owned scroll view that scrolls and magnifies `document` inside `frame`.
///
/// Scrolling, panning, pinching, smart magnifying, the elastic edges and the scrollers all come
/// from `NSScrollView` itself. The caller owns the returned view and must send it `release`.
pub(crate) fn create_scroll_view(frame: Rect, document: *mut Object) -> *mut Object {
    // SAFETY: All objects are valid AppKit instances, and the clip view is released after the
    // scroll view retains it.
    unsafe {
        let scroll_view: *mut Object = msg_send![ScrollView::class(), alloc];
        let scroll_view: *mut Object = msg_send![scroll_view, initWithFrame: frame];
        assert!(!scroll_view.is_null(), "failed to create scroll view");

        let clip_view: *mut Object = msg_send![ClipView::class(), alloc];
        let clip_view: *mut Object = msg_send![clip_view, initWithFrame: frame];
        // The checkerboard behind the scroll view is the background of the window.
        let _: () = msg_send![clip_view, setDrawsBackground: Bool::NO];
        let _: () = msg_send![scroll_view, setContentView: clip_view];
        let _: () = msg_send![clip_view, release];

        let _: () = msg_send![scroll_view, setDrawsBackground: Bool::NO];
        let _: () = msg_send![scroll_view, setBorderType: NS_NO_BORDER];
        let _: () = msg_send![scroll_view, setHasHorizontalScroller: Bool::YES];
        let _: () = msg_send![scroll_view, setHasVerticalScroller: Bool::YES];
        let _: () = msg_send![scroll_view, setAutohidesScrollers: Bool::YES];
        let _: () = msg_send![scroll_view, setAllowsMagnification: Bool::YES];
        let _: () = msg_send![scroll_view, setMinMagnification: MINIMUM_MAGNIFICATION];
        let _: () = msg_send![scroll_view, setMaxMagnification: MAXIMUM_MAGNIFICATION];
        let _: () = msg_send![scroll_view, setDocumentView: document];
        let _: () = msg_send![scroll_view,
            setAutoresizingMask: NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE
        ];
        scroll_view
    }
}
