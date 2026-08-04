/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CStr, CString, c_char, c_void};
use std::path::PathBuf;
use std::ptr::{null, null_mut};

use super::headers::*;

pub(crate) struct PlatformFileDialog;

impl crate::FileDialogInterface for PlatformFileDialog {
    fn pick_file(dialog: crate::FileDialog) -> Option<PathBuf> {
        open_files_impl(dialog, false).map(|mut v| v.remove(0))
    }

    fn pick_files(dialog: crate::FileDialog) -> Option<Vec<PathBuf>> {
        open_files_impl(dialog, true)
    }

    fn save_file(dialog: crate::FileDialog) -> Option<PathBuf> {
        unsafe {
            let title = CString::new(dialog.title.as_deref().unwrap_or("Save File"))
                .expect("Can't convert to CString");

            let chooser = cfg_select! {
                gtk3_20 => gtk_file_chooser_native_new(
                    title.as_ptr(),
                    null_mut(),
                    GTK_FILE_CHOOSER_ACTION_SAVE,
                    c"_Save".as_ptr(),
                    c"_Cancel".as_ptr(),
                ) as *mut c_void,
                _ => gtk_file_chooser_dialog_new(
                    title.as_ptr(),
                    null_mut(),
                    GTK_FILE_CHOOSER_ACTION_SAVE,
                    c"_Cancel".as_ptr(),
                    GTK_RESPONSE_CANCEL,
                    c"_Save".as_ptr(),
                    GTK_RESPONSE_ACCEPT,
                    null::<c_char>(),
                ) as *mut c_void,
            };

            if let Some(dir) = &dialog.directory {
                let dir_c =
                    CString::new(dir.to_string_lossy().as_ref()).expect("Can't convert to CString");
                gtk_file_chooser_set_current_folder(chooser, dir_c.as_ptr());
            }
            if let Some(filename) = &dialog.filename {
                let name_c = CString::new(filename.as_str()).expect("Can't convert to CString");
                gtk_file_chooser_set_current_name(chooser, name_c.as_ptr());
            }
            add_gtk_filters(chooser, &dialog.filters);

            let result = cfg_select! {
                gtk3_20 => gtk_native_dialog_run(chooser as *mut GtkNativeDialog),
                _ => gtk_dialog_run(chooser as *mut GtkWidget),
            };

            let path = if result == GTK_RESPONSE_ACCEPT {
                let raw = gtk_file_chooser_get_filename(chooser);
                if !raw.is_null() {
                    let p = PathBuf::from(CStr::from_ptr(raw).to_string_lossy().into_owned());
                    g_free(raw as *mut c_void);
                    Some(p)
                } else {
                    None
                }
            } else {
                None
            };

            cfg_select! {
                gtk3_20 => g_object_unref(chooser as *mut GObject),
                _ => gtk_widget_destroy(chooser as *mut GtkWidget),
            }
            path
        }
    }
}

fn open_files_impl(dialog: crate::FileDialog, multiple: bool) -> Option<Vec<PathBuf>> {
    unsafe {
        let title = CString::new(dialog.title.as_deref().unwrap_or("Open File"))
            .expect("Can't convert to CString");

        let chooser = cfg_select! {
            gtk3_20 => gtk_file_chooser_native_new(
                title.as_ptr(),
                null_mut(),
                GTK_FILE_CHOOSER_ACTION_OPEN,
                c"_Open".as_ptr(),
                c"_Cancel".as_ptr(),
            ) as *mut c_void,
            _ => gtk_file_chooser_dialog_new(
                title.as_ptr(),
                null_mut(),
                GTK_FILE_CHOOSER_ACTION_OPEN,
                c"_Cancel".as_ptr(),
                GTK_RESPONSE_CANCEL,
                c"_Open".as_ptr(),
                GTK_RESPONSE_ACCEPT,
                null::<c_char>(),
            ) as *mut c_void,
        };

        if multiple {
            gtk_file_chooser_set_select_multiple(chooser, true);
        }
        if let Some(dir) = &dialog.directory {
            let dir_c =
                CString::new(dir.to_string_lossy().as_ref()).expect("Can't convert to CString");
            gtk_file_chooser_set_current_folder(chooser, dir_c.as_ptr());
        }
        add_gtk_filters(chooser, &dialog.filters);

        let result = cfg_select! {
            gtk3_20 => gtk_native_dialog_run(chooser as *mut GtkNativeDialog),
            _ => gtk_dialog_run(chooser as *mut GtkWidget),
        };

        let paths = if result == GTK_RESPONSE_ACCEPT {
            if multiple {
                let slist = gtk_file_chooser_get_filenames(chooser);
                if slist.is_null() {
                    None
                } else {
                    let mut paths = Vec::new();
                    let mut node = slist;
                    while !node.is_null() {
                        let raw = (*node).data as *const c_char;
                        if !raw.is_null() {
                            paths.push(PathBuf::from(
                                CStr::from_ptr(raw).to_string_lossy().into_owned(),
                            ));
                        }
                        node = (*node).next;
                    }
                    g_slist_free_full(slist, g_free);
                    if paths.is_empty() { None } else { Some(paths) }
                }
            } else {
                let raw = gtk_file_chooser_get_filename(chooser);
                if !raw.is_null() {
                    let p = PathBuf::from(CStr::from_ptr(raw).to_string_lossy().into_owned());
                    g_free(raw as *mut c_void);
                    Some(vec![p])
                } else {
                    None
                }
            }
        } else {
            None
        };

        cfg_select! {
            gtk3_20 => g_object_unref(chooser as *mut GObject),
            _ => gtk_widget_destroy(chooser as *mut GtkWidget),
        }
        paths
    }
}

unsafe fn add_gtk_filters(chooser: *mut c_void, filters: &[crate::FileDialogFilter]) {
    for filter in filters {
        let f = unsafe { gtk_file_filter_new() };
        let name = CString::new(filter.name.as_str()).expect("Can't convert to CString");
        unsafe { gtk_file_filter_set_name(f, name.as_ptr()) };
        for ext in &filter.extensions {
            let pattern = format!("*.{ext}");
            let pat = CString::new(pattern.as_str()).expect("Can't convert to CString");
            unsafe { gtk_file_filter_add_pattern(f, pat.as_ptr()) };
        }
        unsafe { gtk_file_chooser_add_filter(chooser, f) };
    }
}
