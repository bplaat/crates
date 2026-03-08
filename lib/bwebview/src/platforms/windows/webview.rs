/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CString, c_void};
use std::ptr::{null, null_mut};
use std::{env, mem};

use super::webview2::*;
use super::win32::*;
use super::window::{PlatformWindow, WindowData, config_dir};
#[cfg(feature = "custom_protocol")]
use crate::CustomProtocol;
use crate::{InjectionTime, WebviewBuilder, WebviewHandler, WindowId};

pub(super) struct WebviewData {
    pub(super) window_id: WindowId,
    pub(super) hwnd: HWND,
    pub(super) background_color: Option<u32>,
    pub(super) should_load_url: Option<String>,
    pub(super) should_load_html: Option<String>,
    #[cfg(feature = "custom_protocol")]
    pub(super) custom_protocols: Vec<CustomProtocol>,
    pub(super) environment: Option<*mut ICoreWebView2Environment>,
    pub(super) webview: Option<*mut ICoreWebView2>,
    pub(super) window_data: *mut WindowData,
    pub(super) webview_handler: Option<*mut dyn WebviewHandler>,
}

pub(crate) struct PlatformWebview {
    pub(super) webview_data: Box<WebviewData>,
}

impl PlatformWebview {
    pub(crate) fn new(window: &PlatformWindow) -> Self {
        let window_data = &*window.0 as *const WindowData as *mut WindowData;
        let webview_data = Box::new(WebviewData {
            window_id: window.0.window_id,
            hwnd: window.0.hwnd,
            background_color: window.0.background_color,
            should_load_url: None,
            should_load_html: None,
            #[cfg(feature = "custom_protocol")]
            custom_protocols: Vec::new(),
            environment: None,
            webview: None,
            window_data,
        });
        PlatformWebview { webview_data }
    }
}

impl PlatformWebview {
    pub(crate) fn init_webview(&mut self, builder: WebviewBuilder<'_>) {
        self.webview_data.should_load_url = builder.should_load_url;
        self.webview_data.should_load_html = builder.should_load_html;
        #[cfg(feature = "custom_protocol")]
        {
            self.webview_data.custom_protocols = builder.custom_protocols;
        }

        // Init Webview2 creation
        unsafe {
            if let Some(color) = self.webview_data.background_color {
                env::set_var(
                    "WEBVIEW2_DEFAULT_BACKGROUND_COLOR",
                    format!("0xFF{:06X}", color & 0xFFFFFF),
                );
            }

            static VTBL: ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandlerVtbl =
                ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandlerVtbl {
                    QueryInterface: unimplemented_query_interface,
                    AddRef: unimplemented_add_ref,
                    Release: unimplemented_release,
                    Invoke: environment_created,
                };
            let completed_handler = Box::into_raw(Box::new(
                ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler {
                    lpVtbl: &VTBL,
                    user_data: self.webview_data.as_mut() as *mut WebviewData as *mut _,
                },
            ));
            if CreateCoreWebView2EnvironmentWithOptions(
                null_mut(),
                config_dir().display().to_string().to_wide_string().as_ptr() as *mut _,
                null_mut(),
                completed_handler,
            ) != S_OK
            {
                use super::win32::MessageBoxA;
                MessageBoxA(
                    null_mut(),
                    c"Failed to create WebView2 environment".as_ptr(),
                    c"Error".as_ptr(),
                    MB_OK,
                );
                std::process::exit(1);
            }
        }
    }
}

impl crate::WebviewInterface for PlatformWebview {
    fn url(&self) -> Option<String> {
        unsafe {
            if let Some(webview) = self.webview_data.webview {
                let mut uri = LPWSTR::default();
                (*webview).get_Source(uri.as_mut_ptr());
                Some(uri.to_string())
            } else {
                None
            }
        }
    }

    fn load_url(&mut self, url: impl AsRef<str>) {
        unsafe {
            if let Some(webview) = self.webview_data.webview {
                #[cfg(feature = "custom_protocol")]
                let url = replace_custom_protocol_in_url(
                    url.as_ref(),
                    &self.webview_data.custom_protocols,
                );
                #[cfg(not(feature = "custom_protocol"))]
                let url: &str = url.as_ref();
                (*webview).Navigate(url.to_wide_string().as_ptr() as *mut _);
            }
        }
    }

    fn load_html(&mut self, html: impl AsRef<str>) {
        unsafe {
            if let Some(webview) = self.webview_data.webview {
                (*webview).NavigateToString(html.as_ref().to_wide_string().as_ptr() as *mut _);
            }
        }
    }

