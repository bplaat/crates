/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unused)]

use std::ffi::{c_char, c_void};

use super::gdk::{GdkRGBA, GdkScreen, GdkWindow};
use super::glib::{GError, GSList};

// MARK: GTK
#[repr(C)]
pub(crate) struct GtkApplication([u8; 0]);
#[repr(C)]
pub(crate) struct GtkWidget([u8; 0]);
#[repr(C)]
pub(crate) struct GtkWindow([u8; 0]);
#[repr(C)]
pub(crate) struct GtkSettings([u8; 0]);
#[repr(C)]
pub(crate) struct GtkSelectionData([u8; 0]);
#[repr(C)]
pub(crate) struct GtkDialog([u8; 0]);
pub(crate) const GTK_WINDOW_TOPLEVEL: i32 = 0;
pub(crate) const GTK_WIN_POS_CENTER: i32 = 1;
pub(crate) const GTK_STATE_FLAG_NORMAL: i32 = 0;
pub(crate) const GTK_DIALOG_MODAL: i32 = 1;
pub(crate) const GTK_DIALOG_DESTROY_WITH_PARENT: i32 = 2;
pub(crate) const GTK_MESSAGE_INFO: i32 = 0;
pub(crate) const GTK_MESSAGE_WARNING: i32 = 1;
pub(crate) const GTK_MESSAGE_ERROR: i32 = 3;
pub(crate) const GTK_BUTTONS_NONE: i32 = 0;
pub(crate) const GDK_POINTER_MOTION_MASK: i32 = 1 << 2;
pub(crate) const GDK_BUTTON_PRESS_MASK: i32 = 1 << 8;
pub(crate) const GDK_BUTTON_RELEASE_MASK: i32 = 1 << 9;
pub(crate) const GDK_ENTER_NOTIFY_MASK: i32 = 1 << 12;
pub(crate) const GDK_LEAVE_NOTIFY_MASK: i32 = 1 << 13;
pub(crate) const GDK_SCROLL_MASK: i32 = 1 << 21;
pub(crate) const GDK_SMOOTH_SCROLL_MASK: i32 = 1 << 23;
unsafe extern "C" {
    pub(crate) fn gtk_widget_get_window(widget: *mut GtkWidget) -> *mut GdkWindow;
    pub(crate) fn gtk_widget_add_events(widget: *mut GtkWidget, events: i32);
    pub(crate) fn gtk_init(argc: *mut i32, argv: *mut *mut *mut c_char);
    pub(crate) fn gtk_main();
    pub(crate) fn gtk_main_quit();
    pub(crate) fn gtk_window_new(r#type: i32) -> *mut GtkWindow;
    pub(crate) fn gtk_window_set_default_icon_name(name: *const c_char) -> bool;
    pub(crate) fn gtk_widget_set_size_request(widget: *mut GtkWidget, width: i32, height: i32);
    pub(crate) fn gtk_window_fullscreen(window: *mut GtkWindow);
    pub(crate) fn gtk_window_unfullscreen(window: *mut GtkWindow);
    pub(crate) fn gtk_container_add(container: *mut GtkWidget, widget: *mut GtkWidget);
    pub(crate) fn gtk_window_get_position(window: *mut GtkWindow, x: *mut i32, y: *mut i32);
    pub(crate) fn gtk_window_set_title(window: *mut GtkWindow, title: *const c_char);
    pub(crate) fn gtk_window_set_position(window: *mut GtkWindow, position: i32);
    pub(crate) fn gtk_window_move(window: *mut GtkWindow, x: i32, y: i32);
    pub(crate) fn gtk_window_get_size(window: *mut GtkWindow, width: *mut i32, height: *mut i32);
    pub(crate) fn gtk_window_set_default_size(window: *mut GtkWindow, width: i32, height: i32);
    pub(crate) fn gtk_window_set_resizable(window: *mut GtkWindow, resizable: bool);
    pub(crate) fn gtk_window_is_maximized(window: *mut GtkWindow) -> bool;
    pub(crate) fn gtk_window_maximize(window: *mut GtkWindow);
    pub(crate) fn gtk_widget_show(widget: *mut GtkWidget);
    pub(crate) fn gtk_widget_hide(widget: *mut GtkWidget);
    pub(crate) fn gtk_widget_show_all(window: *mut GtkWidget);
    pub(crate) fn gtk_settings_get_default() -> *mut GtkSettings;
    pub(crate) fn gtk_widget_override_background_color(
        widget: *mut GtkWidget,
        state: i32,
        color: *const GdkRGBA,
    );
    pub(crate) fn gtk_drag_finish(
        context: *mut super::gdk::GdkDragContext,
        success: i32,
        delete: i32,
        time: u32,
    );
    pub(crate) fn gtk_selection_data_get_uris(
        selection_data: *const GtkSelectionData,
    ) -> *mut *mut c_char;

    // GTK < 3.22
    pub(crate) fn gtk_show_uri(
        screen: *mut GdkScreen,
        uri: *const c_char,
        timestamp: u32,
        error: *mut *mut GError,
    ) -> bool;

    // GTK 3.22+
    pub(crate) fn gtk_show_uri_on_window(
        parent: *mut GtkWindow,
        uri: *const c_char,
        timestamp: u32,
        error: *mut *mut GError,
    );
}

// MARK: GTK Message Dialog
unsafe extern "C" {
    pub(crate) fn gtk_message_dialog_new(
        parent: *mut GtkWindow,
        flags: i32,
        r#type: i32,
        buttons: i32,
        message_format: *const c_char,
        ...
    ) -> *mut GtkWidget;
    pub(crate) fn gtk_dialog_add_button(
        dialog: *mut GtkDialog,
        button_text: *const c_char,
        response_id: i32,
    ) -> *mut GtkWidget;
    pub(crate) fn gtk_dialog_set_default_response(dialog: *mut GtkDialog, response_id: i32);
}

// MARK: GTK File Chooser
pub(crate) const GTK_FILE_CHOOSER_ACTION_OPEN: i32 = 0;
pub(crate) const GTK_FILE_CHOOSER_ACTION_SAVE: i32 = 1;
pub(crate) const GTK_RESPONSE_CANCEL: i32 = -6;
pub(crate) const GTK_RESPONSE_ACCEPT: i32 = -3;
#[repr(C)]
pub(crate) struct GtkFileFilter([u8; 0]);

// GTK 3.20+
#[repr(C)]
pub(crate) struct GtkFileChooserNative([u8; 0]);
#[repr(C)]
pub(crate) struct GtkNativeDialog([u8; 0]);

unsafe extern "C" {
    pub(crate) fn gtk_file_chooser_set_select_multiple(chooser: *mut c_void, select_multiple: bool);
    pub(crate) fn gtk_file_chooser_set_current_folder(
        chooser: *mut c_void,
        folder: *const c_char,
    ) -> bool;
    pub(crate) fn gtk_file_chooser_set_current_name(chooser: *mut c_void, name: *const c_char);
    pub(crate) fn gtk_file_chooser_get_filename(chooser: *mut c_void) -> *mut c_char;
    pub(crate) fn gtk_file_chooser_get_filenames(chooser: *mut c_void) -> *mut GSList;
    pub(crate) fn gtk_file_filter_new() -> *mut GtkFileFilter;
    pub(crate) fn gtk_file_filter_set_name(filter: *mut GtkFileFilter, name: *const c_char);
    pub(crate) fn gtk_file_filter_add_pattern(filter: *mut GtkFileFilter, pattern: *const c_char);
    pub(crate) fn gtk_file_chooser_add_filter(chooser: *mut c_void, filter: *mut GtkFileFilter);

    // GTK < 3.20
    pub(crate) fn gtk_file_chooser_dialog_new(
        title: *const c_char,
        parent: *mut GtkWindow,
        action: i32,
        first_button_text: *const c_char,
        ...
    ) -> *mut GtkWidget;
    pub(crate) fn gtk_dialog_run(dialog: *mut GtkDialog) -> i32;
    pub(crate) fn gtk_widget_destroy(widget: *mut GtkWidget);

    // GTK 3.20+
    pub(crate) fn gtk_file_chooser_native_new(
        title: *const c_char,
        parent: *mut GtkWindow,
        action: i32,
        accept_label: *const c_char,
        cancel_label: *const c_char,
    ) -> *mut GtkFileChooserNative;
    pub(crate) fn gtk_native_dialog_run(dialog: *mut GtkNativeDialog) -> i32;
}
