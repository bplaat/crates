/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unused)]

use std::ffi::{c_char, c_void};

// MARK: GObject
pub(crate) const G_CONNECT_DEFAULT: i32 = 0;
#[link(name = "gobject-2.0")]
unsafe extern "C" {
    pub(crate) fn g_object_set(instance: *mut c_void, first_property_name: *const c_char, ...);
    pub(crate) fn g_signal_connect_data(
        instance: *mut c_void,
        detailed_signal: *const c_char,
        c_handler: *const c_void,
        data: *const c_void,
        destroy_data: *const c_void,
        connect_flags: i32,
    );
}

// MARK: Glib
#[repr(C)]
pub(crate) struct GError(u8);
#[repr(C)]
pub(crate) struct GKeyFile(u8);
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
}

// MARK: GDK
#[repr(C)]
pub(crate) struct GdkDisplay(u8);
#[repr(C)]
pub(crate) struct GdkMonitor(u8);
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
pub(crate) struct GtkWidget(u8);
#[repr(C)]
pub(crate) struct GtkWindow(u8);
#[repr(C)]
pub(crate) struct GtkSettings(u8);
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

// MARK: WebKitGtk
#[repr(C)]
pub(crate) struct WebKitWebView(u8);
#[repr(C)]
pub(crate) struct WebkitSettings(u8);
#[repr(C)]
pub(crate) struct WebKitNavigationPolicyDecision(u8);
#[repr(C)]
pub(crate) struct WebKitURIRequest(u8);
#[repr(C)]
pub(crate) struct WebKitUserContentManager(u8);
#[repr(C)]
pub(crate) struct WebKitUserScript(u8);
#[repr(C)]
pub(crate) struct WebKitJavascriptResult(u8);
pub(crate) const WEBKIT_LOAD_STARTED: i32 = 1;
pub(crate) const WEBKIT_LOAD_FINISHED: i32 = 3;
pub(crate) const WEBKIT_POLICY_DECISION_TYPE_NEW_WINDOW_ACTION: i32 = 1;
pub(crate) const WEBKIT_USER_CONTENT_INJECT_TOP_FRAME: i32 = 1;
pub(crate) const WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START: i32 = 0;
#[link(name = "webkit2gtk-4.1")]
unsafe extern "C" {
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
    pub(crate) fn webkit_web_view_new_with_user_content_manager(
        manager: *mut WebKitUserContentManager,
    ) -> *mut WebKitWebView;
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
