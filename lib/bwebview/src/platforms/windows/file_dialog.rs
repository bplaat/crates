/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::CString;
use std::path::PathBuf;
use std::ptr::{null, null_mut};

use super::win32::*;

#[cfg(feature = "file_dialog")]
pub(crate) struct PlatformFileDialog;

#[cfg(feature = "file_dialog")]
impl crate::FileDialogInterface for PlatformFileDialog {
    fn pick_file(dialog: crate::FileDialog) -> Option<PathBuf> {
        unsafe {
            let mut raw: *mut IFileOpenDialog = null_mut();
            if CoCreateInstance(
                &CLSID_FileOpenDialog,
                null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_IFileOpenDialog,
                &mut raw as *mut _ as *mut *mut c_void,
            ) != S_OK
            {
                return None;
            }
            let dlg = &*raw;
            dlg.SetOptions(FOS_NOCHANGEDIR | FOS_PATHMUSTEXIST | FOS_FILEMUSTEXIST);
            let title_w = dialog.title.as_deref().map(|s| s.to_wide_string());
            if let Some(ref t) = title_w {
                dlg.SetTitle(t.as_ptr());
            }
            let dir_item = make_shell_item_from_path(dialog.directory.as_ref());
            if let Some(item) = dir_item {
                dlg.SetFolder(item);
                (*item).Release();
            }
            let (specs, _storage) = build_com_filters(&dialog.filters);
            if !specs.is_empty() {
                dlg.SetFileTypes(specs.len() as u32, specs.as_ptr());
            }
            let hwnd = #[allow(static_mut_refs)]
            FIRST_HWND.unwrap_or(null_mut());
            let path = if dlg.Show(hwnd) == S_OK {
                let mut item: *mut IShellItem = null_mut();
                if dlg.GetResult(&mut item) == S_OK {
                    let p = shell_item_path(item);
                    (*item).Release();
                    p
                } else {
                    None
                }
            } else {
                None
            };
            dlg.Release();
            path
        }
    }

    fn pick_files(dialog: crate::FileDialog) -> Option<Vec<PathBuf>> {
        unsafe {
            let mut raw: *mut IFileOpenDialog = null_mut();
            if CoCreateInstance(
                &CLSID_FileOpenDialog,
                null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_IFileOpenDialog,
                &mut raw as *mut _ as *mut *mut c_void,
            ) != S_OK
            {
                return None;
            }
            let dlg = &*raw;
            dlg.SetOptions(
                FOS_NOCHANGEDIR | FOS_PATHMUSTEXIST | FOS_FILEMUSTEXIST | FOS_ALLOWMULTISELECT,
            );
            let title_w = dialog.title.as_deref().map(|s| s.to_wide_string());
            if let Some(ref t) = title_w {
                dlg.SetTitle(t.as_ptr());
            }
            let dir_item = make_shell_item_from_path(dialog.directory.as_ref());
            if let Some(item) = dir_item {
                dlg.SetFolder(item);
                (*item).Release();
            }
            let (specs, _storage) = build_com_filters(&dialog.filters);
            if !specs.is_empty() {
                dlg.SetFileTypes(specs.len() as u32, specs.as_ptr());
            }
            let hwnd = #[allow(static_mut_refs)]
            FIRST_HWND.unwrap_or(null_mut());
            let paths = if dlg.Show(hwnd) == S_OK {
                let mut items: *mut IShellItemArray = null_mut();
                if dlg.GetResults(&mut items) == S_OK {
                    let mut count: u32 = 0;
                    (*items).GetCount(&mut count);
                    let mut result = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        let mut item: *mut IShellItem = null_mut();
                        if (*items).GetItemAt(i, &mut item) == S_OK {
                            if let Some(p) = shell_item_path(item) {
                                result.push(p);
                            }
                            (*item).Release();
                        }
                    }
                    (*items).Release();
                    if result.is_empty() {
                        None
                    } else {
                        Some(result)
                    }
                } else {
                    None
                }
            } else {
                None
            };
            dlg.Release();
            paths
        }
    }

    fn save_file(dialog: crate::FileDialog) -> Option<PathBuf> {
        unsafe {
            let mut raw: *mut IFileSaveDialog = null_mut();
            if CoCreateInstance(
                &CLSID_FileSaveDialog,
                null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_IFileSaveDialog,
                &mut raw as *mut _ as *mut *mut c_void,
            ) != S_OK
            {
                return None;
            }
            let dlg = &*raw;
            dlg.SetOptions(FOS_NOCHANGEDIR | FOS_OVERWRITEPROMPT);
            let title_w = dialog.title.as_deref().map(|s| s.to_wide_string());
            if let Some(ref t) = title_w {
                dlg.SetTitle(t.as_ptr());
            }
            let dir_item = make_shell_item_from_path(dialog.directory.as_ref());
            if let Some(item) = dir_item {
                dlg.SetFolder(item);
                (*item).Release();
            }
            let filename_w = dialog.filename.as_deref().map(|s| s.to_wide_string());
            if let Some(ref f) = filename_w {
                dlg.SetFileName(f.as_ptr());
            }
            let (specs, _storage) = build_com_filters(&dialog.filters);
            if !specs.is_empty() {
                dlg.SetFileTypes(specs.len() as u32, specs.as_ptr());
            }
            let def_ext_w = dialog
                .filters
                .first()
                .and_then(|f| f.extensions.first())
                .map(|e| e.to_wide_string());
            if let Some(ref e) = def_ext_w {
                dlg.SetDefaultExtension(e.as_ptr());
            }
            let hwnd = #[allow(static_mut_refs)]
            FIRST_HWND.unwrap_or(null_mut());
            let path = if dlg.Show(hwnd) == S_OK {
                let mut item: *mut IShellItem = null_mut();
                if dlg.GetResult(&mut item) == S_OK {
                    let p = shell_item_path(item);
                    (*item).Release();
                    p
                } else {
                    None
                }
            } else {
                None
            };
            dlg.Release();
            path
        }
    }
}