    fn evaluate_script(&mut self, script: impl AsRef<str>) {
        unsafe {
            if let Some(webview) = self.webview_data.webview {
                (*webview).ExecuteScript(
                    script.as_ref().to_wide_string().as_ptr() as *mut _,
                    null_mut(),
                );
            }
        }
    }

    fn add_user_script(&mut self, script: impl AsRef<str>, injection_time: InjectionTime) {
        let mut script = script.as_ref().to_string();
        unsafe {
            if let Some(webview) = self.webview_data.webview {
                if let InjectionTime::DocumentLoaded = injection_time {
                    script = format!(
                        "window.addEventListener('DOMContentLoaded', function () {{ {script} }});"
                    );
                }
                (*webview).AddScriptToExecuteOnDocumentCreated(
                    script.to_wide_string().as_ptr() as *mut _,
                    null_mut(),
                );
            }
        }
    }
}


// --- Helper: construct a temporary ManuallyDrop<Webview> from WebviewData ---
unsafe fn make_temp_webview(data: &WebviewData) -> std::mem::ManuallyDrop<crate::Webview> {
    std::mem::ManuallyDrop::new(crate::Webview {
        id: data.window_id,
        platform: PlatformWebview(data.window_data),
        webview_handler: data.webview_handler,
    })
}

extern "system" fn unimplemented_query_interface(
    _this: *mut c_void,
    _riid: *const GUID,
    _ppv_object: *mut *mut c_void,
) -> HRESULT {
    E_NOINTERFACE
}
extern "system" fn unimplemented_add_ref(_this: *mut c_void) -> HRESULT {
    E_NOTIMPL
}
extern "system" fn unimplemented_release(_this: *mut c_void) -> HRESULT {
    E_NOTIMPL
}

extern "system" fn environment_created(
    _this: *mut ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
    _result: HRESULT,
    environment: *mut ICoreWebView2Environment,
) -> HRESULT {
    unsafe {
        let _self = &mut *((*_this).user_data as *mut WebviewData);

        (*environment).AddRef();
        _self.environment = Some(environment);

        static VTBL: ICoreWebView2CreateCoreWebView2ControllerCompletedHandlerVtbl =
            ICoreWebView2CreateCoreWebView2ControllerCompletedHandlerVtbl {
                QueryInterface: unimplemented_query_interface,
                AddRef: unimplemented_add_ref,
                Release: unimplemented_release,
                Invoke: controller_created,
            };
        let creation_completed_handler = Box::into_raw(Box::new(
            ICoreWebView2CreateCoreWebView2ControllerCompletedHandler {
                lpVtbl: &VTBL,
                user_data: (*_this).user_data,
            },
        ));
        (*environment).CreateCoreWebView2Controller(_self.hwnd, creation_completed_handler);

        S_OK
    }
}

