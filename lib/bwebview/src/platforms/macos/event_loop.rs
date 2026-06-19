/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::null;

use objc2::rc::autoreleasepool;
use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};

use super::cocoa::*;
use super::webkit::*;
use crate::{Event, EventLoopBuilder, LogicalPoint, LogicalSize, WindowEvent};

// MARK: AppDelegate
struct AppDelegateIvars {
    event_loop: Cell<*mut PlatformEventLoop>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = AppDelegateIvars]
    struct AppDelegate;

    impl AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn _did_finish_launching(&self, notification: *mut Object) { self.did_finish_launching(notification); }

        #[unsafe(method(applicationShouldTerminateAfterLastWindowClosed:))]
        fn _should_terminate(&self, _: *mut Object) -> Bool { Bool::YES }

        #[unsafe(method(sendEvent:))]
        fn _send_event(&self, value: *mut Object) { self.send_event(value); }

        #[unsafe(method(openAboutDialog:))]
        fn _open_about_dialog(&self, _: *mut Object) { self.open_about_dialog(); }
    }
);

impl AppDelegate {
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
        let event = unsafe { Box::from_raw(ptr as *mut Event) };
        send_event(*event);
    }

    fn open_about_dialog(&self) {
        let _: () = unsafe { msg_send![NSApp, orderFrontStandardAboutPanel:null::<Object>()] };
    }
}

// MARK: EventLoop
pub(crate) struct PlatformEventLoop {
    application: *mut Object,
    event_handler: Option<Box<dyn FnMut(Event) + 'static>>,
}

