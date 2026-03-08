/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::env;
use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr::{null, null_mut};

use super::headers::*;
use super::window::{PlatformWindow, WindowData};
use crate::{InjectionTime, WebviewBuilder, WebviewHandler};

pub(crate) struct PlatformWebview(pub(super) *mut WindowData);

impl PlatformWebview {
    pub(crate) fn new(window: &PlatformWindow) -> Self {
        PlatformWebview(&*window.0 as *const WindowData as *mut WindowData)
    }
}

impl PlatformWebview {
    pub(crate) fn init_webview(&mut self, builder: WebviewBuilder<'_>) {
        let data = unsafe { &mut *self.0 };
        let is_wayland = unsafe {
            CStr::from_ptr(gdk_display_get_name(gdk_display_get_default()))
                .to_string_lossy()
                .contains("wayland")
        };
        let window = data.window;

        // Create webview web context
        let web_context = unsafe {
            let web_context = webkit_web_context_get_default();

            #[cfg(feature = "custom_protocol")]
            for custom_protocol in builder.custom_protocols {
                extern "C" fn custom_protocol_destroy(data: *mut c_void) {
                    drop(unsafe {
                        use crate::CustomProtocol;
                        Box::from_raw(data as *mut CustomProtocol)
                    });
                }
                let scheme =
                    CString::new(custom_protocol.scheme.clone()).expect("Can't convert to CString");
                webkit_web_context_register_uri_scheme(
                    web_context,
                    scheme.as_ptr(),
                    webview_custom_uri_scheme as *const c_void,
                    Box::leak(Box::new(custom_protocol)) as *mut _ as *mut c_void,
                    custom_protocol_destroy as *const c_void,
                );
            }

            web_context
        };

        // Create webview user content manager
        let user_content_manager = unsafe {
            const IPC_SCRIPT: &str = "window.ipc = new EventTarget();\
                window.ipc.postMessage = message => window.webkit.messageHandlers.ipc.postMessage(typeof message !== 'string' ? JSON.stringify(message) : message);";
            #[cfg(feature = "log")]
            const CONSOLE_SCRIPT: &str = "for (const level of ['error', 'warn', 'info', 'debug', 'trace', 'log'])\
                window.console[level] = (...args) => window.webkit.messageHandlers.console.postMessage(level.charAt(0) + args.map(arg => typeof arg !== 'string' ? JSON.stringify(arg) : arg).join(' '));";
            #[cfg(not(feature = "log"))]
            let script = IPC_SCRIPT;
            #[cfg(feature = "log")]
            let script = format!("{IPC_SCRIPT}\n{CONSOLE_SCRIPT}");

            let user_content_manager = webkit_user_content_manager_new();
            let script = CString::new(script).expect("Can't convert to CString");
            let user_script = webkit_user_script_new(
                script.as_ptr(),
                WEBKIT_USER_CONTENT_INJECT_TOP_FRAME,
                WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START,
                null(),
                null(),
            );
            webkit_user_content_manager_add_script(user_content_manager, user_script);
            g_signal_connect_data(
                user_content_manager as *mut GObject,
                c"script-message-received::ipc".as_ptr(),
                webview_on_message_ipc as *const c_void,
                self.0 as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            webkit_user_content_manager_register_script_message_handler(
                user_content_manager,
                c"ipc".as_ptr(),
            );
            #[cfg(feature = "log")]
            {
                g_signal_connect_data(
                    user_content_manager as *mut GObject,
                    c"script-message-received::console".as_ptr(),
                    webview_on_message_console as *const c_void,
                    self.0 as *const c_void,
                    null(),
                    G_CONNECT_DEFAULT,
                );
                webkit_user_content_manager_register_script_message_handler(
                    user_content_manager,
                    c"console".as_ptr(),
                );
            }

            user_content_manager
        };

        // Create webview
        let webview = unsafe {
            let webview = g_object_new(
                webkit_web_view_get_type(),
                c"web-context".as_ptr(),
                web_context,
                c"user-content-manager".as_ptr(),
                user_content_manager,
                null::<c_void>(),
            ) as *mut WebKitWebView;
            gtk_container_add(window as *mut GtkWidget, webview as *mut GtkWidget);
            if data.background_color.is_some() {
                let rgba = GdkRGBA {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 0.0,
                };
                webkit_web_view_set_background_color(webview, &rgba);
            }

            let useragent = CString::new(format!(
                "Mozilla/5.0 ({}; {} {}) bwebview/{}",
                if is_wayland { "Wayland" } else { "X11" },
                if env::consts::OS == "linux" {
                    "Linux"
                } else {
                    env::consts::OS
                },
                env::consts::ARCH,
                env!("CARGO_PKG_VERSION"),
            ))
            .expect("Can't convert to CString");
            let webview_settings = webkit_web_view_get_settings(webview);
            webkit_settings_set_user_agent(webview_settings, useragent.as_ptr());
            if cfg!(debug_assertions) {
                webkit_settings_set_enable_developer_extras(webview_settings, true);
            }

            if let Some(should_load_url) = builder.should_load_url {
                let url = CString::new(should_load_url).expect("Can't convert to CString");
                webkit_web_view_load_uri(webview, url.as_ptr());
            }
            if let Some(should_load_html) = builder.should_load_html {
                let html = CString::new(should_load_html).expect("Can't convert to CString");
                webkit_web_view_load_html(webview, html.as_ptr(), null());
            }

            g_signal_connect_data(
                webview as *mut GObject,
                c"load-changed".as_ptr(),
                webview_on_load_changed as *const c_void,
                self.0 as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                webview as *mut GObject,
                c"notify::title".as_ptr(),
                webview_on_title_changed as *const c_void,
                self.0 as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                webview as *mut GObject,
                c"decide-policy".as_ptr(),
                webview_on_navigation_policy_decision as *const c_void,
                self.0 as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            webview
        };

        data.webview = webview;
        data.webview_handler = builder.webview_handler;

        // Show window
        unsafe { gtk_widget_show_all(data.window as *mut GtkWidget) };
    }
}

fn call_webview_handler<F>(data: &mut WindowData, f: F)
where
    F: FnOnce(&mut dyn crate::WebviewHandler, &mut crate::Webview),
{
    if let Some(h_ptr) = data.webview_handler {
        let handler = unsafe { &mut *h_ptr };
        let window_id = data.window_id;
        let webview_handler = data.webview_handler;
        let mut wv = unsafe {
            crate::Webview::from_raw(window_id, PlatformWebview(data as *mut WindowData), webview_handler)
        };
        f(handler, &mut *wv);
    }
}

impl crate::WebviewInterface for PlatformWebview {
    fn url(&self) -> Option<String> {
        unsafe {
            let url = webkit_web_view_get_uri((*self.0).webview);
            if !url.is_null() {
                Some(CStr::from_ptr(url).to_string_lossy().into_owned())
            } else {
                None
            }
        }
    }

    fn load_url(&mut self, url: impl AsRef<str>) {
        let url = CString::new(url.as_ref()).expect("Can't convert to CString");
        unsafe { webkit_web_view_load_uri((*self.0).webview, url.as_ptr()) }
    }

    fn load_html(&mut self, html: impl AsRef<str>) {
        let html = CString::new(html.as_ref()).expect("Can't convert to CString");
        unsafe { webkit_web_view_load_html((*self.0).webview, html.as_ptr(), null()) }
    }

    fn evaluate_script(&mut self, script: impl AsRef<str>) {
        let script = script.as_ref();
        unsafe {
            webkit_web_view_evaluate_javascript(
                (*self.0).webview,
                script.as_ptr() as *const c_char,
                script.len(),
                null(),
                null(),
                null(),
                null(),
                null(),
            )
        }
    }

    fn add_user_script(&mut self, script: impl AsRef<str>, injection_time: InjectionTime) {
        let script = CString::new(script.as_ref()).expect("Can't convert to CString");
        unsafe {
            let user_content_manager = webkit_web_view_get_user_content_manager((*self.0).webview);
            let user_script = webkit_user_script_new(
                script.as_ptr(),
                WEBKIT_USER_CONTENT_INJECT_TOP_FRAME,
                match injection_time {
                    InjectionTime::DocumentStart => WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START,
                    InjectionTime::DocumentLoaded => WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_END,
                },
                null(),
                null(),
            );
            webkit_user_content_manager_add_script(user_content_manager, user_script);
        }
    }
}

extern "C" fn webview_on_load_changed(
    _webview: *mut WebKitWebView,
    event: i32,
    _self: &mut WindowData,
) {
    if event == WEBKIT_LOAD_STARTED {
        call_webview_handler(_self, |handler, wv| handler.on_load_start(wv));
    }
    if event == WEBKIT_LOAD_FINISHED {
        call_webview_handler(_self, |handler, wv| handler.on_load(wv));
    }
}

extern "C" fn webview_on_title_changed(
    webview: *mut WebKitWebView,
    _pspec: *const c_void,
    _self: &mut WindowData,
) {
    let title = unsafe { webkit_web_view_get_title(webview) };
    let title = unsafe { CStr::from_ptr(title) }.to_string_lossy().to_string();
    call_webview_handler(_self, |handler, wv| handler.on_title_change(wv, title.clone()));
}

extern "C" fn webview_on_navigation_policy_decision(
    _webview: *mut WebKitWebView,
    decision: *mut WebKitNavigationPolicyDecision,
    decision_type: i32,
    _self: &mut WindowData,
) -> bool {
    if decision_type == WEBKIT_POLICY_DECISION_TYPE_NEW_WINDOW_ACTION {
        let request = unsafe { webkit_navigation_policy_decision_get_request(decision) };
        let uri = unsafe { webkit_uri_request_get_uri(request) };
        unsafe { gtk_show_uri_on_window(null_mut(), uri, 0, null_mut()) };
        return true;
    }
    false
}

extern "C" fn webview_on_message_ipc(
    _manager: *mut WebKitUserContentManager,
    _message: *mut WebKitJavascriptResult,
    _self: &mut WindowData,
) {
    let message = unsafe { webkit_javascript_result_get_js_value(_message) };
    let message = unsafe { jsc_value_to_string(message) };
    let message = unsafe { CStr::from_ptr(message) }.to_string_lossy().to_string();
    call_webview_handler(_self, |handler, wv| handler.on_message(wv, message.clone()));
}

#[cfg(feature = "log")]
extern "C" fn webview_on_message_console(
    _manager: *mut WebKitUserContentManager,
    _message: *mut WebKitJavascriptResult,
    _self: &mut WindowData,
) {
    let message = unsafe { webkit_javascript_result_get_js_value(_message) };
    let message = unsafe { jsc_value_to_string(message) };
    let message = unsafe { CStr::from_ptr(message) }
        .to_string_lossy()
        .to_string();

    let (level, message) = message.split_at(1);
    match level {
        "e" => log::error!("{message}"),
        "w" => log::warn!("{message}"),
        "i" | "l" => log::info!("{message}"),
        "d" => log::debug!("{message}"),
        "t" => log::trace!("{message}"),
        _ => unimplemented!(),
    }
}

#[cfg(feature = "custom_protocol")]
extern "C" fn webview_custom_uri_scheme(
    uri_scheme_request: *mut WebKitURISchemeRequest,
    custom_protocol: *mut crate::CustomProtocol,
) {
    let custom_protocol = unsafe { &mut *custom_protocol };

    let req = webkit_uri_scheme_request_to_http_request(uri_scheme_request);
    let res = (custom_protocol.handler)(&req);

    let uri_scheme_response = http_response_to_webkit_uri_scheme_response(res);
    unsafe {
        webkit_uri_scheme_request_finish_with_response(uri_scheme_request, uri_scheme_response);
        g_object_unref(uri_scheme_response as *mut GObject);
    };
}

#[cfg(feature = "custom_protocol")]
fn webkit_uri_scheme_request_to_http_request(
    uri_scheme_request: *mut WebKitURISchemeRequest,
) -> small_http::Request {
    use std::str::FromStr;

    let method = unsafe { webkit_uri_scheme_request_get_http_method(uri_scheme_request) };
    let method = unsafe { CStr::from_ptr(method) }.to_string_lossy();

    let uri = unsafe { webkit_uri_scheme_request_get_uri(uri_scheme_request) };
    let uri = unsafe { CStr::from_ptr(uri) }.to_string_lossy();

    let mut req = small_http::Request::with_method_and_url(
        small_http::Method::from_str(&method).unwrap_or(small_http::Method::Get),
        &uri,
    );

    let headers = unsafe { webkit_uri_scheme_request_get_http_headers(uri_scheme_request) };
    extern "C" fn headers_foreach(
        key: *const c_char,
        value: *const c_char,
        user_data: *mut c_void,
    ) {
        let req = unsafe { &mut *(user_data as *mut small_http::Request) };
        let key = unsafe { CStr::from_ptr(key) }.to_string_lossy();
        let value = unsafe { CStr::from_ptr(value) }.to_string_lossy();
        req.headers.insert(key.to_string(), value.to_string());
    }
    unsafe {
        soup_message_headers_foreach(headers, headers_foreach, &mut req as *mut _ as *mut c_void)
    };

    let body = unsafe { webkit_uri_scheme_request_get_http_body(uri_scheme_request) };
    if !body.is_null() {
        let mut body_data = Vec::new();
        let mut bytes_read = 0;
        let mut buffer = [0u8; 4096];
        loop {
            let result = unsafe {
                g_input_stream_read_all(
                    body,
                    buffer.as_mut_ptr() as *mut c_void,
                    buffer.len(),
                    &mut bytes_read,
                    null_mut(),
                    null_mut(),
                )
            };
            if result || bytes_read == 0 {
                break;
            }
            body_data.extend_from_slice(&buffer[..bytes_read]);
            if bytes_read < buffer.len() {
                break;
            }
        }
        req = req.body(body_data);
    }

    req
}

#[cfg(feature = "custom_protocol")]
fn http_response_to_webkit_uri_scheme_response(
    res: small_http::Response,
) -> *mut WebKitURISchemeResponse {
    extern "C" fn body_data_destroy(data: *mut c_void) {
        drop(unsafe { Box::from_raw(data as *mut u8) });
    }
    let stream = unsafe {
        g_memory_input_stream_new_from_data(
            Box::into_raw(res.body.clone().into_boxed_slice()) as *const c_void,
            res.body.len(),
            body_data_destroy as *const c_void,
        )
    };

    let uri_scheme_response =
        unsafe { webkit_uri_scheme_response_new(stream, res.body.len() as i64) };
    unsafe {
        webkit_uri_scheme_response_set_status(uri_scheme_response, res.status as u32, null())
    };

    let headers = unsafe {
        let headers = soup_message_headers_new(SOUP_MESSAGE_HEADERS_RESPONSE);
        for (key, value) in &res.headers {
            let key = CString::new(key.as_str()).expect("Can't convert to CString");
            let value = CString::new(value.as_str()).expect("Can't convert to CString");
            soup_message_headers_append(headers, key.as_ptr(), value.as_ptr());
        }
        headers
    };
    unsafe { webkit_uri_scheme_response_set_http_headers(uri_scheme_response, headers) };

    if let Some(content_type) = res.headers.get("Content-Type") {
        let content_type = CString::new(content_type).expect("Can't convert to CString");
        unsafe {
            webkit_uri_scheme_response_set_content_type(uri_scheme_response, content_type.as_ptr())
        };
    }

    uri_scheme_response
}
