/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::PathBuf;

use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send};

use super::cocoa::{NS_DRAG_OPERATION_COPY, NSFilenamesPboardType, NSString};
use super::event_loop::send_event;
use crate::WindowEvent;

/// Sends a `DroppedFile` event for every file in a completed drag operation
pub(super) fn perform_file_drop(sender: *mut Object) -> Bool {
    unsafe {
        let pasteboard: *mut Object = msg_send![sender, draggingPasteboard];
        let filenames: *mut Object =
            msg_send![pasteboard, propertyListForType:NSFilenamesPboardType];
        if filenames.is_null() {
            return Bool::NO;
        }
        let count: usize = msg_send![filenames, count];
        for index in 0..count {
            let filename: NSString = msg_send![filenames, objectAtIndex:index];
            send_event(crate::Event::Window(WindowEvent::DroppedFile(
                PathBuf::from(filename.to_string()),
            )));
        }
        Bool::YES
    }
}

/// Registers a window or view for file drags
pub(super) unsafe fn register_dragged_types(view: *mut Object) {
    let dragged_types: *mut Object =
        unsafe { msg_send![class!(NSArray), arrayWithObject:NSFilenamesPboardType] };
    let _: () = unsafe { msg_send![view, registerForDraggedTypes:dragged_types] };
}

define_class!(
    #[unsafe(super(WKWebView))]
    #[name = "BWebviewDroppableWebview"]
    struct DroppableWebview;

    impl DroppableWebview {
        #[unsafe(method(draggingEntered:))]
        const fn _dragging_entered(&self, _: *mut Object) -> u64 {
            NS_DRAG_OPERATION_COPY
        }

        #[unsafe(method(draggingUpdated:))]
        const fn _dragging_updated(&self, _: *mut Object) -> u64 {
            NS_DRAG_OPERATION_COPY
        }

        #[unsafe(method(prepareForDragOperation:))]
        const fn _prepare_for_drag_operation(&self, _: *mut Object) -> Bool {
            Bool::YES
        }

        #[unsafe(method(performDragOperation:))]
        fn _perform_drag_operation(&self, sender: *mut Object) -> Bool {
            perform_file_drop(sender)
        }
    }
);

/// A WKWebView subclass that reports native file drags instead of navigating to them
pub(super) fn droppable_webview_class() -> *mut Object {
    DroppableWebview::class()
}
