/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unused)]

use std::ffi::{c_char, c_void};

use super::gdk::GdkRGBA;
use super::glib::GInputStream;

// MARK: Soup
#[repr(C)]
pub(crate) struct SoupMessageHeaders([u8; 0]);
pub(crate) const SOUP_MESSAGE_HEADERS_RESPONSE: i32 = 1;
// webkit2gtk-4.1 uses libsoup-3.0; webkit2gtk-4.0 uses libsoup-2.4.
#[cfg_attr(webkit2gtk_4_1, link(name = "soup-3.0"))]
#[cfg_attr(webkit2gtk_4_0, link(name = "soup-2.4"))]
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

// MARK: WebKitGTK
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
#[cfg_attr(webkit2gtk_4_1, link(name = "webkit2gtk-4.1"))]
#[cfg_attr(webkit2gtk_4_0, link(name = "webkit2gtk-4.0"))]
unsafe extern "C" {
    pub(crate) fn webkit_web_context_get_default() -> *mut WebKitWebContext;
    pub(crate) fn webkit_web_context_register_uri_scheme(
        context: *mut WebKitWebContext,
        scheme: *const c_char,
        callback: *const c_void,
        user_data: *mut c_void,
        user_data_destroy_func: *const c_void,
    );

    // webkit2gtk-4.1: full URI scheme response with status, headers, content-type
    #[cfg(webkit2gtk_4_1)]
    pub(crate) fn webkit_uri_scheme_response_new(
        input_stream: *mut GInputStream,
        stream_length: i64,
    ) -> *mut WebKitURISchemeResponse;
    #[cfg(webkit2gtk_4_1)]
    pub(crate) fn webkit_uri_scheme_response_set_status(
        response: *mut WebKitURISchemeResponse,
        status_code: u32,
        reason_phrase: *const c_char,
    );
    #[cfg(webkit2gtk_4_1)]
    pub(crate) fn webkit_uri_scheme_response_set_http_headers(
        response: *mut WebKitURISchemeResponse,
        headers: *mut SoupMessageHeaders,
    );
    #[cfg(webkit2gtk_4_1)]
    pub(crate) fn webkit_uri_scheme_response_set_content_type(
        response: *mut WebKitURISchemeResponse,
        content_type: *const c_char,
    );

    pub(crate) fn webkit_uri_scheme_request_get_uri(
        request: *mut WebKitURISchemeRequest,
    ) -> *const c_char;
    // webkit2gtk-4.1: method, headers and body available on URI scheme requests
    #[cfg(webkit2gtk_4_1)]
    pub(crate) fn webkit_uri_scheme_request_get_http_method(
        request: *mut WebKitURISchemeRequest,
    ) -> *const c_char;
    #[cfg(webkit2gtk_4_1)]
    pub(crate) fn webkit_uri_scheme_request_get_http_headers(
        request: *mut WebKitURISchemeRequest,
    ) -> *mut SoupMessageHeaders;
    #[cfg(webkit2gtk_4_1)]
    pub(crate) fn webkit_uri_scheme_request_get_http_body(
        request: *mut WebKitURISchemeRequest,
    ) -> *mut GInputStream;
    #[cfg(webkit2gtk_4_1)]
    pub(crate) fn webkit_uri_scheme_request_finish_with_response(
        request: *mut WebKitURISchemeRequest,
        response: *mut WebKitURISchemeResponse,
    );
    // webkit2gtk-4.0: only URI is available; finish with stream + content-type
    #[cfg(webkit2gtk_4_0)]
    pub(crate) fn webkit_uri_scheme_request_finish(
        request: *mut WebKitURISchemeRequest,
        stream: *mut GInputStream,
        stream_length: i64,
        content_type: *const c_char,
    );

    pub(crate) fn webkit_web_view_get_type() -> *mut c_void;
    pub(crate) fn webkit_web_view_get_settings(
        web_view: *mut WebKitWebView,
    ) -> *mut WebkitSettings;
    pub(crate) fn webkit_web_view_load_uri(web_view: *mut WebKitWebView, uri: *const c_char);
    pub(crate) fn webkit_web_view_load_html(
        web_view: *mut WebKitWebView,
        content: *const c_char,
        base_uri: *const c_char,
    );
    pub(crate) fn webkit_web_view_get_title(web_view: *mut WebKitWebView) -> *mut c_char;
    pub(crate) fn webkit_web_view_get_uri(web_view: *mut WebKitWebView) -> *mut c_char;
    // webkit2gtk-4.1: evaluate_javascript replaces run_javascript
    #[cfg(webkit2gtk_4_1)]
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
    #[cfg(webkit2gtk_4_0)]
    pub(crate) fn webkit_web_view_run_javascript(
        web_view: *mut WebKitWebView,
        script: *const c_char,
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
    // JSC GLib API available since webkit2gtk-4.0 2.22; the 4.1 API always has it.
    #[cfg(any(webkit2gtk_4_1, webkit2gtk_4_0_jsc_glib))]
    pub(crate) fn webkit_javascript_result_get_js_value(
        result: *mut WebKitJavascriptResult,
    ) -> *mut c_void;
    // Legacy JavaScriptCore C API (webkit2gtk-4.0 < 2.22)
    #[cfg(all(webkit2gtk_4_0, not(webkit2gtk_4_0_jsc_glib)))]
    pub(crate) fn webkit_javascript_result_get_global_context(
        result: *mut WebKitJavascriptResult,
    ) -> *mut c_void;
    #[cfg(all(webkit2gtk_4_0, not(webkit2gtk_4_0_jsc_glib)))]
    pub(crate) fn webkit_javascript_result_get_value(
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

// MARK: JavaScriptCore GLib API (webkit2gtk-4.1 and webkit2gtk-4.0 >= 2.22)
#[cfg(any(webkit2gtk_4_1, webkit2gtk_4_0_jsc_glib))]
#[cfg_attr(webkit2gtk_4_1, link(name = "javascriptcoregtk-4.1"))]
#[cfg_attr(webkit2gtk_4_0_jsc_glib, link(name = "javascriptcoregtk-4.0"))]
unsafe extern "C" {
    pub(crate) fn jsc_value_to_string(value: *mut c_void) -> *const c_char;
}

// MARK: JavaScriptCore C API (webkit2gtk-4.0 < 2.22)
#[cfg(all(webkit2gtk_4_0, not(webkit2gtk_4_0_jsc_glib)))]
#[link(name = "javascriptcoregtk-4.0")]
unsafe extern "C" {
    pub(crate) fn JSValueToStringCopy(
        ctx: *mut c_void,
        value: *mut c_void,
        exception: *mut c_void,
    ) -> *mut c_void;
    pub(crate) fn JSStringGetMaximumUTF8CStringSize(string: *mut c_void) -> usize;
    pub(crate) fn JSStringGetUTF8CString(
        string: *mut c_void,
        buffer: *mut c_char,
        buffer_size: usize,
    ) -> usize;
    pub(crate) fn JSStringRelease(string: *mut c_void);
}
