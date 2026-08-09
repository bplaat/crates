/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::Cell;
use std::ffi::c_void;
use std::path::PathBuf;
use std::ptr::null;

use objc2::rc::autoreleasepool;
use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};

use super::cocoa::*;
use super::menu::create_menu_bar;
#[cfg(feature = "webview")]
use super::webkit::*;
use crate::{CloseRequest, Event, EventLoopBuilder, LogicalPoint, LogicalSize, Theme, WindowEvent};

// MARK: AppDelegate
struct AppDelegateIvars {
    event_loop: Cell<*mut PlatformEventLoop>,
    allow_termination: Cell<bool>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = AppDelegateIvars]
    struct AppDelegate;

    impl AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn _did_finish_launching(&self, notification: *mut Object) { self.did_finish_launching(notification); }

        #[unsafe(method(applicationShouldTerminateAfterLastWindowClosed:))]
        const fn _should_terminate(&self, _: *mut Object) -> Bool { Bool::YES }

        #[unsafe(method(applicationShouldTerminate:))]
        fn _application_should_terminate(&self, _: *mut Object) -> u64 { self.application_should_terminate() }

        #[unsafe(method(application:openURLs:))]
        fn _open_urls(&self, _: *mut Object, urls: *mut Object) { self.open_urls(urls); }

        #[unsafe(method(sendEvent:))]
        fn _send_event(&self, value: *mut Object) { self.send_event(value); }

        #[unsafe(method(openAboutDialog:))]
        fn _open_about_dialog(&self, _: *mut Object) { self.open_about_dialog(); }

        #[cfg(feature = "menu")]
        #[unsafe(method(menuItemSelected:))]
        fn _menu_item_selected(&self, sender: *mut Object) { self.menu_item_selected(sender); }
    }
);

impl AppDelegate {
    fn application_should_terminate(&self) -> u64 {
        if self.ivars().allow_termination.get() {
            1
        } else {
            let request = CloseRequest::new();
            send_event(Event::Window(WindowEvent::CloseRequested(request.clone())));
            u64::from(!request.is_prevented())
        }
    }

    fn did_finish_launching(&self, notification: *mut Object) {
        unsafe {
            let application: *mut Object = msg_send![notification, object];
            let _: Bool = msg_send![application, setActivationPolicy:NS_APPLICATION_ACTIVATION_POLICY_REGULAR];
            let _: () = msg_send![application, activateIgnoringOtherApps:true];

            let windows: *mut Object = msg_send![application, windows];
            let windows_count: usize = msg_send![windows, count];
            for i in 0..windows_count {
                let window: *mut Object = msg_send![windows, objectAtIndex:i];
                let _: () = msg_send![window, makeKeyAndOrderFront:null::<Object>()];
                send_event(Event::Window(WindowEvent::Create));
            }
        }
    }

