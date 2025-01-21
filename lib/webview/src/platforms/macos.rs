/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{c_char, c_void};
use std::fmt::{self, Display, Formatter};
use std::ptr::null;

use objc::*;

use crate::{Event, LogicalPoint, LogicalSize, WebviewBuilder};

#[link(name = "Cocoa", kind = "framework")]
extern "C" {}

#[link(name = "WebKit", kind = "framework")]
extern "C" {}

/// Webview
pub(crate) struct Webview {
    window: Object,
    webview: Object,
    event_handler: Option<fn(&mut Webview, Event)>,
}

impl Webview {
    fn send_event(&mut self, event: Event) {
        self.event_handler.expect("Should be some")(self, event);
    }
}

impl Webview {
    pub(crate) fn new(builder: WebviewBuilder) -> Self {
        // Register AppDelegate class
        let mut decl = ClassDecl::new("AppDelegate", class!(NSObject))
            .expect("Can't create AppDelegate class");
        decl.add_ivar::<*const c_void>("_self", "^v");
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
            sel!(windowDidMove:),
            window_did_move as *const c_void,
            "v@:",
        );
        decl.add_method(
            sel!(windowDidResize:),
            window_did_resize as *const c_void,
            "v@:",
        );
        decl.add_method(
            sel!(windowWillClose:),
            window_will_close as *const c_void,
            "v@:",
        );
        decl.add_method(
            sel!(webView:didFinishNavigation:),
            webview_did_finish_navigation as *const c_void,
            "v@:@",
        );
        #[cfg(feature = "ipc")]
        decl.add_method(
            sel!(userContentController:didReceiveScriptMessage:),
            webview_did_receive_script_message as *const c_void,
            "v@:@",
        );
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
        let window_rect = NSRect {
            point: if let Some(position) = builder.position {
                NSPoint {
                    x: position.x as f64,
                    y: position.y as f64,
                }
            } else {
                NSPoint { x: 0.0, y: 0.0 }
            },
            size: NSSize {
                width: builder.size.width as f64,
                height: builder.size.height as f64,
            },
        };
        let mut window_style_mask = NS_WINDOW_STYLE_MASK_TITLED
            | NS_WINDOW_STYLE_MASK_CLOSABLE
            | NS_WINDOW_STYLE_MASK_MINIATURIZABLE;
        if builder.resizable {
            window_style_mask |= NS_WINDOW_STYLE_MASK_RESIZABLE;
        }
        let window = unsafe {
            let window: Object = msg_send![msg_send![class!(NSWindow), alloc], initWithContentRect:window_rect styleMask:window_style_mask backing:NS_BACKING_STORE_BUFFERED defer:false];
            let _: () = msg_send![window, setTitle:NSString::from_str(&builder.title).0];
            if let Some(min_size) = builder.min_size {
                let _: () = msg_send![window, setMinSize:NSSize { width: min_size.width as f64, height: min_size.height as f64 }];
            }
            if builder.position.is_none() || builder.should_center {
                let screen_frame: NSRect = msg_send![msg_send![window, screen], frame];
                let window_frame: NSRect = msg_send![window, frame];
                let centered_rect = NSRect {
                    point: NSPoint {
                        x: (screen_frame.size.width - window_frame.size.width) / 2.0,
                        y: (screen_frame.size.height - window_frame.size.height) / 2.0,
                    },
                    size: window_frame.size,
                };
                let _: () = msg_send![window, setFrame:centered_rect display:true];
            }
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

            if let Some(url) = builder.should_load_url {
                let url: Object = msg_send![class!(NSURL), URLWithString:NSString::from_str(url).0];
                let request: Object = msg_send![class!(NSURLRequest), requestWithURL:url];
                let _: () = msg_send![webview, loadRequest:request];
            }
            if let Some(html) = builder.should_load_html {
                let _: () = msg_send![webview, loadHTMLString:NSString::from_str(html).0 baseURL:null::<c_void>()];
            }
            webview
        };

