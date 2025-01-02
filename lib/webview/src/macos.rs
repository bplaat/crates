/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{c_char, c_void};
use std::fmt::{self, Display, Formatter};
use std::ptr::null;

use objc::*;

use super::*;

#[link(name = "Cocoa", kind = "framework")]
extern "C" {}

#[link(name = "WebKit", kind = "framework")]
extern "C" {}

/// Webview
pub struct Webview {
    window: Object,
    webview: Object,
}

impl Webview {
    pub(crate) fn new(builder: WebviewBuilder) -> Self {
        // Register AppDelegate class
        let mut decl = ClassDecl::new("AppDelegate", class!(NSObject))
            .expect("Can't create AppDelegate class");
        decl.add_ivar::<*const c_void>("_event_handler", "^v");
        decl.add_ivar::<*const c_void>("_window", "^v");
        decl.add_ivar::<*const c_void>("_webview", "^v");
        decl.add_method(
            sel!(applicationDidFinishLaunching:),
            app_did_finish_launching as *const c_void,
            "v@:",
        );
        decl.add_method(
            sel!(applicationShouldTerminateAfterLastWindowClosed:),
            app_should_terminate_after_last_window_closed as *const c_void,
            "B@:",
        );
        decl.add_method(
            sel!(windowDidResize:),
            window_did_resize as *const c_void,
            "v@:",
        );
        decl.add_method(
            sel!(webView:didFinishNavigation:),
            webview_did_finish_navigation as *const c_void,
            "v@:@",
        );
        if builder.remember_window_state {
            decl.add_method(
                sel!(userContentController:didReceiveScriptMessage:),
                webview_did_receive_script_message as *const c_void,
                "v@:@",
            );
        }
        decl.register();

        // Get application
        let application = unsafe { msg_send![class!(NSApplication), sharedApplication] };

        // Create menu
        unsafe {
            let menubar = msg_send![class!(NSMenu), new];
            let _: () = msg_send![application, setMainMenu:menubar];

            let menu_bar_item = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![menubar, addItem:menu_bar_item];

            let app_menu = msg_send![class!(NSMenu), new];
            let _: () = msg_send![menu_bar_item, setSubmenu:app_menu];

            let app_name: NSString =
                msg_send![application, valueForKey:NSString::from_str("name").0];
            let quit_menu_item: *mut Object = msg_send![msg_send![class!(NSMenuItem), alloc],
            initWithTitle:NSString::from_str(format!("Quit {}", app_name)).0
            action:sel!(terminate:) keyEquivalent:NSString::from_str("q").0];
            let _: () = msg_send![app_menu, addItem:quit_menu_item];
        }

        // Create window
        let window = unsafe {
            let window: Object = msg_send![msg_send![class!(NSWindow), alloc], initWithContentRect:NSRect { x: 0.0, y: 0.0, width: builder.size.0 as f64, height: builder.size.1 as f64 }
                    styleMask:NS_WINDOW_STYLE_MASK_TITLED | NS_WINDOW_STYLE_MASK_CLOSABLE | NS_WINDOW_STYLE_MASK_MINIATURIZABLE | NS_WINDOW_STYLE_MASK_RESIZABLE
                    backing:NS_BACKING_STORE_BUFFERED
                    defer:false];
            let _: () = msg_send![window, setTitle:NSString::from_str(&builder.title).0];
            if let Some(min_size) = builder.min_size {
                let _: () = msg_send![window, setMinSize:NSSize { width: min_size.0 as f64, height: min_size.1 as f64 }];
            }

            // Center window
            let screen_frame: NSRect = msg_send![msg_send![window, screen], frame];
            let window_frame: NSRect = msg_send![window, frame];
            let _: () = msg_send![window, setFrame:NSRect { x: (screen_frame.width - window_frame.width) / 2.0, y : (screen_frame.height - window_frame.height) / 2.0,width: window_frame.width,height: window_frame.height} display:true];

            if builder.remember_window_state {
                let _: () = msg_send![window, setFrameAutosaveName:NSString::from_str("window").0];
            }

            window
        };

        // Create webview
        let webview = unsafe {
            let content_view: Object = msg_send![window, contentView];
            let content_view_rect: NSRect = msg_send![content_view, frame];
            let webview: Object =
                msg_send![msg_send![class!(WKWebView), alloc], initWithFrame:content_view_rect];
            let _: () = msg_send![content_view, addSubview:webview];

            if let Some(url) = builder.url {
                let url: Object = msg_send![class!(NSURL), URLWithString:NSString::from_str(url).0];
                let request: Object = msg_send![class!(NSURLRequest), requestWithURL:url];
                let _: () = msg_send![webview, loadRequest:request];
            }
            if let Some(html) = builder.html {
                let _: () = msg_send![webview, loadHTMLString:NSString::from_str(html).0 baseURL:null::<c_void>()];
            }
            webview
        };

        // Create AppDelegate and set
        unsafe {
            let app_delegate: Object = msg_send![class!(AppDelegate), new];
            object_setInstanceVariable(app_delegate, c"_window".as_ptr(), window);
            object_setInstanceVariable(app_delegate, c"_webview".as_ptr(), webview);
            let _: () = msg_send![application, setDelegate:app_delegate];
            let _: () = msg_send![window, setDelegate:app_delegate];
            let _: () = msg_send![webview, setNavigationDelegate:app_delegate];
            if builder.enable_ipc {
                let _: () = msg_send![msg_send![msg_send![webview, configuration], userContentController], addScriptMessageHandler:app_delegate name:NSString::from_str("ipc").0];
            }
        }

        Self { window, webview }
    }