extern "system" fn controller_created(
    _this: *mut ICoreWebView2CreateCoreWebView2ControllerCompletedHandler,
    _result: HRESULT,
    controller: *mut ICoreWebView2Controller,
) -> HRESULT {
    unsafe {
        let _self = &mut *((*_this).user_data as *mut WebviewData);
        (*controller).AddRef();
        (*_self.window_data).controller = Some(controller);
        (*_self.window_data).webview_handler = _self.webview_handler;

        // Set bounds
        let mut rect: RECT = mem::zeroed();
        GetClientRect(_self.hwnd, &mut rect);
        (*controller).put_Bounds(rect);

        // Get webview
        let mut webview: *mut ICoreWebView2 = null_mut();
        (*controller).get_CoreWebView2(&mut webview);
        _self.webview = Some(webview);

        // Set transparent background if needed
        if _self.background_color.is_some() {
            let mut controller2: *mut ICoreWebView2Controller2 = null_mut();
            (*controller).QueryInterface(
                &IID_ICoreWebView2Controller2,
                &mut controller2 as *mut _ as *mut *mut c_void,
            );
            (*controller2).put_DefaultBackgroundColor(COREWEBVIEW2_COLOR {
                A: 0,
                R: 0,
                G: 0,
                B: 0,
            });
        }

        // Set user agent
        let useragent = format!(
            "Mozilla/5.0 (Windows NT; {}) bwebview/{}",
            env::consts::ARCH,
            env!("CARGO_PKG_VERSION"),
        );
        let mut settings = null_mut();
        (*webview).get_Settings(&mut settings);

        let mut settings2: *mut ICoreWebView2Settings2 = null_mut();
        (*settings).QueryInterface(
            &IID_ICoreWebView2Settings2,
            &mut settings2 as *mut _ as *mut *mut c_void,
        );
        (*settings2).put_UserAgent(useragent.to_wide_string().as_ptr() as *mut _);

        // Set custom protocols
        #[cfg(feature = "custom_protocol")]
        {
            for custom_protocol in &_self.custom_protocols {
                (*webview).AddWebResourceRequestedFilter(
                    format!("http://{}.localhost/*", custom_protocol.scheme)
                        .to_wide_string()
                        .as_ptr() as *mut _,
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                );
            }

            static VTBL: ICoreWebView2WebResourceRequestedEventHandlerVtbl =
                ICoreWebView2WebResourceRequestedEventHandlerVtbl {
                    QueryInterface: unimplemented_query_interface,
                    AddRef: unimplemented_add_ref,
                    Release: unimplemented_release,
                    Invoke: web_resource_requested,
                };
            let web_resource_requested_handler =
                Box::into_raw(Box::new(ICoreWebView2WebResourceRequestedEventHandler {
                    lpVtbl: &VTBL,
                    user_data: (*_this).user_data,
                }));
            (*webview).add_WebResourceRequested(web_resource_requested_handler, null_mut());
        }

        // Setup event handlers
        {
            static VTBL: ICoreWebView2NavigationStartingEventHandlerVtbl =
                ICoreWebView2NavigationStartingEventHandlerVtbl {
                    QueryInterface: unimplemented_query_interface,
                    AddRef: unimplemented_add_ref,
                    Release: unimplemented_release,
                    Invoke: navigation_starting,
                };
            let navigation_starting_handler =
                Box::into_raw(Box::new(ICoreWebView2NavigationStartingEventHandler {
                    lpVtbl: &VTBL,
                    user_data: (*_this).user_data,
                }));
            (*webview).add_NavigationStarting(navigation_starting_handler, null_mut());
        }
        {
            static VTBL: ICoreWebView2NavigationCompletedEventHandlerVtbl =
                ICoreWebView2NavigationCompletedEventHandlerVtbl {
                    QueryInterface: unimplemented_query_interface,
                    AddRef: unimplemented_add_ref,
                    Release: unimplemented_release,
                    Invoke: navigation_completed,
                };
            let navigation_completed_handler =
                Box::into_raw(Box::new(ICoreWebView2NavigationCompletedEventHandler {
                    lpVtbl: &VTBL,
                    user_data: (*_this).user_data,
                }));
            (*webview).add_NavigationCompleted(navigation_completed_handler, null_mut());
        }
        {
            static VTBL: ICoreWebView2DocumentTitleChangedEventHandlerVtbl =
                ICoreWebView2DocumentTitleChangedEventHandlerVtbl {
                    QueryInterface: unimplemented_query_interface,
                    AddRef: unimplemented_add_ref,
                    Release: unimplemented_release,
                    Invoke: document_title_changed,
                };
            let document_title_changed_handler =
                Box::into_raw(Box::new(ICoreWebView2DocumentTitleChangedEventHandler {
                    lpVtbl: &VTBL,
                    user_data: (*_this).user_data,
                }));
            (*webview).add_DocumentTitleChanged(document_title_changed_handler, null_mut());
        }
        {
            static VTBL: ICoreWebView2NewWindowRequestedEventHandlerVtbl =
                ICoreWebView2NewWindowRequestedEventHandlerVtbl {
                    QueryInterface: unimplemented_query_interface,
                    AddRef: unimplemented_add_ref,
                    Release: unimplemented_release,
                    Invoke: new_window_requested,
                };
            let new_window_requested_handler =
                Box::into_raw(Box::new(ICoreWebView2NewWindowRequestedEventHandler {
                    lpVtbl: &VTBL,
                    user_data: null_mut(),
                }));
            (*webview).add_NewWindowRequested(new_window_requested_handler, null_mut());
        }

        // Setup ipc and console logging
        const IPC_SCRIPT: &str = "window.ipc = new EventTarget();\
            window.ipc.postMessage = message => window.chrome.webview.postMessage('i' + (typeof message !== 'string' ? JSON.stringify(message) : message));";
        #[cfg(feature = "log")]
        const CONSOLE_SCRIPT: &str = "for (const level of ['error', 'warn', 'info', 'debug', 'trace', 'log'])\
            window.console[level] = (...args) => window.chrome.webview.postMessage('c' + level.charAt(0) + args.map(arg => typeof arg !== 'string' ? JSON.stringify(arg) : arg).join(' '));";
        #[cfg(not(feature = "log"))]
        let script = IPC_SCRIPT;
        #[cfg(feature = "log")]
        let script = format!("{IPC_SCRIPT}\n{CONSOLE_SCRIPT}");
        (*webview).AddScriptToExecuteOnDocumentCreated(
            script.to_wide_string().as_ptr() as *mut _,
            null_mut(),
        );

        static VTBL: ICoreWebView2WebMessageReceivedEventHandlerVtbl =
            ICoreWebView2WebMessageReceivedEventHandlerVtbl {
                QueryInterface: unimplemented_query_interface,
                AddRef: unimplemented_add_ref,
                Release: unimplemented_release,
                Invoke: web_message_received,
            };
        let message_received_handler =
            Box::into_raw(Box::new(ICoreWebView2WebMessageReceivedEventHandler {
                lpVtbl: &VTBL,
                user_data: (*_this).user_data,
            }));
        (*webview).add_WebMessageReceived(message_received_handler, null_mut());

        // Load initial contents
        if let Some(url) = &_self.should_load_url {
            #[cfg(feature = "custom_protocol")]
            let url = replace_custom_protocol_in_url(url, &_self.custom_protocols);
            #[cfg(not(feature = "custom_protocol"))]
            let url: &str = url.as_ref();
            (*webview).Navigate(url.to_wide_string().as_ptr() as *mut _);
        }
        if let Some(html) = &_self.should_load_html {
            (*webview).NavigateToString(html.to_wide_string().as_ptr() as *mut _);
        }

        S_OK
    }
}

