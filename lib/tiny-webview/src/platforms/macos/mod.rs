/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CString, c_void};
use std::ptr::null;

use block2::Block;
use objc2::runtime::{AnyClass, AnyObject as Object, Bool, ClassBuilder, Sel};
use objc2::{class, msg_send, sel};

use self::cocoa::*;
use self::webkit::*;
use crate::{Event, LogicalPoint, LogicalSize, MacosTitlebarStyle, Theme, WebviewBuilder};

mod cocoa;
mod webkit;

const IVAR_SELF: &str = "_self";

// MARK: EventLoop
pub(crate) struct PlatformEventLoop {
    application: *mut Object,
    event_handler: Option<Box<dyn FnMut(Event) + 'static>>,
}

impl PlatformEventLoop {
    pub(crate) fn new() -> Self {
        // Register AppDelegate class
        let mut decl = ClassBuilder::new(c"AppDelegate", class!(NSObject))
            .expect("Can't create AppDelegate class");
        let ivar_self = CString::new(IVAR_SELF).expect("Should be some");
        decl.add_ivar::<*const c_void>(&ivar_self);
        unsafe {
            decl.add_method(
                sel!(applicationDidFinishLaunching:),
                app_did_finish_launching as extern "C" fn(_, _, _),
            );
            decl.add_method(
                sel!(applicationShouldTerminateAfterLastWindowClosed:),
                app_should_terminate_after_last_window_closed as extern "C" fn(_, _, _) -> _,
            );
            decl.add_method(sel!(sendEvent:), app_send_event as extern "C" fn(_, _, _));
            decl.add_method(
                sel!(openAboutDialog:),
                app_open_about_dialog as extern "C" fn(_, _, _),
            );
        }
        decl.register();

        // Create AppDelegate instance
        let app_delegate: *mut Object = unsafe { msg_send![class!(AppDelegate), new] };

        // Get application
        let application = unsafe {
            let application: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let _: () = msg_send![application, setDelegate:app_delegate];
            application
        };

        // Create menu
        unsafe {
            let app_name: NSString = msg_send![application, valueForKey:NSString::from_str("name")];

            let menubar: *mut Object = msg_send![class!(NSMenu), new];
            let _: () = msg_send![application, setMainMenu:menubar];

            let menu_bar_item: *mut Object = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![menubar, addItem:menu_bar_item];

            let app_menu: *mut Object = msg_send![class!(NSMenu), new];
            let _: () = msg_send![menu_bar_item, setSubmenu:app_menu];

            let about_menu_item: *mut Object = msg_send![class!(NSMenuItem), alloc];
            let about_menu_item: *mut Object = msg_send![about_menu_item,
                initWithTitle:NSString::from_str(format!("About {}", app_name)),
                action:sel!(openAboutDialog:),
                keyEquivalent:NSString::from_str("")];
            let _: () = msg_send![app_menu, addItem:about_menu_item];

            let separator_item: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
            let _: () = msg_send![app_menu, addItem:separator_item];

            let quit_menu_item: *mut Object = msg_send![class!(NSMenuItem), alloc];
            let quit_menu_item: *mut Object = msg_send![quit_menu_item,
                initWithTitle:NSString::from_str(format!("Quit {}", app_name)),
                action:sel!(terminate:), keyEquivalent:NSString::from_str("q")];
            let _: () = msg_send![app_menu, addItem:quit_menu_item];
        }

        Self {
            application,
            event_handler: None,
        }
    }
}

impl crate::EventLoopInterface for PlatformEventLoop {
    fn available_monitors(&self) -> Vec<PlatformMonitor> {
        let mut monitors = Vec::new();
        unsafe {
            let screens: *mut Object = msg_send![class!(NSScreen), screens];
            let count: usize = msg_send![screens, count];
            for i in 0..count {
                let screen: *mut Object = msg_send![screens, objectAtIndex:i];
                monitors.push(PlatformMonitor::new(screen));
            }
        }
        monitors
    }

    fn run(mut self, event_handler: impl FnMut(Event) + 'static) -> ! {
        self.event_handler = Some(Box::new(event_handler));
        unsafe {
            let delegate: *mut Object = msg_send![self.application, delegate];
            #[allow(deprecated)]
            let self_ptr = (*delegate).get_mut_ivar::<*const c_void>(IVAR_SELF);
            *self_ptr = &mut self as *mut Self as *const c_void;
        };
        let _: () = unsafe { msg_send![self.application, run] };
        unreachable!();
    }

