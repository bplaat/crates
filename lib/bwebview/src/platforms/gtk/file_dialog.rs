/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CStr, CString, c_char, c_void};
use std::path::PathBuf;
use std::ptr::null_mut;

use super::headers::*;

#[cfg(feature = "file_dialog")]
pub(crate) struct PlatformFileDialog;

#[cfg(feature = "file_dialog")]
impl crate::FileDialogInterface for PlatformFileDialog {
    fn pick_file(dialog: crate::FileDialog) -> Option<std::path::PathBuf> {
        open_files_impl(dialog, false).map(|mut v| v.remove(0))
    }

    fn pick_files(dialog: crate::FileDialog) -> Option<Vec<std::path::PathBuf>> {
        open_files_impl(dialog, true)
    }

    fn save_file(dialog: crate::FileDialog) -> Option<std::path::PathBuf> {
        unsafe {
            let title = CString::new(dialog.title.as_deref().unwrap_or("Save File"))
                .expect("Can't convert to CString");
            let accept = CString::new("_Save").expect("Can't convert to CString");
            let cancel = CString::new("_Cancel").expect("Can't convert to CString");

            let chooser = gtk_file_chooser_native_new(
                title.as_ptr(),
                null_mut(),
                GTK_FILE_CHOOSER_ACTION_SAVE,
                accept.as_ptr(),
                cancel.as_ptr(),
            );

            if let Some(dir) = &dialog.directory {
                let dir_c =
                    CString::new(dir.to_string_lossy().as_ref()).expect("Can't convert to CString");
                gtk_file_chooser_set_current_folder(chooser as *mut c_void, dir_c.as_ptr());
            }
            if let Some(filename) = &dialog.filename {
                let name_c = CString::new(filename.as_str()).expect("Can't convert to CString");
                gtk_file_chooser_set_current_name(chooser as *mut c_void, name_c.as_ptr());
            }
            add_gtk_filters(chooser as *mut c_void, &dialog.filters);

            let result = gtk_native_dialog_run(chooser as *mut GtkNativeDialog);
            let path = if result == GTK_RESPONSE_ACCEPT {
                let raw = gtk_file_chooser_get_filename(chooser as *mut c_void);
                if !raw.is_null() {
                    let p = std::path::PathBuf::from(
                        CStr::from_ptr(raw).to_string_lossy().into_owned(),
                    );
                    g_free(raw as *mut c_void);
                    Some(p)
                } else {
                    None
                }
            } else {
                None
            };

            g_object_unref(chooser as *mut GObject);
            path
        }
    }
}

#[cfg(feature = "file_dialog")]
fn open_files_impl(dialog: crate::FileDialog, multiple: bool) -> Option<Vec<std::path::PathBuf>> {
    unsafe {
        let title = CString::new(dialog.title.as_deref().unwrap_or("Open File"))
            .expect("Can't convert to CString");
        let accept = CString::new("_Open").expect("Can't convert to CString");
        let cancel = CString::new("_Cancel").expect("Can't convert to CString");

        let chooser = gtk_file_chooser_native_new(
            title.as_ptr(),
            null_mut(),
            GTK_FILE_CHOOSER_ACTION_OPEN,
            accept.as_ptr(),
            cancel.as_ptr(),
        );

        if multiple {
            gtk_file_chooser_set_select_multiple(chooser as *mut c_void, true);
        }
        if let Some(dir) = &dialog.directory {
            let dir_c =
                CString::new(dir.to_string_lossy().as_ref()).expect("Can't convert to CString");
            gtk_file_chooser_set_current_folder(chooser as *mut c_void, dir_c.as_ptr());
        }
        add_gtk_filters(chooser as *mut c_void, &dialog.filters);

        let result = gtk_native_dialog_run(chooser as *mut GtkNativeDialog);
        let paths = if result == GTK_RESPONSE_ACCEPT {
            if multiple {
                let slist = gtk_file_chooser_get_filenames(chooser as *mut c_void);
                if slist.is_null() {
                    None
                } else {
                    let mut paths = Vec::new();
                    let mut node = slist;
                    while !node.is_null() {
                        let raw = (*node).data as *const c_char;
                        if !raw.is_null() {
                            paths.push(std::path::PathBuf::from(
                                CStr::from_ptr(raw).to_string_lossy().into_owned(),
                            ));
                        }
                        node = (*node).next;
                    }
                    g_slist_free_full(slist, g_free);
                    if paths.is_empty() { None } else { Some(paths) }
                }
            } else {
                let raw = gtk_file_chooser_get_filename(chooser as *mut c_void);
                if !raw.is_null() {
                    let p = std::path::PathBuf::from(
                        CStr::from_ptr(raw).to_string_lossy().into_owned(),
                    );
                    g_free(raw as *mut c_void);
                    Some(vec![p])
                } else {
                    None
                }
            }
        } else {
            None
        };

        g_object_unref(chooser as *mut GObject);
        paths
    }
}

#[cfg(feature = "file_dialog")]
unsafe fn add_gtk_filters(chooser: *mut c_void, filters: &[crate::FileDialogFilter]) {
    for filter in filters {
        let f = gtk_file_filter_new();
        let name = CString::new(filter.name.as_str()).expect("Can't convert to CString");
        gtk_file_filter_set_name(f, name.as_ptr());
        for ext in &filter.extensions {
            let pattern = format!("*.{ext}");
            let pat = CString::new(pattern.as_str()).expect("Can't convert to CString");
            gtk_file_filter_add_pattern(f, pat.as_ptr());
        }
        gtk_file_chooser_add_filter(chooser, f);
    }
}
