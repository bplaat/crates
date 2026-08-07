/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CStr, OsStr, c_void};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr::{null, null_mut};

use super::event_loop::send_event;
use super::headers::*;
use super::webview::WebviewData;
use crate::WindowEvent;

/// The paths of the drag in progress, collected before the drop is confirmed
#[derive(Default)]
pub(super) struct FileDropState {
    paths: Option<Vec<PathBuf>>,
    leaving: bool,
}

/// Reports native file drags on the web view instead of letting WebKit navigate to them
pub(super) unsafe fn connect_signals(webview: *mut WebKitWebView, data: &WebviewData) {
    let data = data as *const WebviewData as *const c_void;
    unsafe {
        g_signal_connect_data(
            webview as *mut GObject,
            c"drag-leave".as_ptr(),
            webview_on_drag_leave as *const c_void,
            data,
            null(),
            G_CONNECT_DEFAULT,
        );
        g_signal_connect_data(
            webview as *mut GObject,
            c"drag-drop".as_ptr(),
            webview_on_drag_drop as *const c_void,
            data,
            null(),
            G_CONNECT_DEFAULT,
        );
        g_signal_connect_data(
            webview as *mut GObject,
            c"drag-data-received".as_ptr(),
            webview_on_drag_data_received as *const c_void,
            data,
            null(),
            G_CONNECT_DEFAULT,
        );
    }
}

const extern "C" fn webview_on_drag_leave(
    _webview: *mut WebKitWebView,
    _context: *mut GdkDragContext,
    _time: u32,
    data: &mut WebviewData,
) {
    data.file_drop.leaving = true;
}

extern "C" fn webview_on_drag_drop(
    _webview: *mut WebKitWebView,
    context: *mut GdkDragContext,
    _x: i32,
    _y: i32,
    time: u32,
    data: &mut WebviewData,
) -> i32 {
    if data.file_drop.leaving
        && let Some(paths) = data.file_drop.paths.take()
    {
        unsafe { gtk_drag_finish(context, 1, 0, time) };
        data.file_drop.leaving = false;
        for path in paths {
            send_event(crate::Event::Window(WindowEvent::DroppedFile(path)));
        }
        return 1;
    }
    0
}

extern "C" fn webview_on_drag_data_received(
    _webview: *mut WebKitWebView,
    _context: *mut GdkDragContext,
    _x: i32,
    _y: i32,
    selection_data: *mut GtkSelectionData,
    info: u32,
    _time: u32,
    data: &mut WebviewData,
) {
    // WebKitGTK registers URI lists as target info 2. Other data requests are
    // used internally by the web view and are not native file drags.
    if info != 2 {
        return;
    }

    let uris = unsafe { gtk_selection_data_get_uris(selection_data) };
    if !uris.is_null() {
        let mut paths = Vec::new();
        let mut uri = uris;
        while !unsafe { *uri }.is_null() {
            let filename = unsafe { g_filename_from_uri(*uri, null_mut(), null_mut()) };
            if !filename.is_null() {
                let path = PathBuf::from(OsStr::from_bytes(unsafe {
                    CStr::from_ptr(filename).to_bytes()
                }));
                unsafe { g_free(filename as *mut c_void) };
                paths.push(path);
            }
            uri = unsafe { uri.add(1) };
        }
        unsafe { g_strfreev(uris) };
        data.file_drop.paths = Some(paths);
        data.file_drop.leaving = false;
    }
}