    fn create_proxy(&self) -> PlatformEventLoopProxy {
        PlatformEventLoopProxy::new()
    }
}

fn send_event(event: Event) {
    let _self = unsafe {
        let app_delegate: *mut Object = msg_send![NSApp, delegate];
        #[allow(deprecated)]
        &mut *(*(*app_delegate).get_ivar::<*const c_void>(IVAR_SELF) as *mut PlatformEventLoop)
    };

    if let Some(handler) = _self.event_handler.as_mut() {
        handler(event);
    }
}

extern "C" fn app_did_finish_launching(_this: *mut Object, _sel: Sel, notification: *mut Object) {
    // Focus windows
    unsafe {
        let application: *mut Object = msg_send![notification, object];
        let _: Bool =
            msg_send![application, setActivationPolicy:NS_APPLICATION_ACTIVATION_POLICY_REGULAR];
        let _: () = msg_send![application, activateIgnoringOtherApps:true];

        let windows: *mut Object = msg_send![application, windows];
        let windows_count: usize = msg_send![windows, count];
        for i in 0..windows_count {
            let window: *mut Object = msg_send![windows, objectAtIndex:i];
            let _: () = msg_send![window, makeKeyAndOrderFront:null::<Object>()];

            // Send window created event
            send_event(Event::WindowCreated);

            // Send window resized event
            let frame: NSRect = msg_send![window, frame];
            send_event(Event::WindowResized(LogicalSize::new(
                frame.size.width as f32,
                frame.size.height as f32,
            )));
        }
    }
}

extern "C" fn app_should_terminate_after_last_window_closed(
    _this: *mut Object,
    _sel: Sel,
    _sender: *mut Object,
) -> Bool {
    Bool::YES
}

extern "C" fn app_send_event(_this: *mut Object, _sel: Sel, value: *mut Object) {
    let ptr: *mut c_void = unsafe { msg_send![value, pointerValue] };
    let event = unsafe { Box::from_raw(ptr as *mut Event) };
    send_event(*event);
}

extern "C" fn app_open_about_dialog(_this: *mut Object, _sel: Sel, _sender: *mut Object) {
    let _: () = unsafe { msg_send![NSApp, orderFrontStandardAboutPanel:null::<Object>()] };
}

// MARK: PlatformEventLoopProxy
pub(crate) struct PlatformEventLoopProxy;

impl PlatformEventLoopProxy {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl crate::EventLoopProxyInterface for PlatformEventLoopProxy {
    fn send_user_event(&self, data: String) {
        unsafe {
            let ptr = Box::leak(Box::new(Event::UserEvent(data))) as *mut Event as *mut c_void;
            let value: *mut Object = msg_send![class!(NSValue), valueWithPointer:ptr];
            let app_delegate: *mut Object = msg_send![NSApp, delegate];
            let _: () = msg_send![app_delegate, performSelectorOnMainThread:sel!(sendEvent:),
                       withObject:value,
                    waitUntilDone:Bool::NO];
        }
    }
}

// MARK: PlatformMonitor
pub(crate) struct PlatformMonitor {
    pub(crate) screen: *mut Object,
}

impl PlatformMonitor {
    pub(crate) fn new(screen: *mut Object) -> Self {
        Self { screen }
    }
}

impl crate::MonitorInterface for PlatformMonitor {
    fn name(&self) -> String {
        let name: NSString = unsafe { msg_send![self.screen, localizedName] };
        name.to_string()
    }

    fn position(&self) -> LogicalPoint {
        let frame: NSRect = unsafe { msg_send![self.screen, frame] };
        LogicalPoint::new(frame.origin.x as f32, frame.origin.y as f32)
    }

    fn size(&self) -> LogicalSize {
        let frame: NSRect = unsafe { msg_send![self.screen, frame] };
        LogicalSize::new(frame.size.width as f32, frame.size.height as f32)
    }

