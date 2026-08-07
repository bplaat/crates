/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::PathBuf;
use std::sync::OnceLock;

use objc2::runtime::{AnyObject as Object, Bool, ClassBuilder, Sel};
use objc2::{class, msg_send, sel};

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

const extern "C" fn webview_dragging_entered(_: *mut Object, _: Sel, _: *mut Object) -> u64 {
    NS_DRAG_OPERATION_COPY
}

const extern "C" fn webview_prepare_for_drag_operation(
    _: *mut Object,
    _: Sel,
    _: *mut Object,
) -> Bool {
    Bool::YES
}

extern "C" fn webview_perform_drag_operation(_: *mut Object, _: Sel, sender: *mut Object) -> Bool {
    perform_file_drop(sender)
}

/// A WKWebView subclass that reports native file drags instead of navigating to them
pub(super) fn droppable_webview_class() -> *mut Object {
    static CLASS: OnceLock<usize> = OnceLock::new();
    *CLASS.get_or_init(|| {
        let mut builder = ClassBuilder::new(c"BWebviewDroppableWebview", class!(WKWebView))
            .expect("Failed to create droppable webview class");
        assert!(builder.add_method(
            sel!(draggingEntered:),
            webview_dragging_entered as extern "C" fn(_, _, _) -> _
        ));
        assert!(builder.add_method(
            sel!(draggingUpdated:),
            webview_dragging_entered as extern "C" fn(_, _, _) -> _
        ));
        assert!(builder.add_method(
            sel!(prepareForDragOperation:),
            webview_prepare_for_drag_operation as extern "C" fn(_, _, _) -> _
        ));
        assert!(builder.add_method(
            sel!(performDragOperation:),
            webview_perform_drag_operation as extern "C" fn(_, _, _) -> _
        ));
        builder.register() as usize
    }) as *mut Object
}
