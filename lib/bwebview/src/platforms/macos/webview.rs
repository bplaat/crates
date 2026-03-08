/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;
use std::ptr::{null, null_mut};

use block2::Block;
use objc2::runtime::{AnyClass, AnyObject as Object, Bool, ClassBuilder, Sel};
use objc2::{class, msg_send, sel};

use super::cocoa::*;
use super::event_loop::{IVAR_PTR, IVAR_PTR_CSTR, send_event};
use super::webkit::*;
use super::window::PlatformWindow;
use crate::{InjectionTime, WebviewBuilder, WebviewEvent};

pub(super) struct WebviewData {
    pub(super) window: *mut Object,
    pub(super) background_color: Option<u32>,
    pub(super) webview: *mut Object,
}

pub(crate) struct PlatformWebview(pub(super) Box<WebviewData>);

impl PlatformWebview {
    pub(crate) fn new(window: &PlatformWindow) -> Self {
        PlatformWebview(Box::new(WebviewData {
            window: window.0.window,
            background_color: window.0.background_color,
            webview: std::ptr::null_mut(),
        }))
    }
}

impl PlatformWebview {
    pub(crate) fn init_webview(&mut self, builder: WebviewBuilder<'_>) {
        // Register WebviewDelegate class once
        if AnyClass::get(c"WebviewDelegate").is_none() {
            let mut decl = ClassBuilder::new(c"WebviewDelegate", class!(NSObject))
                .expect("Can't create WebviewDelegate class");
            decl.add_ivar::<*const c_void>(IVAR_PTR_CSTR);
            unsafe {
                decl.add_method(
                    sel!(webView:didStartProvisionalNavigation:),
                    webview_did_start_provisional_navigation as extern "C" fn(_, _, _, _),
                );
                decl.add_method(
                    sel!(webView:didFinishNavigation:),
                    webview_did_finish_navigation as extern "C" fn(_, _, _, _),
                );
                decl.add_method(
                    sel!(observeValueForKeyPath:ofObject:change:context:),
                    webview_observe_value_for_key_path as extern "C" fn(_, _, _, _, _, _),
                );
                decl.add_method(
                    sel!(webView:decidePolicyForNavigationAction:decisionHandler:),
                    webview_decide_policy_for_navigation_action as extern "C" fn(_, _, _, _, _),
                );
                decl.add_method(
                    sel!(userContentController:didReceiveScriptMessage:),
                    webview_did_receive_script_message as extern "C" fn(_, _, _, _),
                );
            }
            decl.register();
        }

        // Create WebviewDelegate and set _ptr to PlatformWindowData
        let webview_delegate: *mut Object = unsafe { msg_send![class!(WebviewDelegate), new] };
        unsafe {
            #[allow(deprecated)]
            let ptr_ivar = (*webview_delegate).get_mut_ivar::<*const c_void>(IVAR_PTR);
            *ptr_ivar = self.0.as_ref() as *const WebviewData as *const c_void;
        };

        // Create webview
        let webview = unsafe {
            // Create webview configuration
            let webview_config: *mut Object = msg_send![class!(WKWebViewConfiguration), new];
            let webview_config: *mut Object = msg_send![webview_config, autorelease];
            let website_data_store: *mut Object =
                msg_send![class!(WKWebsiteDataStore), defaultDataStore];
            let _: () = msg_send![webview_config, setWebsiteDataStore:website_data_store];

            #[cfg(feature = "custom_protocol")]
            for custom_protocol in builder.custom_protocols {
                use objc2::runtime::ClassBuilder;

                let url_scheme = NSString::from_str(&custom_protocol.scheme);

                let class_name = std::ffi::CString::new(format!(
                    "{}_CustomProtocolDelegate",
                    custom_protocol.scheme
                ))
                .expect("Can't create CString");
                let mut decl = ClassBuilder::new(class_name.as_c_str(), class!(NSObject))
                    .expect("Can't create custom protocol delegate class");
                decl.add_ivar::<*const c_void>(IVAR_PTR_CSTR);
                decl.add_method(
                    objc2::sel!(webView:startURLSchemeTask:),
                    custom_protocol_start_url_scheme_task as extern "C" fn(_, _, _, _),
                );
                let class = decl.register();

                let delegate: *mut Object = msg_send![class, new];
                #[allow(deprecated)]
                let ptr_to_ptr = (*delegate).get_mut_ivar::<*const c_void>(IVAR_PTR);
                *ptr_to_ptr = Box::leak(Box::new(custom_protocol)) as *mut _ as *const c_void;

                let _: () = msg_send![webview_config, setURLSchemeHandler:delegate, forURLScheme:url_scheme];
            }

            // Get content view rect
            let content_view: *mut Object = msg_send![self.0.window, contentView];
            let webview_rect = if unsafe { msg_send![self.0.window, titlebarAppearsTransparent] } {
                let mut window_frame: NSRect = msg_send![self.0.window, frame];
                window_frame.origin.x = 0.0;
                window_frame.origin.y = 0.0;
                window_frame
            } else {
                msg_send![content_view, frame]
            };

            // Create webview
            let webview: *mut Object = msg_send![class!(WKWebView), alloc];
            let webview: *mut Object =
                msg_send![webview, initWithFrame:webview_rect, configuration:webview_config];
            let _: () = msg_send![webview, setNavigationDelegate:webview_delegate];
            let _: () = msg_send![content_view, addSubview:webview];
            let _: () = msg_send![webview, setAutoresizingMask: NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE];
            if unsafe { self.0.background_color }.is_some() {
                let value: *mut Object = msg_send![class!(NSNumber), numberWithBool:false];
                let _: () = msg_send![webview, setValue:value, forKey:NSString::from_str("drawsBackground")];
            }
            let useragent = format!(
                "Mozilla/5.0 (Macintosh; {}) bwebview/{}",
                std::env::consts::ARCH,
                env!("CARGO_PKG_VERSION"),
            );
            let _: () = msg_send![webview, setCustomUserAgent:NSString::from_str(&useragent)];
            let _: () = msg_send![
                webview,
                addObserver:webview_delegate,
                forKeyPath:NSString::from_str("title"),
                options:NS_KEY_VALUE_OBSERVING_OPTION_NEW,
                context:null::<c_void>()
            ];
            if let Some(url) = builder.should_load_url {
                let url: *mut Object =
                    msg_send![class!(NSURL), URLWithString:NSString::from_str(url)];
                let request: *mut Object = msg_send![class!(NSURLRequest), requestWithURL:url];
                let _: *mut Object = msg_send![webview, loadRequest:request];
            }
            if let Some(html) = builder.should_load_html {
                let _: *mut Object = msg_send![webview, loadHTMLString:NSString::from_str(html), baseURL:null::<Object>()];
            }
            if cfg!(debug_assertions) {
                let webview_configuration: *mut Object = msg_send![webview, configuration];
                let webview_preferences: *mut Object =
                    msg_send![webview_configuration, preferences];
                let ns_bool_yes: *mut Object = msg_send![class!(NSNumber), numberWithBool:true];
                let _: () = msg_send![webview_preferences, setValue:ns_bool_yes, forKey:NSString::from_str("developerExtrasEnabled")];
            }
            webview
        };

        // Create ipc handler
        unsafe {
            const IPC_SCRIPT: &str = "window.ipc = new EventTarget();\
                window.ipc.postMessage = message => window.webkit.messageHandlers.ipc.postMessage(typeof message !== 'string' ? JSON.stringify(message) : message);";
            #[cfg(feature = "log")]
            const CONSOLE_SCRIPT: &str = "for (const level of ['error', 'warn', 'info', 'debug', 'trace', 'log'])\
                window.console[level] = (...args) => window.webkit.messageHandlers.console.postMessage(level.charAt(0) + args.map(arg => typeof arg !== 'string' ? JSON.stringify(arg) : arg).join(' '));";
            #[cfg(not(feature = "log"))]
            let script = IPC_SCRIPT;
            #[cfg(feature = "log")]
            let script = format!("{IPC_SCRIPT}\n{CONSOLE_SCRIPT}");

            let webview_configuration: *mut Object = msg_send![webview, configuration];
            let user_content_controller: *mut Object =
                msg_send![webview_configuration, userContentController];
            let user_script: *mut Object = msg_send![class!(WKUserScript), alloc];
            let user_script: *mut Object = msg_send![user_script,
                    initWithSource:NSString::from_str(script),
                    injectionTime:WK_USER_SCRIPT_INJECTION_TIME_AT_DOCUMENT_START,
                    forMainFrameOnly:true];
            let _: () = msg_send![user_content_controller, addUserScript:user_script];
            let _: () = msg_send![user_content_controller, addScriptMessageHandler:webview_delegate, name:NSString::from_str("ipc")];
            #[cfg(feature = "log")]
            let _: () = msg_send![user_content_controller, addScriptMessageHandler:webview_delegate, name:NSString::from_str("console")];
        }

        self.0.webview = webview;
    }
}