impl PlatformEventLoop {
    pub(crate) fn new(_builder: EventLoopBuilder) -> Self {
        // Create AppDelegate instance (registers class lazily on first call)
        let app_delegate: *mut Object = unsafe { msg_send![AppDelegate::class(), new] };

        // Get application
        let application = unsafe {
            let application: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let _: () = msg_send![application, setDelegate:app_delegate];
            application
        };

        // Create menu
        unsafe {
            let app_name: NSString = msg_send![application, valueForKey:ns_string!("name")];

            let menubar: *mut Object = msg_send![class!(NSMenu), new];
            let _: () = msg_send![application, setMainMenu:menubar];

            // App menu
            let app_menu_item: *mut Object = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![menubar, addItem:app_menu_item];
            let app_menu: *mut Object = msg_send![class!(NSMenu), new];
            let _: () = msg_send![app_menu_item, setSubmenu:app_menu];

            let _: *mut Object = msg_send![app_menu,
                addItemWithTitle:NSString::from_str(format!("About {app_name}")),
                action:sel!(openAboutDialog:),
                keyEquivalent:ns_string!("")
            ];

            let separator_item: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
            let _: () = msg_send![app_menu, addItem:separator_item];

            let services_menu_item: *mut Object = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![services_menu_item, setTitle:ns_string!("Services")];
            let _: () = msg_send![app_menu, addItem:services_menu_item];
            let services_menu: *mut Object = msg_send![class!(NSMenu), new];
            let _: () = msg_send![services_menu_item, setSubmenu:services_menu];
            let _: () = msg_send![application, setServicesMenu:services_menu];

            let separator_item: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
            let _: () = msg_send![app_menu, addItem:separator_item];

            let _: *mut Object = msg_send![app_menu,
                addItemWithTitle:NSString::from_str(format!("Hide {app_name}")),
                action:sel!(hide:),
                keyEquivalent:ns_string!("h")
            ];
            let hide_others_menu_item: *mut Object = msg_send![app_menu,
                addItemWithTitle:ns_string!("Hide Others"),
                action:sel!(hideOtherApplications:),
                keyEquivalent:ns_string!("h")
            ];
            let _: () = msg_send![hide_others_menu_item, setKeyEquivalentModifierMask:NS_EVENT_MODIFIER_FLAG_OPTION | NS_EVENT_MODIFIER_FLAG_COMMAND];
            let _: *mut Object = msg_send![app_menu,
                addItemWithTitle:ns_string!("Show All"),
                action:sel!(unhideAllApplications:),
                keyEquivalent:ns_string!("")];

            let separator_item: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
            let _: () = msg_send![app_menu, addItem:separator_item];

            let _: *mut Object = msg_send![app_menu,
                addItemWithTitle:NSString::from_str(format!("Quit {app_name}")),
                action:sel!(terminate:),
                keyEquivalent:ns_string!("q")];

            // File menu
            let file_menu_item: *mut Object = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![file_menu_item, setTitle:ns_string!("File")];
            let _: () = msg_send![menubar, addItem:file_menu_item];
            let file_menu: *mut Object = msg_send![class!(NSMenu), new];
            let _: () = msg_send![file_menu_item, setSubmenu:file_menu];

            let _: *mut Object = msg_send![file_menu,
                addItemWithTitle:ns_string!("Close Window"),
                action:sel!(performClose:),
                keyEquivalent:ns_string!("w")];

            // Edit menu
            let edit_menu_item: *mut Object = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![edit_menu_item, setTitle:ns_string!("Edit")];
            let _: () = msg_send![menubar, addItem:edit_menu_item];
            let edit_menu: *mut Object = msg_send![class!(NSMenu), new];
            let _: () = msg_send![edit_menu_item, setSubmenu:edit_menu];

            let _: *mut Object = msg_send![edit_menu,
                addItemWithTitle:ns_string!("Undo"),
                action:sel!(undo:),
                keyEquivalent:ns_string!("z")];
            let _: *mut Object = msg_send![edit_menu,
                addItemWithTitle:ns_string!("Redo"),
                action:sel!(redo:),
                keyEquivalent:ns_string!("Z")];

            let separator_item: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
            let _: () = msg_send![edit_menu, addItem:separator_item];

            let _: *mut Object = msg_send![edit_menu,
                addItemWithTitle:ns_string!("Cut"),
                action:sel!(cut:),
                keyEquivalent:ns_string!("x")];
            let _: *mut Object = msg_send![edit_menu,
                addItemWithTitle:ns_string!("Copy"),
                action:sel!(copy:),
                keyEquivalent:ns_string!("c")];
            let _: *mut Object = msg_send![edit_menu,
                addItemWithTitle:ns_string!("Paste"),
                action:sel!(paste:),
                keyEquivalent:ns_string!("v")];
            let _: *mut Object = msg_send![edit_menu,
                addItemWithTitle:ns_string!("Delete"),
                action:sel!(delete:),
                keyEquivalent:ns_string!("")];
            let _: *mut Object = msg_send![edit_menu,
                addItemWithTitle:ns_string!("Select All"),
                action:sel!(selectAll:),
                keyEquivalent:ns_string!("a")];

            // Window menu
            let window_menu_item: *mut Object = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![window_menu_item, setTitle:ns_string!("Window")];
            let _: () = msg_send![menubar, addItem:window_menu_item];
            let window_menu: *mut Object = msg_send![class!(NSMenu), new];
            let _: () = msg_send![window_menu_item, setSubmenu:window_menu];
            let _: () = msg_send![application, setWindowsMenu:window_menu];

            let _: *mut Object = msg_send![window_menu,
                addItemWithTitle:ns_string!("Minimize"),
                action:sel!(performMiniaturize:),
                keyEquivalent:ns_string!("m")];
            let _: *mut Object = msg_send![window_menu,
                addItemWithTitle:ns_string!("Zoom"),
                action:sel!(performZoom:),
                keyEquivalent:ns_string!("")];

            // Help menu
            let help_menu_item: *mut Object = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![help_menu_item, setTitle:ns_string!("Help")];
            let _: () = msg_send![menubar, addItem:help_menu_item];
            let help_menu: *mut Object = msg_send![class!(NSMenu), new];
            let _: () = msg_send![help_menu_item, setSubmenu:help_menu];
            let _: () = msg_send![application, setHelpMenu:help_menu];
        }

        Self {
            application,
            event_handler: None,
        }
    }
}

impl crate::EventLoopInterface for PlatformEventLoop {
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

    fn run(mut self, event_handler: impl FnMut(Event) + 'static) -> ! {
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

pub(crate) fn send_event(event: Event) {
    let _self = unsafe {
        let app_delegate: *mut Object = msg_send![NSApp, delegate];
        let delegate_ref = &*(app_delegate as *const AppDelegate);
        &mut *delegate_ref.ivars().event_loop.get()
    };

    if let Some(handler) = _self.event_handler.as_mut() {
        handler(event);
    }
}

// MARK: EventLoopProxy
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

// MARK: Monitor
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

    fn is_primary(&self) -> bool {
        let main_screen: *mut Object = unsafe { msg_send![class!(NSScreen), mainScreen] };
        self.screen == main_screen
    }
}