        // Create AppDelegate
        unsafe {
            let app_delegate: Object = msg_send![class!(AppDelegate), new];
            let _: () = msg_send![application, setDelegate:app_delegate];
            let _: () = msg_send![window, setDelegate:app_delegate];
            let _: () = msg_send![webview, setNavigationDelegate:app_delegate];

            #[cfg(feature = "ipc")]
            {
                let user_content_controller: Object =
                    msg_send![msg_send![webview, configuration], userContentController];
                let user_script: Object = msg_send![msg_send![class!(WKUserScript), alloc],
                    initWithSource:NSString::from_str("window.ipc=new EventTarget();window.ipc.postMessage=message=>window.webkit.messageHandlers.ipc.postMessage(message);").0
                    injectionTime:WK_USER_SCRIPT_INJECTION_TIME_AT_DOCUMENT_START
                    forMainFrameOnly:true];
                let _: () = msg_send![user_content_controller, addUserScript:user_script];
                let _: () = msg_send![user_content_controller, addScriptMessageHandler:app_delegate name:NSString::from_str("ipc").0];
            }
        }

        Self {
            window,
            webview,
            event_handler: None,
        }
    }
}

impl crate::Webview for Webview {
    fn run(&mut self, event_handler: fn(&mut Webview, Event)) -> ! {
        self.event_handler = Some(event_handler);
        unsafe {
            let delegate: Object = msg_send![NSApp, delegate];
            object_setInstanceVariable(
                delegate,
                c"_self".as_ptr(),
                self as *const _ as *const c_void,
            );
        };
        unsafe { msg_send![NSApp, run] }
    }

    fn set_title(&mut self, title: impl AsRef<str>) {
        unsafe { msg_send![self.window, setTitle:NSString::from_str(title).0] }
    }

    fn set_position(&mut self, point: LogicalPoint) {
        let frame: NSRect = unsafe { msg_send![self.window, frame] };
        unsafe {
            msg_send![self.window, setFrame:NSRect { point: NSPoint { x: point.x as f64, y: point.y as f64}, size: frame.size } display:true]
        }
    }

    fn set_size(&mut self, size: LogicalSize) {
        let frame: NSRect = unsafe { msg_send![self.window, frame] };
        unsafe {
            msg_send![self.window, setFrame:NSRect { point: frame.point, size: NSSize { width: size.width as f64, height: size.height as f64 } } display:true]
        }
    }

    fn set_min_size(&mut self, min_size: LogicalSize) {
        unsafe {
            msg_send![self.window, setMinSize:NSSize { width: min_size.width as f64, height: min_size.height as f64 }]
        }
    }

    fn set_resizable(&mut self, resizable: bool) {
        let mut style_mask: i32 = unsafe { msg_send![self.window, styleMask] };
        if resizable {
            style_mask |= NS_WINDOW_STYLE_MASK_RESIZABLE;
        } else {
            style_mask &= !NS_WINDOW_STYLE_MASK_RESIZABLE;
        }
        unsafe { msg_send![self.window, setStyleMask:style_mask] }
    }

    fn load_url(&mut self, url: impl AsRef<str>) {
        unsafe {
            let url: Object = msg_send![class!(NSURL), URLWithString:NSString::from_str(url).0];
            let request: Object = msg_send![class!(NSURLRequest), requestWithURL:url];
            msg_send![self.webview, loadRequest:request]
        }
    }

    fn load_html(&mut self, html: impl AsRef<str>) {
        unsafe {
            msg_send![self.webview, loadHTMLString:NSString::from_str(html).0 baseURL:null::<c_void>()]
        }
    }

    fn evaluate_script(&mut self, script: impl AsRef<str>) {
        unsafe {
            msg_send![self.webview, evaluateJavaScript:NSString::from_str(script).0 completionHandler:null::<*const c_void>()]
        }
    }

    #[cfg(feature = "ipc")]
    fn send_ipc_message(&mut self, message: impl AsRef<str>) {
        self.evaluate_script(format!(
            "window.ipc.dispatchEvent(new MessageEvent('message',{{data:`{}`}}));",
            message.as_ref()
        ));
    }
}

extern "C" fn app_did_finish_launching(this: Object, _sel: Sel, _notification: Object) {
    // Get self
    let _self = unsafe {
        let mut _self = null();
        object_getInstanceVariable(this, c"_self".as_ptr(), &mut _self);
        &mut *(_self as *mut Webview)
    };

    // Show window
    unsafe {
        let _: () = msg_send![NSApp, setActivationPolicy:NS_APPLICATION_ACTIVATION_POLICY_REGULAR];
        let _: () = msg_send![NSApp, activateIgnoringOtherApps:true];
        let _: () = msg_send![_self.window, makeKeyAndOrderFront:null::<c_void>()];
    }

    // Send window created event
    _self.send_event(Event::WindowCreated);

    // Send window resized event
    let content_view_rect: NSRect =
        unsafe { msg_send![msg_send![_self.window, contentView], frame] };
    _self.send_event(Event::WindowResized(LogicalSize::new(
        content_view_rect.size.width as f32,
        content_view_rect.size.height as f32,
    )));
}