impl crate::WebviewInterface for PlatformWebview {
    fn url(&self) -> Option<String> {
        unsafe {
            let url: *mut Object = msg_send![self.0.webview, URL];
            if !url.is_null() {
                let url: NSString = msg_send![url, absoluteString];
                Some(url.to_string())
            } else {
                None
            }
        }
    }

    fn load_url(&mut self, url: impl AsRef<str>) {
        unsafe {
            let url: *mut Object = msg_send![class!(NSURL), URLWithString:NSString::from_str(url)];
            let request: *mut Object = msg_send![class!(NSURLRequest), requestWithURL:url];
            msg_send![self.0.webview, loadRequest:request]
        }
    }

    fn load_html(&mut self, html: impl AsRef<str>) {
        unsafe {
            msg_send![self.0.webview, loadHTMLString:NSString::from_str(html), baseURL:null::<c_void>()]
        }
    }

    fn evaluate_script(&mut self, script: impl AsRef<str>) {
        let script = script.as_ref();
        let _: () = unsafe {
            msg_send![self.0.webview, evaluateJavaScript:NSString::from_str(script), completionHandler:null::<Object>()]
        };
    }

    fn add_user_script(&mut self, script: impl AsRef<str>, injection_time: InjectionTime) {
        let script = script.as_ref();
        unsafe {
            let webview_configuration: *mut Object = msg_send![self.0.webview, configuration];
            let user_content_controller: *mut Object =
                msg_send![webview_configuration, userContentController];
            let user_script: *mut Object = msg_send![class!(WKUserScript), alloc];
            let user_script: *mut Object = msg_send![user_script,
                    initWithSource:NSString::from_str(script),
                    injectionTime: match injection_time {
                        InjectionTime::DocumentStart => WK_USER_SCRIPT_INJECTION_TIME_AT_DOCUMENT_START,
                        InjectionTime::DocumentLoaded => WK_USER_SCRIPT_INJECTION_TIME_AT_DOCUMENT_END,
                    },
                    forMainFrameOnly:true];
            let _: () = msg_send![user_content_controller, addUserScript:user_script];
        }
    }

