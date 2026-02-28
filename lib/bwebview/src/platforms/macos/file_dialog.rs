/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;
use std::path::PathBuf;

use block2::RcBlock;
use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, msg_send, sel};

use super::cocoa::*;

#[cfg(feature = "file_dialog")]
pub(crate) struct PlatformFileDialog;

#[cfg(feature = "file_dialog")]
impl crate::FileDialogInterface for PlatformFileDialog {
    fn pick_file(dialog: crate::FileDialog) -> Option<std::path::PathBuf> {
        unsafe {
            let panel: *mut Object = msg_send![class!(NSOpenPanel), openPanel];
            let _: () = msg_send![panel, setCanChooseFiles: Bool::YES];
            let _: () = msg_send![panel, setCanChooseDirectories: Bool::NO];
            let _: () = msg_send![panel, setAllowsMultipleSelection: Bool::NO];
            setup_ns_panel(panel, &dialog);
            let result: i64 = run_panel_modal(panel);
            if result == NS_MODAL_RESPONSE_OK {
                let urls: *mut Object = msg_send![panel, URLs];
                let url: *mut Object = msg_send![urls, objectAtIndex: 0usize];
                let path: NSString = msg_send![url, path];
                Some(std::path::PathBuf::from(path.to_string()))
            } else {
                None
            }
        }
    }

    fn pick_files(dialog: crate::FileDialog) -> Option<Vec<std::path::PathBuf>> {
        unsafe {
            let panel: *mut Object = msg_send![class!(NSOpenPanel), openPanel];
            let _: () = msg_send![panel, setCanChooseFiles: Bool::YES];
            let _: () = msg_send![panel, setCanChooseDirectories: Bool::NO];
            let _: () = msg_send![panel, setAllowsMultipleSelection: Bool::YES];
            setup_ns_panel(panel, &dialog);
            let result: i64 = run_panel_modal(panel);
            if result == NS_MODAL_RESPONSE_OK {
                let urls: *mut Object = msg_send![panel, URLs];
                let count: usize = msg_send![urls, count];
                let paths: Vec<_> = (0..count)
                    .map(|i| {
                        let url: *mut Object = msg_send![urls, objectAtIndex: i];
                        let path: NSString = msg_send![url, path];
                        std::path::PathBuf::from(path.to_string())
                    })
                    .collect();
                if paths.is_empty() { None } else { Some(paths) }
            } else {
                None
            }
        }
    }

    fn save_file(dialog: crate::FileDialog) -> Option<std::path::PathBuf> {
        unsafe {
            let panel: *mut Object = msg_send![class!(NSSavePanel), savePanel];
            setup_ns_panel(panel, &dialog);
            if let Some(filename) = &dialog.filename {
                let _: () = msg_send![panel, setNameFieldStringValue: NSString::from_str(filename)];
            }
            let result: i64 = run_panel_modal(panel);
            if result == NS_MODAL_RESPONSE_OK {
                let url: *mut Object = msg_send![panel, URL];
                let path: NSString = msg_send![url, path];
                Some(std::path::PathBuf::from(path.to_string()))
            } else {
                None
            }
        }
    }
}

#[cfg(feature = "file_dialog")]
unsafe fn setup_ns_panel(panel: *mut Object, dialog: &crate::FileDialog) {
    if let Some(title) = &dialog.title {
        let _: () = msg_send![panel, setTitle: NSString::from_str(title)];
    }
    if let Some(dir) = &dialog.directory {
        let url: *mut Object =
            msg_send![class!(NSURL), fileURLWithPath: NSString::from_str(dir.to_string_lossy())];
        let _: () = msg_send![panel, setDirectoryURL: url];
    }
    if !dialog.filters.is_empty() {
        let arr: *mut Object = msg_send![class!(NSMutableArray), new];
        for filter in &dialog.filters {
            for ext in &filter.extensions {
                let _: () = msg_send![arr, addObject: NSString::from_str(ext)];
            }
        }
        // setAllowedFileTypes: is deprecated in macOS 12 but still functional
        let _: () = msg_send![panel, setAllowedFileTypes: arr];
    }
}

#[cfg(feature = "file_dialog")]
unsafe fn run_panel_modal(panel: *mut Object) -> i64 {
    let key_window: *mut Object = unsafe { msg_send![NSApp, keyWindow] };
    if !key_window.is_null() {
        // Show as a sheet attached to the active window
        let block = RcBlock::new(move |response: i64| {
            let _: () = unsafe { msg_send![NSApp, stopModalWithCode: response] };
        });
        let _: () =
            msg_send![panel, beginSheetModalForWindow: key_window, completionHandler: &*block];
        unsafe { msg_send![NSApp, runModalForWindow: panel] }
    } else {
        msg_send![panel, runModal]
    }
}