extern "C" fn app_should_terminate_after_last_window_closed(
    _this: Object,
    _sel: Sel,
    _sender: Object,
) -> bool {
    true
}

extern "C" fn window_did_move(this: Object, _sel: Sel, _notification: Object) {
    // Get self
    let _self = unsafe {
        let mut _self = null();
        object_getInstanceVariable(this, c"_self".as_ptr(), &mut _self);
        &mut *(_self as *mut Webview)
    };

    // Get new position
    let window_frame: NSRect = unsafe { msg_send![_self.window, frame] };
    let new_position = LogicalPoint::new(window_frame.point.x as f32, window_frame.point.y as f32);

    // Send window moved event
    _self.send_event(Event::WindowMoved(new_position));
}

extern "C" fn window_did_resize(this: Object, _sel: Sel, _notification: Object) {
    // Get self
    let _self = unsafe {
        let mut _self = null();
        object_getInstanceVariable(this, c"_self".as_ptr(), &mut _self);
        &mut *(_self as *mut Webview)
    };

    // Get new size
    let content_view_rect: NSRect =
        unsafe { msg_send![msg_send![_self.window, contentView], frame] };
    let new_size = LogicalSize::new(
        content_view_rect.size.width as f32,
        content_view_rect.size.height as f32,
    );

    // Resize webview
    let _: () = unsafe { msg_send![_self.webview, setFrame:content_view_rect] };

    // Send window resized event
    _self.send_event(Event::WindowResized(new_size));
}

extern "C" fn window_will_close(this: Object, _sel: Sel, _notification: Object) {
    // Get self
    let _self = unsafe {
        let mut _self = null();
        object_getInstanceVariable(this, c"_self".as_ptr(), &mut _self);
        &mut *(_self as *mut Webview)
    };

    // Send window closed event
    _self.send_event(Event::WindowClosed);
}

extern "C" fn webview_did_finish_navigation(
    this: Object,
    _sel: Sel,
    _webview: Object,
    _navigation: Object,
) {
    // Get self
    let _self = unsafe {
        let mut _self = null();
        object_getInstanceVariable(this, c"_self".as_ptr(), &mut _self);
        &mut *(_self as *mut Webview)
    };

    // Send page load finished event
    _self.send_event(Event::PageLoadFinished);
}

#[cfg(feature = "ipc")]
extern "C" fn webview_did_receive_script_message(
    this: Object,
    _sel: Sel,
    _user_content_controller: Object,
    message: Object,
) {
    // Get self
    let _self = unsafe {
        let mut _self = null();
        object_getInstanceVariable(this, c"_self".as_ptr(), &mut _self);
        &mut *(_self as *mut Webview)
    };

    // Send ipc message received event
    let body: NSString = unsafe { msg_send![message, body] };
    _self.send_event(Event::IpcMessageReceived(body.to_string()));
}

// MARK: Cocoa headers
#[repr(C)]
struct NSPoint {
    x: f64,
    y: f64,
}
#[repr(C)]
struct NSSize {
    width: f64,
    height: f64,
}
#[repr(C)]
struct NSRect {
    point: NSPoint,
    size: NSSize,
}

const NS_APPLICATION_ACTIVATION_POLICY_REGULAR: i32 = 0;

const NS_UTF8_STRING_ENCODING: i32 = 4;

const NS_WINDOW_STYLE_MASK_TITLED: i32 = 1;
const NS_WINDOW_STYLE_MASK_CLOSABLE: i32 = 2;
const NS_WINDOW_STYLE_MASK_MINIATURIZABLE: i32 = 4;
const NS_WINDOW_STYLE_MASK_RESIZABLE: i32 = 8;

const NS_BACKING_STORE_BUFFERED: i32 = 2;

#[cfg(feature = "ipc")]
const WK_USER_SCRIPT_INJECTION_TIME_AT_DOCUMENT_START: i32 = 0;

extern "C" {
    static NSApp: Object;
}

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