    fn set_background_color(&mut self, color: u32) {
        self.0.background_color = Some(color);
        if !self.0.webview.is_null() {
            unsafe {
                let value: *mut Object = msg_send![class!(NSNumber), numberWithBool:false];
                let _: () = msg_send![self.0.webview, setValue:value, forKey:NSString::from_str("drawsBackground")];
            }
        }
    }
}

extern "C" fn webview_did_start_provisional_navigation(
    _this: *mut Object,
    _sel: Sel,
    _webview: *mut Object,
    _navigation: *mut Object,
) {
    send_event(crate::Event::Webview(WebviewEvent::PageLoadStart));
}

extern "C" fn webview_did_finish_navigation(
    _this: *mut Object,
    _sel: Sel,
    _webview: *mut Object,
    _navigation: *mut Object,
) {
    send_event(crate::Event::Webview(WebviewEvent::PageLoadFinish));
}

extern "C" fn webview_observe_value_for_key_path(
    _this: *mut Object,
    _sel: Sel,
    key_path: NSString,
    _of_object: *mut Object,
    change: *mut Object,
    _context: *mut c_void,
) {
    let key_path = key_path.to_string();
    if key_path == "title" {
        let change: NSString = unsafe { msg_send![change, objectForKey:NSKeyValueChangeNewKey] };
        send_event(crate::Event::Webview(WebviewEvent::PageTitleChange(
            change.to_string(),
        )));
    }
}

extern "C" fn webview_decide_policy_for_navigation_action(
    _this: *mut Object,
    _sel: Sel,
    _webview: *mut Object,
    navigation_action: *mut Object,
    decision_handler: &Block<dyn Fn(i64)>,
) {
    unsafe {
        let target_frame: *mut Object = msg_send![navigation_action, targetFrame];
        if target_frame.is_null() {
            let request: *mut Object = msg_send![navigation_action, request];
            let url: *mut Object = msg_send![request, URL];
            let workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
            let _: Bool = msg_send![workspace, openURL:url];
            decision_handler.call((WK_NAVIGATION_ACTION_POLICY_CANCEL,));
        } else {
            decision_handler.call((WK_NAVIGATION_ACTION_POLICY_ALLOW,));
        }
    }
}