extern "system" fn navigation_starting(
    _this: *mut ICoreWebView2NavigationStartingEventHandler,
    _sender: *mut ICoreWebView2,
    _args: *mut ICoreWebView2NavigationStartingEventArgs,
) -> HRESULT {
    let _self = unsafe { &*((*_this).user_data as *const WebviewData) };
    if let Some(h_ptr) = _self.webview_handler {
        unsafe {
            let handler = &mut *h_ptr;
            let mut webview = make_temp_webview(_self);
            handler.on_load_start(&mut webview);
            std::mem::forget(webview);
        }
    }
    S_OK
}

extern "system" fn navigation_completed(
    _this: *mut ICoreWebView2NavigationCompletedEventHandler,
    _sender: *mut ICoreWebView2,
    _args: *mut ICoreWebView2NavigationCompletedEventArgs,
) -> HRESULT {
    let _self = unsafe { &*((*_this).user_data as *const WebviewData) };
    if let Some(h_ptr) = _self.webview_handler {
        unsafe {
            let handler = &mut *h_ptr;
            let mut webview = make_temp_webview(_self);
            handler.on_load(&mut webview);
            std::mem::forget(webview);
        }
    }
    S_OK
}

extern "system" fn document_title_changed(
    _this: *mut ICoreWebView2DocumentTitleChangedEventHandler,
    _sender: *mut ICoreWebView2,
    _args: *mut c_void,
) -> HRESULT {
    let _self = unsafe { &*((*_this).user_data as *const WebviewData) };
    if let Some(h_ptr) = _self.webview_handler {
        unsafe {
            let handler = &mut *h_ptr;
            let mut title = LPWSTR::default();
            (*_sender).get_DocumentTitle(title.as_mut_ptr());
            let title_str = title.to_string();
            let mut webview = make_temp_webview(_self);
            handler.on_title_change(&mut webview, title_str);
            std::mem::forget(webview);
        }
    }
    S_OK
}

extern "system" fn new_window_requested(
    _this: *mut ICoreWebView2NewWindowRequestedEventHandler,
    _sender: *mut ICoreWebView2,
    args: *mut ICoreWebView2NewWindowRequestedEventArgs,
) -> HRESULT {
    unsafe {
        (*args).put_Handled(TRUE);
        let mut uri = LPWSTR::default();
        (*args).get_Uri(uri.as_mut_ptr());
        let uri = CString::new(uri.to_string()).expect("Can't convert to CString");
        ShellExecuteA(
            null_mut(),
            c"open".as_ptr(),
            uri.as_ptr(),
            null_mut(),
            null_mut(),
            SW_SHOWNORMAL,
        );
    }
    S_OK
}

extern "system" fn web_message_received(
    _this: *mut ICoreWebView2WebMessageReceivedEventHandler,
    _sender: *mut ICoreWebView2,
    args: *mut ICoreWebView2WebMessageReceivedEventArgs,
) -> HRESULT {
    let _self = unsafe { &*((*_this).user_data as *const WebviewData) };
    let mut message = LPWSTR::default();
    unsafe { (*args).TryGetWebMessageAsString(message.as_mut_ptr()) };
    let message = message.to_string();
    let (r#type, message) = message.split_at(1);

    #[cfg(feature = "log")]
    if r#type == "c" {
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
    if r#type == "i" {
        if let Some(h_ptr) = _self.webview_handler {
            unsafe {
                let handler = &mut *h_ptr;
                let msg_str = message.to_string();
                let mut webview = make_temp_webview(_self);
                handler.on_message(&mut webview, msg_str);
                std::mem::forget(webview);
            }
        }
    }

    S_OK
}