    /// Start event loop
    pub fn run(&mut self, event_handler: fn(&mut Webview, Event)) {
        // Set event handler
        let application = unsafe { msg_send![class!(NSApplication), sharedApplication] };
        unsafe {
            object_setInstanceVariable(
                msg_send![application, delegate],
                c"_event_handler".as_ptr(),
                event_handler as *const c_void,
            )
        };

        // Start event loop
        unsafe { msg_send![application, run] }
    }

    /// Set title
    pub fn set_title(&mut self, title: impl AsRef<str>) {
        unsafe { msg_send![self.window, setTitle:NSString::from_str(title).0] }
    }

    /// Set position
    pub fn set_position(&mut self, x: i32, y: i32) {
        let frame: NSRect = unsafe { msg_send![self.window, frame] };
        unsafe {
            msg_send![self.window, setFrame:NSRect { x: x as f64, y: y as f64, width: frame.width, height: frame.height } display:true]
        }
    }

    /// Set size
    pub fn set_size(&mut self, width: i32, height: i32) {
        let frame: NSRect = unsafe { msg_send![self.window, frame] };
        unsafe {
            msg_send![self.window, setFrame:NSRect { x: frame.x, y: frame.y, width: width as f64, height: height as f64 } display:true]
        }
    }

    /// Open URL
    pub fn open_url(&mut self, url: impl AsRef<str>) {
        unsafe {
            let url: Object = msg_send![class!(NSURL), URLWithString:NSString::from_str(url).0];
            let request: Object = msg_send![class!(NSURLRequest), requestWithURL:url];
            msg_send![self.webview, loadRequest:request]
        }
    }

    /// Open HTML
    pub fn open_html(&mut self, html: impl AsRef<str>) {
        unsafe {
            msg_send![self.webview, loadHTMLString:NSString::from_str(html).0 baseURL:null::<c_void>()]
        }
    }

    /// Eval JavaScript
    pub fn eval(&mut self, js: String) {
        unsafe {
            msg_send![self.webview, evaluateJavaScript:NSString::from_str(js).0 completionHandler:null::<*const c_void>()]
        }
    }
}