    fn scale_factor(&self) -> f32 {
        let backing_scale_factor: f64 = unsafe { msg_send![self.screen, backingScaleFactor] };
        backing_scale_factor as f32
    }
}

// MARK: Webview
pub(crate) struct PlatformWebview {
    window: *mut Object,
    webview: *mut Object,
}

impl PlatformWebview {
    pub(crate) fn new(builder: WebviewBuilder) -> Self {
        // Register WindowDelegate class
        if AnyClass::get(c"WindowDelegate").is_none() {
            let mut decl = ClassBuilder::new(c"WindowDelegate", class!(NSObject))
                .expect("Can't create WindowDelegate class");
            unsafe {
                decl.add_method(
                    sel!(windowDidMove:),
                    window_did_move as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(windowDidResize:),
                    window_did_resize as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(windowWillClose:),
                    window_will_close as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(webView:didStartProvisionalNavigation:),
                    webview_did_start_provisional_navigation as extern "C" fn(_, _, _, _),
                );
                decl.add_method(
                    sel!(webView:didFinishNavigation:),
                    webview_did_finish_navigation as extern "C" fn(_, _, _, _),
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

        // Create WindowDelegate instance
        let window_delegate: *mut Object = unsafe { msg_send![class!(WindowDelegate), new] };

        // Create window
        let window_rect = if builder.should_fullscreen && builder.position.is_none() {
            let screen: *mut Object = unsafe { msg_send![class!(NSScreen), mainScreen] };
            unsafe { msg_send![screen, frame] }
        } else {
            NSRect::new(
                if let Some(position) = builder.position {
                    NSPoint::new(position.x as f64, position.y as f64)
                } else {
                    NSPoint::new(0.0, 0.0)
                },
                NSSize::new(builder.size.width as f64, builder.size.height as f64),
            )
        };
        let mut window_style_mask = NS_WINDOW_STYLE_MASK_TITLED
            | NS_WINDOW_STYLE_MASK_CLOSABLE
            | NS_WINDOW_STYLE_MASK_MINIATURIZABLE;
        if builder.resizable {
            window_style_mask |= NS_WINDOW_STYLE_MASK_RESIZABLE;
        }
        if builder.should_fullscreen {
            window_style_mask = 0;
        }
        let window = unsafe {
            let window: *mut Object = msg_send![class!(NSWindow), alloc];
            let window: *mut Object = msg_send![window, initWithContentRect:window_rect, styleMask:window_style_mask, backing:NS_BACKING_STORE_BUFFERED, defer:false];
            let _: () = msg_send![window, setTitle:NSString::from_str(&builder.title)];
            if builder.should_fullscreen {
                let _: () = msg_send![window, setLevel: 25i64];
            }
            if let Some(color) = builder.background_color {
                let color: *mut Object = msg_send![class!(NSColor), colorWithRed:((color >> 16) & 0xFF) as f64 / 255.0,
                    green:((color >> 8) & 0xFF) as f64 / 255.0,
                    blue:(color & 0xFF) as f64 / 255.0, alpha:1.0];
                let _: () = msg_send![window, setBackgroundColor:color];
            }
            if builder.macos_titlebar_style == MacosTitlebarStyle::Transparent
                || builder.macos_titlebar_style == MacosTitlebarStyle::Hidden
            {
                let _: () = msg_send![window, setTitlebarAppearsTransparent:Bool::YES];
            }
            if builder.macos_titlebar_style == MacosTitlebarStyle::Hidden {
                let _: () = msg_send![window, setTitleVisibility:NS_WINDOW_TITLE_VISIBILITY_HIDDEN];
            }
            if let Some(theme) = builder.theme {
                let appearance: *mut Object = msg_send![class!(NSAppearance), appearanceNamed:match theme {
                    Theme::Light => NSAppearanceNameAqua,
                    Theme::Dark => NSAppearanceNameDarkAqua,
                }];
                let _: () = msg_send![window, setAppearance:appearance];
            }
            if let Some(min_size) = builder.min_size {
                let _: () = msg_send![window, setMinSize:NSSize::new(min_size.width as f64, min_size.height as f64)];
            }
            if builder.position.is_none() || builder.should_center {
                let screen: *mut Object = msg_send![window, screen];
                let screen_frame: NSRect = msg_send![screen, frame];
                let window_frame: NSRect = msg_send![window, frame];
                let centered_rect = NSRect::new(
                    NSPoint::new(
                        (screen_frame.size.width - window_frame.size.width) / 2.0,
                        (screen_frame.size.height - window_frame.size.height) / 2.0,
                    ),
                    window_frame.size,
                );
                let _: () = msg_send![window, setFrame:centered_rect, display:true];
            }
            #[cfg(feature = "remember_window_state")]
            if builder.remember_window_state {
                let _: Bool = msg_send![window, setFrameAutosaveName:NSString::from_str("window")];
            }
            let _: () = msg_send![window, setDelegate:window_delegate];
            window
        };

        // Create webview
        let webview = unsafe {
            let content_view: *mut Object = msg_send![window, contentView];
            let webview_rect = if builder.macos_titlebar_style == MacosTitlebarStyle::Hidden {
                let mut window_frame: NSRect = msg_send![window, frame];
                window_frame.origin.x = 0.0;
                window_frame.origin.y = 0.0;
                window_frame
            } else {
                msg_send![content_view, frame]
            };
            let webview: *mut Object = msg_send![class!(WKWebView), alloc];
            let webview: *mut Object = msg_send![webview, initWithFrame:webview_rect];
            let _: () = msg_send![webview, setNavigationDelegate:window_delegate];
            let _: () = msg_send![content_view, addSubview:webview];
            if builder.background_color.is_some() {
                let value: *mut Object = msg_send![class!(NSNumber), numberWithBool:false];
                let _: () = msg_send![webview, setValue:value, forKey:NSString::from_str("drawsBackground")];
            }
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
            let webview_configuration: *mut Object = msg_send![webview, configuration];
            let user_content_controller: *mut Object =
                msg_send![webview_configuration, userContentController];
            let user_script: *mut Object = msg_send![class!(WKUserScript), alloc];
            let user_script: *mut Object = msg_send![user_script,
                    initWithSource:NSString::from_str("window.ipc = new EventTarget();\
                        window.ipc.postMessage = message => window.webkit.messageHandlers.ipc.postMessage(typeof message !== 'string' ? JSON.stringify(message) : message);\
                        console.log = message => window.webkit.messageHandlers.console.postMessage(typeof message !== 'string' ? JSON.stringify(message) : message);"),
                    injectionTime:WK_USER_SCRIPT_INJECTION_TIME_AT_DOCUMENT_START,
                    forMainFrameOnly:true];
            let _: () = msg_send![user_content_controller, addUserScript:user_script];
            let _: () = msg_send![user_content_controller, addScriptMessageHandler:window_delegate, name:NSString::from_str("ipc")];
            let _: () = msg_send![user_content_controller, addScriptMessageHandler:window_delegate, name:NSString::from_str("console")];
        }

        Self { window, webview }
    }
}

impl crate::WebviewInterface for PlatformWebview {
    fn set_title(&mut self, title: impl AsRef<str>) {
        unsafe { msg_send![self.window, setTitle:NSString::from_str(title)] }
    }

    fn position(&self) -> LogicalPoint {
        let frame: NSRect = unsafe { msg_send![self.window, frame] };
        LogicalPoint::new(frame.origin.x as f32, frame.origin.y as f32)
    }

    fn size(&self) -> LogicalSize {
        let frame: NSRect = unsafe { msg_send![self.webview, frame] };
        LogicalSize::new(frame.size.width as f32, frame.size.height as f32)
    }

    fn set_position(&mut self, point: LogicalPoint) {
        unsafe {
            msg_send![self.window, setFrameTopLeftPoint:NSPoint::new(point.x as f64, point.y as f64)]
        }
    }

    fn set_size(&mut self, size: LogicalSize) {
        let frame: NSRect = unsafe { msg_send![self.window, frame] };
        unsafe {
            msg_send![self.window, setFrame:NSRect::new(frame.origin, NSSize::new(size.width as f64, size.height as f64)), display:true]
        }
    }

    fn set_min_size(&mut self, min_size: LogicalSize) {
        unsafe {
            msg_send![self.window, setMinSize:NSSize::new(min_size.width as f64, min_size.height as f64)]
        }
    }

    fn set_resizable(&mut self, resizable: bool) {
        let mut style_mask: u64 = unsafe { msg_send![self.window, styleMask] };
        if resizable {
            style_mask |= NS_WINDOW_STYLE_MASK_RESIZABLE;
        } else {
            style_mask &= !NS_WINDOW_STYLE_MASK_RESIZABLE;
        }
        unsafe { msg_send![self.window, setStyleMask:style_mask] }
    }

    fn set_theme(&mut self, theme: Theme) {
        unsafe {
            let appearance: *mut Object = msg_send![class!(NSAppearance), appearanceNamed:match theme {
                Theme::Light => NSAppearanceNameAqua,
                Theme::Dark => NSAppearanceNameDarkAqua,
            }];
            let _: () = msg_send![self.window, setAppearance:appearance];
        }
    }

    fn set_background_color(&mut self, color: u32) {
        unsafe {
            let color: *mut Object = msg_send![class!(NSColor), colorWithRed:((color >> 16) & 0xFF) as f64 / 255.0,
                green:((color >> 8) & 0xFF) as f64 / 255.0,
                blue:(color & 0xFF) as f64 / 255.0, alpha:1.0];
            let _: () = msg_send![self.window, setBackgroundColor:color];

            let value: *mut Object = msg_send![class!(NSNumber), numberWithBool:false];
            let _: () = msg_send![self.webview, setValue:value, forKey:NSString::from_str("drawsBackground")];
        }
    }

    fn load_url(&mut self, url: impl AsRef<str>) {
        unsafe {
            let url: *mut Object = msg_send![class!(NSURL), URLWithString:NSString::from_str(url)];
            let request: *mut Object = msg_send![class!(NSURLRequest), requestWithURL:url];
            msg_send![self.webview, loadRequest:request]
        }
    }

    fn load_html(&mut self, html: impl AsRef<str>) {
        unsafe {
            msg_send![self.webview, loadHTMLString:NSString::from_str(html), baseURL:null::<c_void>()]
        }
    }

    fn evaluate_script(&mut self, script: impl AsRef<str>) {
        unsafe {
            msg_send![self.webview, evaluateJavaScript:NSString::from_str(script), completionHandler:null::<Object>()]
        }
    }
}

extern "C" fn window_did_move(_this: *mut Object, _sel: Sel, notification: *mut Object) {
    // Send window moved event
    let window: *mut Object = unsafe { msg_send![notification, object] };
    let frame: NSRect = unsafe { msg_send![window, frame] };
    send_event(Event::WindowMoved(LogicalPoint::new(
        frame.origin.x as f32,
        frame.origin.y as f32,
    )));
}

extern "C" fn window_did_resize(_this: *mut Object, _sel: Sel, notification: *mut Object) {
    let window: *mut Object = unsafe { msg_send![notification, object] };
    let content_view: *mut Object = unsafe { msg_send![window, contentView] };
    let subviews: *mut Object = unsafe { msg_send![content_view, subviews] };

    // Update webview size
    let title_visibility: i64 = unsafe { msg_send![window, titleVisibility] };
    let webview_rect = if title_visibility == NS_WINDOW_TITLE_VISIBILITY_HIDDEN {
        let mut webview_rect: NSRect = unsafe { msg_send![window, frame] };
        webview_rect.origin.x = 0.0;
        webview_rect.origin.y = 0.0;
        webview_rect
    } else {
        unsafe { msg_send![content_view, frame] }
    };
    let webview: *mut Object = unsafe { msg_send![subviews, objectAtIndex:0u64] };
    let _: () = unsafe { msg_send![webview, setFrame:webview_rect] };

    // Send window resized event
    let frame: NSRect = unsafe { msg_send![webview, frame] };
    send_event(Event::WindowResized(LogicalSize::new(
        frame.size.width as f32,
        frame.size.height as f32,
    )));
}

extern "C" fn window_will_close(_this: *mut Object, _sel: Sel, _notification: *mut Object) {
    // Send window closed event
    send_event(Event::WindowClosed);
}

extern "C" fn webview_did_start_provisional_navigation(
    _this: *mut Object,
    _sel: Sel,
    _webview: *mut Object,
    _navigation: *mut Object,
) {
    // Send page load started event
    send_event(Event::PageLoadStarted);
}

extern "C" fn webview_did_finish_navigation(
    _this: *mut Object,
    _sel: Sel,
    _webview: *mut Object,
    _navigation: *mut Object,
) {
    // Send page load finished event
    send_event(Event::PageLoadFinished);
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

    if name == "ipc" {
        // Send ipc message received event
        send_event(Event::PageMessageReceived(body));
    } else if name == "console" {
        // Print console message
        println!("{}", body);
    }
}