    fn send_event(&self, value: *mut Object) {
        let ptr: *mut c_void = unsafe { msg_send![value, pointerValue] };
        let event = unsafe { Box::from_raw(ptr as *mut Event<'static>) };
        send_event(*event);
    }

    fn open_urls(&self, urls: *mut Object) {
        let mut paths = Vec::new();
        unsafe {
            let count: usize = msg_send![urls, count];
            for index in 0..count {
                let url: *mut Object = msg_send![urls, objectAtIndex:index];
                let is_file_url: Bool = msg_send![url, isFileURL];
                if is_file_url == Bool::YES {
                    let path: NSString = msg_send![url, path];
                    paths.push(PathBuf::from(path.to_string()));
                }
            }
        }
        if !paths.is_empty() {
            send_event(Event::MacosOpenFiles(paths));
        }
    }

    fn open_about_dialog(&self) {
        let _: () = unsafe { msg_send![NSApp, orderFrontStandardAboutPanel:null::<Object>()] };
    }

    #[cfg(feature = "menu")]
    fn menu_item_selected(&self, sender: *mut Object) {
        let action: NSString = unsafe { msg_send![sender, representedObject] };
        send_event(Event::MacosMenuItem(action.to_string()));
    }
}

// MARK: EventLoop
pub(crate) struct PlatformEventLoop {
    application: *mut Object,
    event_handler: Option<crate::EventHandler>,
}

impl PlatformEventLoop {
    pub(crate) fn new(mut builder: EventLoopBuilder) -> Self {
        // Create AppDelegate instance (registers class lazily on first call)
        let app_delegate: *mut Object = unsafe { msg_send![AppDelegate::class(), new] };

        // Get application
        let application = unsafe {
            let application: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let _: () = msg_send![application, setDelegate:app_delegate];
            application
        };

        // Create menu
        unsafe { create_menu_bar(application, app_delegate, &mut builder) };

        Self {
            application,
            event_handler: None,
        }
    }
}

// MARK: Theme
fn system_theme() -> Theme {
    unsafe {
        let application: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let appearance: *mut Object = msg_send![application, effectiveAppearance];
        let name: NSString = msg_send![appearance, name];
        if name.to_string().contains("Dark") {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

impl crate::EventLoopInterface for PlatformEventLoop {
    fn theme(&self) -> Theme {
        system_theme()
    }

    fn primary_monitor(&self) -> PlatformMonitor {
        unsafe {
            let screen: *mut Object = msg_send![class!(NSScreen), mainScreen];
            PlatformMonitor::new(screen)
        }
    }

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

    fn run(mut self, event_handler: impl for<'a> FnMut(Event<'a>) + 'static) -> ! {
        self.event_handler = Some(Box::new(event_handler));
        autoreleasepool(|_| unsafe {
            let delegate: *mut Object = msg_send![self.application, delegate];
            let delegate_ref = &*(delegate as *const AppDelegate);
            delegate_ref
                .ivars()
                .event_loop
                .set(&mut self as *mut PlatformEventLoop);
            let _: () = msg_send![self.application, run];
        });
        unreachable!()
    }

    fn create_proxy(&self) -> PlatformEventLoopProxy {
        PlatformEventLoopProxy::new()
    }
}

pub(crate) fn send_event(event: Event<'_>) {
    let event_loop = unsafe {
        let app_delegate: *mut Object = msg_send![NSApp, delegate];
        let delegate_ref = &*(app_delegate as *const AppDelegate);
        delegate_ref.ivars().event_loop.get()
    };

    let Some(_self) = (unsafe { event_loop.as_mut() }) else {
        return;
    };

    if let Some(handler) = _self.event_handler.as_mut() {
        handler(event);
    }
}

pub(super) fn allow_termination_if_last_window(closing_window: *mut Object) {
    let app_delegate: *mut Object = unsafe { msg_send![NSApp, delegate] };
    let app_delegate = unsafe { &*(app_delegate as *const AppDelegate) };
    let windows: *mut Object = unsafe { msg_send![NSApp, windows] };
    let count: usize = unsafe { msg_send![windows, count] };
    let has_other_visible_window = (0..count).any(|index| unsafe {
        let window: *mut Object = msg_send![windows, objectAtIndex:index];
        let visible: Bool = msg_send![window, isVisible];
        window != closing_window && visible == Bool::YES
    });
    if !has_other_visible_window {
        app_delegate.ivars().allow_termination.set(true);
    }
}

// MARK: EventLoopProxy
pub(crate) struct PlatformEventLoopProxy;

impl PlatformEventLoopProxy {
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl crate::EventLoopProxyInterface for PlatformEventLoopProxy {
    fn send_user_event(&self, data: crate::UserEvent) {
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

// MARK: Monitor
pub(crate) struct PlatformMonitor {
    pub(crate) screen: *mut Object,
}

impl PlatformMonitor {
    pub(crate) const fn new(screen: *mut Object) -> Self {
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

    fn is_primary(&self) -> bool {
        let main_screen: *mut Object = unsafe { msg_send![class!(NSScreen), mainScreen] };
        self.screen == main_screen
    }
}
