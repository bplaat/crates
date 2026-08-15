/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::Cell;

use macview_appkit::Size;
use objc2::runtime::AnyObject as Object;
use objc2::{define_class, msg_send};

use crate::cocoa::ns_string;

struct WindowControllerIvars {
    size: Cell<Size>,
}

define_class!(
    #[unsafe(super(NSWindowController))]
    #[name = "MacViewWindowController"]
    #[ivars = WindowControllerIvars]
    struct WindowController;

    impl WindowController {
        #[unsafe(method(windowTitleForDocumentDisplayName:))]
        fn _window_title_for_document_display_name(
            &self,
            display_name: *mut Object,
        ) -> *mut Object {
            self.window_title(display_name)
        }
    }
);

impl WindowController {
    fn window_title(&self, display_name: *mut Object) -> *mut Object {
        let size = self.ivars().size.get();
        let suffix = format!(
            " ({}x{})",
            size.width.round() as i64,
            size.height.round() as i64
        );
        // SAFETY: AppKit passes a valid NSString and the appended string is autoreleased, which
        // matches the ownership convention of this method.
        unsafe { msg_send![display_name, stringByAppendingString: ns_string(&suffix)] }
    }
}

/// Creates an owned `NSWindowController` that shows the media size in its window title.
///
/// The caller owns the returned controller and must send it `release`.
pub(crate) fn create_window_controller(window: *mut Object, size: Size) -> *mut Object {
    // SAFETY: WindowController is a registered NSWindowController subclass and window is a live
    // NSWindow. The zero initialized ivars are filled in before AppKit asks for a window title.
    unsafe {
        let controller: *mut Object = msg_send![WindowController::class(), alloc];
        let controller: *mut Object = msg_send![controller, initWithWindow: window];
        assert!(!controller.is_null(), "failed to create window controller");
        let controller_ref = &*(controller.cast::<WindowController>());
        controller_ref.ivars().size.set(size);
        controller
    }
}
