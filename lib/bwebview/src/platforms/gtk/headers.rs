/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unused)]

use std::ffi::{c_char, c_void};

// MARK: Libc
pub(crate) const LOCK_EX: i32 = 2;
pub(crate) const LOCK_NB: i32 = 4;
unsafe extern "C" {
    pub(crate) fn flock(fd: i32, op: i32) -> i32;
}

// MARK: GObject
#[repr(C)]
pub(crate) struct GObject([u8; 0]);
pub(crate) const G_CONNECT_DEFAULT: i32 = 0;
#[link(name = "gobject-2.0")]
unsafe extern "C" {
    pub(crate) fn g_object_new(
        object_type: *mut c_void,
        first_property_name: *const c_char,
        ...
    ) -> *mut GObject;
    pub(crate) fn g_object_set(instance: *mut GObject, first_property_name: *const c_char, ...);
    pub(crate) fn g_signal_connect_data(
        instance: *mut GObject,
        detailed_signal: *const c_char,
        c_handler: *const c_void,
        data: *const c_void,
        destroy_data: *const c_void,
        connect_flags: i32,
    );
    pub(crate) fn g_object_unref(object: *mut GObject);
}

// MARK: Glib
#[repr(C)]
pub(crate) struct GError([u8; 0]);
#[repr(C)]
pub(crate) struct GKeyFile([u8; 0]);
#[repr(C)]
pub(crate) struct GSList {
    pub(crate) data: *mut c_void,
    pub(crate) next: *mut GSList,
}
#[link(name = "glib-2.0")]
unsafe extern "C" {
    pub(crate) fn g_error_free(error: *mut GError);
    pub(crate) fn g_key_file_new() -> *mut GKeyFile;
    pub(crate) fn g_key_file_load_from_file(
        key_file: *mut GKeyFile,
        file: *const c_char,
        flags: i32,
        error: *mut *mut GError,
    ) -> bool;
    pub(crate) fn g_key_file_get_integer(
        key_file: *mut GKeyFile,
        group_name: *const c_char,
        key: *const c_char,
        error: *mut *mut c_void,
    ) -> i32;
    pub(crate) fn g_key_file_get_boolean(
        key_file: *mut GKeyFile,
        group_name: *const c_char,
        key: *const c_char,
        error: *mut *mut c_void,
    ) -> bool;
    pub(crate) fn g_key_file_set_integer(
        key_file: *mut GKeyFile,
        group_name: *const c_char,
        key: *const c_char,
        value: i32,
    );
    pub(crate) fn g_key_file_set_boolean(
        key_file: *mut GKeyFile,
        group_name: *const c_char,
        key: *const c_char,
        value: bool,
    );
    pub(crate) fn g_key_file_save_to_file(
        key_file: *mut GKeyFile,
        file: *const c_char,
        error: *mut *mut c_void,
    ) -> bool;
    pub(crate) fn g_key_file_free(key_file: *mut GKeyFile);
    pub(crate) fn g_idle_add(function: extern "C" fn(*mut c_void) -> i32, data: *mut c_void)
    -> u32;
    pub(crate) fn g_free(mem: *mut c_void);
    pub(crate) fn g_slist_free_full(
        list: *mut GSList,
        free_func: unsafe extern "C" fn(*mut c_void),
    );
}

// MARK: GIO
#[repr(C)]
pub(crate) struct GInputStream([u8; 0]);
#[link(name = "gio-2.0")]
unsafe extern "C" {
    pub(crate) fn g_memory_input_stream_new_from_data(
        data: *const c_void,
        len: usize,
        destroy: *const c_void,
    ) -> *mut GInputStream;
    pub(crate) fn g_input_stream_read_all(
        stream: *mut GInputStream,
        buffer: *mut c_void,
        count: usize,
        bytes_read: *mut usize,
        cancellable: *mut c_void,
        error: *mut *mut GError,
    ) -> bool;
}

// MARK: GDK
#[repr(C)]
pub(crate) struct GdkDisplay([u8; 0]);
#[repr(C)]
pub(crate) struct GdkMonitor([u8; 0]);
#[repr(C)]
pub(crate) struct GdkRectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
#[repr(C)]
pub(crate) struct GdkRGBA {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64,
}
#[link(name = "gdk-3")]
unsafe extern "C" {
    pub(crate) fn gdk_display_get_default() -> *mut GdkDisplay;
    pub(crate) fn gdk_display_get_name(display: *mut GdkDisplay) -> *const c_char;
    pub(crate) fn gdk_display_get_n_monitors(display: *mut GdkDisplay) -> i32;
    pub(crate) fn gdk_display_get_monitor(
        display: *mut GdkDisplay,
        monitor_num: i32,
    ) -> *mut GdkMonitor;
    pub(crate) fn gdk_display_get_primary_monitor(display: *mut GdkDisplay) -> *mut GdkMonitor;
    pub(crate) fn gdk_monitor_get_model(monitor: *mut GdkMonitor) -> *const c_char;
    pub(crate) fn gdk_monitor_get_geometry(monitor: *mut GdkMonitor, geometry: *mut GdkRectangle);
    pub(crate) fn gdk_monitor_get_scale_factor(monitor: *mut GdkMonitor) -> i32;
    pub(crate) fn gdk_monitor_is_primary(monitor: *mut GdkMonitor) -> bool;
}