extern "C" fn webview_did_receive_script_message(
    _this: *mut Object,
    _sel: Sel,
    _user_content_controller: *mut Object,
    message: *mut Object,
) {
    let name: NSString = unsafe { msg_send![message, name] };
    let name = name.to_string();
    let body: NSString = unsafe { msg_send![message, body] };
    let body = body.to_string();

    #[cfg(feature = "log")]
    if name == "console" {
        let (level, message) = body.split_at(1);
        match level {
            "e" => log::error!("{message}"),
            "w" => log::warn!("{message}"),
            "i" | "l" => log::info!("{message}"),
            "d" => log::debug!("{message}"),
            "t" => log::trace!("{message}"),
            _ => unimplemented!(),
        }
    }
    if name == "ipc" {
        send_event(crate::Event::Webview(WebviewEvent::MessageReceive(body)));
    }
}

#[cfg(feature = "custom_protocol")]
extern "C" fn custom_protocol_start_url_scheme_task(
    this: *mut Object,
    _sel: Sel,
    _webview: *mut Object,
    url_scheme_task: *mut Object,
) {
    let custom_protocol = unsafe {
        #[allow(deprecated)]
        let ptr = *(*this).get_ivar::<*mut c_void>(IVAR_PTR);
        &mut *(ptr as *mut crate::CustomProtocol)
    };

    let ns_request: *mut Object = unsafe { msg_send![url_scheme_task, request] };
    let req = ns_request_to_http_request(ns_request);

    let res = (custom_protocol.handler)(&req);

    let (ns_response, ns_data) = http_response_to_ns_response(&res, &req);
    unsafe {
        let _: () = msg_send![url_scheme_task, didReceiveResponse:ns_response];
        let _: () = msg_send![url_scheme_task, didReceiveData:ns_data];
        let _: () = msg_send![url_scheme_task, didFinish];
    }
}

#[cfg(feature = "custom_protocol")]
fn ns_request_to_http_request(ns_request: *mut Object) -> small_http::Request {
    use std::str::FromStr;

    let method: NSString = unsafe { msg_send![ns_request, HTTPMethod] };
    let method = method.to_string();
    let url: *mut Object = unsafe { msg_send![ns_request, URL] };
    let url: NSString = unsafe { msg_send![url, absoluteString] };
    let url = url.to_string();
    let mut req = small_http::Request::with_method_and_url(
        small_http::Method::from_str(&method).unwrap_or(small_http::Method::Get),
        &url,
    );

    let headers: *mut Object = unsafe { msg_send![ns_request, allHTTPHeaderFields] };
    let keys: *mut Object = unsafe { msg_send![headers, allKeys] };
    let count: usize = unsafe { msg_send![keys, count] };
    for i in 0..count {
        let key: NSString = unsafe { msg_send![keys, objectAtIndex:i] };
        let value: NSString = unsafe { msg_send![headers, objectForKey:key.0] };
        req = req.header(key.to_string(), value.to_string());
    }

    let body: *mut Object = unsafe { msg_send![ns_request, HTTPBody] };
    if !body.is_null() {
        let length: usize = unsafe { msg_send![body, length] };
        let bytes: *const c_void = unsafe { msg_send![body, bytes] };
        let mut post_data = Vec::with_capacity(length);
        post_data
            .extend_from_slice(unsafe { std::slice::from_raw_parts(bytes as *const u8, length) });
        req = req.body(post_data)
    }
    req
}

#[cfg(feature = "custom_protocol")]
fn http_response_to_ns_response(
    res: &small_http::Response,
    req: &small_http::Request,
) -> (*mut Object, *mut Object) {
    let ns_response: *mut Object = unsafe {
        let url: *mut Object =
            msg_send![class!(NSURL), URLWithString:NSString::from_str(req.url.to_string())];

        let headers: *mut Object = msg_send![class!(NSMutableDictionary), dictionary];
        for (key, value) in &res.headers {
            let _: () = msg_send![headers, setObject:NSString::from_str(value), forKey:NSString::from_str(key)];
        }

        let ns_response: *mut Object = msg_send![class!(NSHTTPURLResponse), alloc];
        let ns_response: *mut Object = msg_send![ns_response, autorelease];
        let ns_response: *mut Object = msg_send![
            ns_response,
            initWithURL:url,
            statusCode:res.status as i64,
            HTTPVersion:NSString::from_str(req.version.to_string()),
            headerFields:headers
        ];
        ns_response
    };
    let ns_data: *mut Object = unsafe {
        msg_send![class!(NSData), dataWithBytes:res.body.as_ptr() as *const c_void, length:res.body.len()]
    };
    (ns_response, ns_data)
}
