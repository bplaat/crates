/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::CString;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::{env, fs};

use super::event_loop::APP_ID;
use super::headers::*;

/// Restores the position, size and maximized state saved by [`save_window_state`]
pub(super) fn load_window_state(window: *mut GtkWindow) {
    unsafe {
        let settings = g_key_file_new();
        let file = CString::new(config_dir().join("settings.ini").display().to_string())
            .expect("Can't convert to CString");
        let mut err = null_mut();
        g_key_file_load_from_file(settings, file.as_ptr(), 0, &mut err);
        if err.is_null() {
            let group = c"window".as_ptr();
            let x = g_key_file_get_integer(settings, group, c"x".as_ptr(), null_mut());
            let y = g_key_file_get_integer(settings, group, c"y".as_ptr(), null_mut());
            gtk_window_move(window, x, y);

            let width = g_key_file_get_integer(settings, group, c"width".as_ptr(), null_mut());
            let height = g_key_file_get_integer(settings, group, c"height".as_ptr(), null_mut());
            gtk_window_set_default_size(window, width, height);

            let maximized =
                g_key_file_get_boolean(settings, group, c"maximized".as_ptr(), null_mut());
            if maximized {
                gtk_window_maximize(window);
            }
        } else {
            g_error_free(err);
        }
        g_key_file_free(settings);
    }
}

/// Writes the window's current position, size and maximized state to the config directory
pub(super) fn save_window_state(window: *mut GtkWindow) {
    fs::create_dir_all(config_dir()).expect("Can't create settings directory");
    let settings_path = config_dir().join("settings.ini");
    unsafe {
        let settings = g_key_file_new();
        let group = c"window".as_ptr();

        let mut x = 0;
        let mut y = 0;
        gtk_window_get_position(window, &mut x, &mut y);
        g_key_file_set_integer(settings, group, c"x".as_ptr(), x);
        g_key_file_set_integer(settings, group, c"y".as_ptr(), y);

        let mut width = 0;
        let mut height = 0;
        gtk_window_get_size(window, &mut width, &mut height);
        g_key_file_set_integer(settings, group, c"width".as_ptr(), width);
        g_key_file_set_integer(settings, group, c"height".as_ptr(), height);

        let maximized = gtk_window_is_maximized(window);
        g_key_file_set_boolean(settings, group, c"maximized".as_ptr(), maximized);

        let file =
            CString::new(settings_path.display().to_string()).expect("Can't convert to CString");
        g_key_file_save_to_file(settings, file.as_ptr(), null_mut());
        g_key_file_free(settings);
    }
}

fn config_dir() -> PathBuf {
    let project_dirs = unsafe {
        if let Some(ref app_id) = APP_ID {
            directories::ProjectDirs::from(
                &app_id.qualifier,
                &app_id.organization,
                &app_id.application,
            )
        } else {
            directories::ProjectDirs::from_path(PathBuf::from(
                env::current_exe()
                    .expect("Can't get current process name")
                    .file_name()
                    .expect("Can't get current process name")
                    .to_string_lossy()
                    .into_owned(),
            ))
        }
    }
    .expect("Can't get dirs");
    project_dirs.config_dir()
}
