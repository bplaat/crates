/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::PathBuf;

use objc2::msg_send;
use objc2::runtime::{AnyObject as Object, Bool};

use super::cocoa::{NSFilenamesPboardType, NSString};
use super::event_loop::send_event;
use crate::WindowEvent;

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