extern "C" fn app_did_finish_launching(this: Object, _sel: Sel, _notification: Object) {
    let application = unsafe { msg_send![class!(NSApplication), sharedApplication] };
    let mut window: Object = null();
    unsafe { object_getInstanceVariable(this, c"_window".as_ptr(), &mut window) };

    // Show window
    unsafe {
        let _: () =
            msg_send![application, setActivationPolicy:NS_APPLICATION_ACTIVATION_POLICY_REGULAR];
        let _: () = msg_send![application, activateIgnoringOtherApps:true];
        let _: () = msg_send![window, makeKeyAndOrderFront:null::<c_void>()];
    }
}

extern "C" fn app_should_terminate_after_last_window_closed(
    _this: Object,
    _sel: Sel,
    _sender: Object,
) -> bool {
    true
}

extern "C" fn window_did_resize(this: Object, _sel: Sel, _notification: Object) {
    let mut window: Object = null();
    unsafe { object_getInstanceVariable(this, c"_window".as_ptr(), &mut window) };
    let mut webview: Object = null();
    unsafe { object_getInstanceVariable(this, c"_webview".as_ptr(), &mut webview) };

    // Resize webview
    unsafe {
        let content_view: Object = msg_send![window, contentView];
        let content_view_rect: NSRect = msg_send![content_view, frame];
        msg_send![webview, setFrame:content_view_rect]
    }
}

extern "C" fn webview_did_finish_navigation(
    this: Object,
    _sel: Sel,
    _webview: Object,
    _navigation: Object,
) {
    send_message(this, Event::PageLoaded);
}

extern "C" fn webview_did_receive_script_message(
    this: Object,
    _sel: Sel,
    _user_content_controller: Object,
    message: Object,
) {
    let body: NSString = unsafe { msg_send![message, body] };
    send_message(this, Event::IpcMessageReceived(body.to_string()));
}

fn send_message(this: Object, event: Event) {
    let mut webview = unsafe {
        let mut window = null();
        object_getInstanceVariable(this, c"_window".as_ptr(), &mut window);
        let mut webview = null();
        object_getInstanceVariable(this, c"_webview".as_ptr(), &mut webview);
        Webview { window, webview }
    };
    let event_handler: fn(&mut Webview, Event) = unsafe {
        let mut event_handler = null();
        object_getInstanceVariable(this, c"_event_handler".as_ptr(), &mut event_handler);
        std::mem::transmute(event_handler)
    };
    event_handler(&mut webview, event);
}

// MARK: Cocoa defs
#[repr(C)]
struct NSSize {
    width: f64,
    height: f64,
}

#[repr(C)]
struct NSRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

const NS_APPLICATION_ACTIVATION_POLICY_REGULAR: i32 = 0;

const NS_UTF8_STRING_ENCODING: i32 = 4;
#[allow(dead_code)]
struct NSString(Object);
impl NSString {
    fn from_str(str: impl AsRef<str>) -> Self {
        let str = str.as_ref();
        unsafe {
            msg_send![
                msg_send![msg_send![class!(NSString), alloc], initWithBytes:str.as_ptr() length:str.len() encoding:NS_UTF8_STRING_ENCODING],
                autorelease
            ]
        }
    }
}
impl Display for NSString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe {
            let bytes: *const c_char = msg_send![self.0, UTF8String];
            let len: usize = msg_send![self.0, lengthOfBytesUsingEncoding:NS_UTF8_STRING_ENCODING];
            String::from_utf8_lossy(std::slice::from_raw_parts(bytes as *const u8, len as usize))
        })
    }
}

type NSWindowStyleMask = i32;
const NS_WINDOW_STYLE_MASK_TITLED: NSWindowStyleMask = 1;
const NS_WINDOW_STYLE_MASK_CLOSABLE: NSWindowStyleMask = 2;
const NS_WINDOW_STYLE_MASK_MINIATURIZABLE: NSWindowStyleMask = 4;
const NS_WINDOW_STYLE_MASK_RESIZABLE: NSWindowStyleMask = 8;

type NSBackingStoreType = i32;
const NS_BACKING_STORE_BUFFERED: NSBackingStoreType = 2;