#[cfg(feature = "file_dialog")]
fn build_com_filters(
    filters: &[crate::FileDialogFilter],
) -> (Vec<COMDLG_FILTERSPEC>, Vec<(Box<[u16]>, Box<[u16]>)>) {
    let mut storage: Vec<(Box<[u16]>, Box<[u16]>)> = Vec::new();
    let mut specs: Vec<COMDLG_FILTERSPEC> = Vec::new();
    for f in filters {
        let name: Box<[u16]> = f.name.to_wide_string().into_boxed_slice();
        let pattern = f
            .extensions
            .iter()
            .map(|e| format!("*.{e}"))
            .collect::<Vec<_>>()
            .join(";");
        let spec: Box<[u16]> = pattern.to_wide_string().into_boxed_slice();
        specs.push(COMDLG_FILTERSPEC {
            pszName: name.as_ptr(),
            pszSpec: spec.as_ptr(),
        });
        storage.push((name, spec));
    }
    (specs, storage)
}

#[cfg(feature = "file_dialog")]
unsafe fn make_shell_item_from_path(path: Option<&std::path::PathBuf>) -> Option<*mut IShellItem> {
    let path = path?;
    let w = path.to_string_lossy().to_wide_string();
    let mut item: *mut IShellItem = null_mut();
    if unsafe {
        SHCreateItemFromParsingName(
            w.as_ptr(),
            null_mut(),
            &IID_IShellItem,
            &mut item as *mut _ as *mut *mut c_void,
        )
    } == S_OK
    {
        Some(item)
    } else {
        None
    }
}

#[cfg(feature = "file_dialog")]
unsafe fn shell_item_path(item: *mut IShellItem) -> Option<PathBuf> {
    let mut ptr: *mut u16 = null_mut();
    if unsafe { (*item).GetDisplayName(SIGDN_FILESYSPATH, &mut ptr) } == S_OK && !ptr.is_null() {
        let mut len = 0;
        while unsafe { *ptr.add(len) } != 0 {
            len += 1;
        }
        let s = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(ptr, len) });
        unsafe { CoTaskMemFree(ptr as *mut c_void) };
        Some(PathBuf::from(s))
    } else {
        None
    }
}