#[cfg(feature = "custom_protocol")]
extern "system" fn web_resource_requested(
    _this: *mut ICoreWebView2WebResourceRequestedEventHandler,
    _sender: *mut ICoreWebView2,
    args: *mut ICoreWebView2WebResourceRequestedEventArgs,
) -> HRESULT {
    let _self = unsafe { &mut *((*_this).user_data as *mut WebviewData) };

    let mut webview2_request = null_mut();
    unsafe { (*args).get_Request(&mut webview2_request) };
    let http_request = webview2_request_to_http_request(webview2_request);
    unsafe { (*webview2_request).Release() };

    for custom_protocol in &_self.custom_protocols {
        if http_request.url.host() == Some(&format!("{}.localhost", &custom_protocol.scheme)) {
            let response = (custom_protocol.handler)(&http_request);

            let webview2_response = http_response_to_webview2_response(
                response,
                _self.environment.expect("Should be some"),
            );
            unsafe { (*args).put_Response(webview2_response) };
            unsafe { (*webview2_response).Release() };

            return S_OK;
        }
    }
    panic!("No handler found for custom protocol");
}

#[cfg(feature = "custom_protocol")]
fn replace_custom_protocol_in_url(url: &str, custom_protocols: &[CustomProtocol]) -> String {
    for custom_protocol in custom_protocols {
        if url.starts_with(&format!("{}://", &custom_protocol.scheme)) {
            return url.replace(
                &format!("{}://", &custom_protocol.scheme),
                &format!("http://{}.localhost/", &custom_protocol.scheme),
            );
        }
    }
    url.to_string()
}

#[cfg(feature = "custom_protocol")]
fn webview2_request_to_http_request(
    request: *mut ICoreWebView2WebResourceRequest,
) -> small_http::Request {
    unsafe {
        use std::str::FromStr;

        let mut method = LPWSTR::default();
        (*request).get_Method(method.as_mut_ptr());
        let method = method.to_string();

        let mut uri = LPWSTR::default();
        (*request).get_Uri(uri.as_mut_ptr());
        let uri = uri.to_string();

        let mut req = small_http::Request::with_method_and_url(
            small_http::Method::from_str(&method).unwrap_or(small_http::Method::Get),
            &uri,
        );
        {
            let mut headers = null_mut();
            (*request).get_Headers(&mut headers);
            let mut iterator = null_mut();
            (*headers).GetIterator(&mut iterator);
            let mut has_current: BOOL = FALSE;
            (*iterator).get_HasCurrentHeader(&mut has_current);
            while has_current == TRUE {
                let mut name = LPWSTR::default();
                let mut value = LPWSTR::default();
                (*iterator).GetCurrentHeader(name.as_mut_ptr(), value.as_mut_ptr());
                req = req.header(name.to_string(), value.to_string());
                (*iterator).MoveNext(&mut has_current);
            }
            (*iterator).Release();
            (*headers).Release();
        }
        {
            let mut body_stream = null_mut();
            (*request).get_Content(&mut body_stream);
            if !body_stream.is_null() {
                let mut stat: STATSTG = mem::zeroed();
                (*body_stream).Stat(&mut stat as *mut _, STATFLAG_NONAME);
                let size = stat.cbSize as usize;
                let mut buffer = vec![0u8; size];
                let mut read: u32 = 0;
                (*body_stream).Read(buffer.as_mut_ptr() as *mut c_void, size as u32, &mut read);
                req = req.body(buffer);
                (*body_stream).Release();
            }
        }
        req
    }
}

#[cfg(feature = "custom_protocol")]
fn http_response_to_webview2_response(
    response: small_http::Response,
    environment: *mut ICoreWebView2Environment,
) -> *mut ICoreWebView2WebResourceResponse {
    unsafe {
        let body_stream = SHCreateMemStream(response.body.as_ptr(), response.body.len() as u32);

        let mut webview2_response = null_mut();
        (*environment).CreateWebResourceResponse(
            body_stream,
            response.status as i32,
            response.status.to_string().to_wide_string().as_ptr() as *mut _,
            response
                .headers
                .iter()
                .map(|(name, value)| format!("{name}: {value}"))
                .collect::<Vec<_>>()
                .join("\n")
                .to_wide_string()
                .as_ptr() as *mut _,
            &mut webview2_response,
        );
        (*body_stream).Release();
        webview2_response
    }
}