// MARK: GTK
#[repr(C)]
pub(crate) struct GtkApplication([u8; 0]);
#[repr(C)]
pub(crate) struct GtkWidget([u8; 0]);
#[repr(C)]
pub(crate) struct GtkWindow([u8; 0]);
#[repr(C)]
pub(crate) struct GtkSettings([u8; 0]);
pub(crate) const GTK_WINDOW_TOPLEVEL: i32 = 0;
pub(crate) const GTK_WIN_POS_CENTER: i32 = 1;
pub(crate) const GTK_STATE_FLAG_NORMAL: i32 = 0;
#[link(name = "gtk-3")]
unsafe extern "C" {
    pub(crate) fn gtk_init(argc: *mut i32, argv: *mut *mut *mut c_char);
    pub(crate) fn gtk_main();
    pub(crate) fn gtk_main_quit();
    pub(crate) fn gtk_window_new(r#type: i32) -> *mut GtkWindow;
    pub(crate) fn gtk_widget_set_size_request(widget: *mut GtkWidget, width: i32, height: i32);
    pub(crate) fn gtk_window_fullscreen(window: *mut GtkWindow);
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
    pub(crate) fn gtk_widget_show_all(window: *mut GtkWindow);
    pub(crate) fn gtk_show_uri_on_window(
        parent: *mut GtkWindow,
        uri: *const c_char,
        timestamp: u32,
        error: *mut *mut GError,
    );
    pub(crate) fn gtk_settings_get_default() -> *mut GtkSettings;
    pub(crate) fn gtk_widget_override_background_color(
        widget: *mut GtkWidget,
        state: i32,
        color: *const GdkRGBA,
    );
}

// MARK: GTK File Chooser
pub(crate) const GTK_FILE_CHOOSER_ACTION_OPEN: i32 = 0;
pub(crate) const GTK_FILE_CHOOSER_ACTION_SAVE: i32 = 1;
pub(crate) const GTK_RESPONSE_ACCEPT: i32 = -3;

#[repr(C)]
pub(crate) struct GtkFileChooserNative([u8; 0]);
#[repr(C)]
pub(crate) struct GtkNativeDialog([u8; 0]);
#[repr(C)]
pub(crate) struct GtkFileFilter([u8; 0]);

#[link(name = "gtk-3")]
unsafe extern "C" {
    pub(crate) fn gtk_file_chooser_native_new(
        title: *const c_char,
        parent: *mut GtkWindow,
        action: i32,
        accept_label: *const c_char,
        cancel_label: *const c_char,
    ) -> *mut GtkFileChooserNative;
    pub(crate) fn gtk_native_dialog_run(dialog: *mut GtkNativeDialog) -> i32;
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
}

// MARK: Soup
#[repr(C)]
pub(crate) struct SoupMessageHeaders([u8; 0]);
pub(crate) const SOUP_MESSAGE_HEADERS_RESPONSE: i32 = 1;
#[link(name = "soup-3.0")]
unsafe extern "C" {
    pub(crate) fn soup_message_headers_new(r#type: i32) -> *mut SoupMessageHeaders;
    pub(crate) fn soup_message_headers_foreach(
        headers: *mut SoupMessageHeaders,
        func: extern "C" fn(name: *const c_char, value: *const c_char, user_data: *mut c_void),
        user_data: *mut c_void,
    );
    pub(crate) fn soup_message_headers_append(
        headers: *mut SoupMessageHeaders,
        name: *const c_char,
        value: *const c_char,
    );
}

// MARK: WebKitGtk
#[repr(C)]
pub(crate) struct WebKitWebContext([u8; 0]);
#[repr(C)]
pub(crate) struct WebKitWebView([u8; 0]);
#[repr(C)]
pub(crate) struct WebkitSettings([u8; 0]);
#[repr(C)]
pub(crate) struct WebKitNavigationPolicyDecision([u8; 0]);
#[repr(C)]
pub(crate) struct WebKitURIRequest([u8; 0]);
#[repr(C)]
pub(crate) struct WebKitUserContentManager([u8; 0]);
#[repr(C)]
pub(crate) struct WebKitUserScript([u8; 0]);
#[repr(C)]
pub(crate) struct WebKitJavascriptResult([u8; 0]);
#[repr(C)]
pub(crate) struct WebKitURISchemeResponse([u8; 0]);
#[repr(C)]
pub(crate) struct WebKitURISchemeRequest([u8; 0]);
pub(crate) const WEBKIT_LOAD_STARTED: i32 = 1;
pub(crate) const WEBKIT_LOAD_FINISHED: i32 = 3;
pub(crate) const WEBKIT_POLICY_DECISION_TYPE_NEW_WINDOW_ACTION: i32 = 1;
pub(crate) const WEBKIT_USER_CONTENT_INJECT_TOP_FRAME: i32 = 1;
pub(crate) const WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START: i32 = 0;
pub(crate) const WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_END: i32 = 1;
#[link(name = "webkit2gtk-4.1")]
unsafe extern "C" {
    pub(crate) fn webkit_web_context_get_default() -> *mut WebKitWebContext;
    pub(crate) fn webkit_web_context_register_uri_scheme(
        context: *mut WebKitWebContext,
        scheme: *const c_char,
        callback: *const c_void,
        user_data: *mut c_void,
        user_data_destroy_func: *const c_void,
    );

    pub(crate) fn webkit_uri_scheme_response_new(
        input_stream: *mut GInputStream,
        stream_length: i64,
    ) -> *mut WebKitURISchemeResponse;
    pub(crate) fn webkit_uri_scheme_response_set_status(
        response: *mut WebKitURISchemeResponse,
        status_code: u32,
        reason_phrase: *const c_char,
    );
    pub(crate) fn webkit_uri_scheme_response_set_http_headers(
        response: *mut WebKitURISchemeResponse,
        headers: *mut SoupMessageHeaders,
    );
    pub(crate) fn webkit_uri_scheme_response_set_content_type(
        response: *mut WebKitURISchemeResponse,
        content_type: *const c_char,
    );

    pub(crate) fn webkit_uri_scheme_request_get_http_method(
        request: *mut WebKitURISchemeRequest,
    ) -> *const c_char;
    pub(crate) fn webkit_uri_scheme_request_get_uri(
        request: *mut WebKitURISchemeRequest,
    ) -> *const c_char;
    pub(crate) fn webkit_uri_scheme_request_get_http_headers(
        request: *mut WebKitURISchemeRequest,
    ) -> *mut SoupMessageHeaders;
    pub(crate) fn webkit_uri_scheme_request_get_http_body(
        request: *mut WebKitURISchemeRequest,
    ) -> *mut GInputStream;
    pub(crate) fn webkit_uri_scheme_request_finish_with_response(
        request: *mut WebKitURISchemeRequest,
        response: *mut WebKitURISchemeResponse,
    );

    pub(crate) fn webkit_web_view_get_type() -> *mut c_void;
    pub(crate) fn webkit_web_view_get_settings(web_view: *mut WebKitWebView)
    -> *mut WebkitSettings;
    pub(crate) fn webkit_web_view_load_uri(web_view: *mut WebKitWebView, uri: *const c_char);
    pub(crate) fn webkit_web_view_load_html(
        web_view: *mut WebKitWebView,
        content: *const c_char,
        base_uri: *const c_char,
    );
    pub(crate) fn webkit_web_view_get_title(web_view: *mut WebKitWebView) -> *mut c_char;
    pub(crate) fn webkit_web_view_get_uri(web_view: *mut WebKitWebView) -> *mut c_char;
    pub(crate) fn webkit_web_view_evaluate_javascript(
        web_view: *mut WebKitWebView,
        script: *const c_char,
        length: usize,
        world_name: *const c_char,
        source_uri: *const c_char,
        cancellable: *const c_void,
        callback: *const c_void,
        user_data: *const c_void,
    );
    pub(crate) fn webkit_navigation_policy_decision_get_request(
        decision: *mut WebKitNavigationPolicyDecision,
    ) -> *mut WebKitURIRequest;
    pub(crate) fn webkit_uri_request_get_uri(request: *mut WebKitURIRequest) -> *const c_char;
    pub(crate) fn webkit_user_content_manager_new() -> *mut WebKitUserContentManager;
    pub(crate) fn webkit_web_view_get_user_content_manager(
        web_view: *mut WebKitWebView,
    ) -> *mut WebKitUserContentManager;
    pub(crate) fn webkit_user_script_new(
        source: *const c_char,
        injected_frames: i32,
        injection_time: i32,
        whitelist: *const *const c_char,
        blacklist: *const *const c_char,
    ) -> *mut WebKitUserScript;
    pub(crate) fn webkit_user_content_manager_add_script(
        manager: *mut WebKitUserContentManager,
        script: *mut WebKitUserScript,
    );
    pub(crate) fn webkit_user_content_manager_register_script_message_handler(
        manager: *mut WebKitUserContentManager,
        name: *const c_char,
    );
    pub(crate) fn webkit_javascript_result_get_js_value(
        result: *mut WebKitJavascriptResult,
    ) -> *mut c_void;
    pub(crate) fn webkit_settings_set_user_agent(
        settings: *mut WebkitSettings,
        user_agent: *const c_char,
    );
    pub(crate) fn webkit_settings_set_enable_developer_extras(
        settings: *mut WebkitSettings,
        enable: bool,
    );
    pub(crate) fn webkit_web_view_set_background_color(
        web_view: *mut WebKitWebView,
        color: *const GdkRGBA,
    );
}

// MARK: JavaScriptCoreGtk
#[link(name = "javascriptcoregtk-4.1")]
unsafe extern "C" {
    pub(crate) fn jsc_value_to_string(value: *mut c_void) -> *const c_char;
}
