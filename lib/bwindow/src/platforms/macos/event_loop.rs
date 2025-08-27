/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CString, c_void};
use std::ptr::null;

use objc2::runtime::{AnyObject, Bool, ClassBuilder, Sel};
use objc2::{class, msg_send, sel};

use super::cocoa::*;
use crate::dpi::LogicalSize;
use crate::event::Event;
use crate::platforms::PlatformMonitor;

const IVAR_SELF: &str = "_self";

// MARK: EventLoop
pub(crate) struct PlatformEventLoop {
    application: *mut AnyObject,
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
            decl.add_method(sel!(sendEvent:), app_send_event as extern "C" fn(_, _, _));
            decl.add_method(
                sel!(openAboutDialog:),
                app_open_about_dialog as extern "C" fn(_, _, _),
            );
        }
        decl.register();

        // Create AppDelegate instance
        let app_delegate: *mut AnyObject = unsafe { msg_send![class!(AppDelegate), new] };

        // Get application
        let application = unsafe {
            let application: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
            let _: () = msg_send![application, setDelegate:app_delegate];
            application
        };

        // Create menu
        unsafe {
            let app_name: NSString = msg_send![application, valueForKey:NSString::from_str("name")];

            let menubar: *mut AnyObject = msg_send![class!(NSMenu), new];
            let _: () = msg_send![application, setMainMenu:menubar];

            // App menu
            let app_menu_item: *mut AnyObject = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![menubar, addItem:app_menu_item];
            let app_menu: *mut AnyObject = msg_send![class!(NSMenu), new];
            let _: () = msg_send![app_menu_item, setSubmenu:app_menu];

            let _: *mut AnyObject = msg_send![app_menu,
                addItemWithTitle:NSString::from_str(format!("About {app_name}")),
                action:sel!(openAboutDialog:),
                keyEquivalent:NSString::from_str("")
            ];

            let separator_item: *mut AnyObject = msg_send![class!(NSMenuItem), separatorItem];
            let _: () = msg_send![app_menu, addItem:separator_item];

            let services_menu_item: *mut AnyObject = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![services_menu_item, setTitle:NSString::from_str("Services")];
            let _: () = msg_send![app_menu, addItem:services_menu_item];
            let services_menu: *mut AnyObject = msg_send![class!(NSMenu), new];
            let _: () = msg_send![services_menu_item, setSubmenu:services_menu];
            let _: () = msg_send![application, setServicesMenu:services_menu];

            let separator_item: *mut AnyObject = msg_send![class!(NSMenuItem), separatorItem];
            let _: () = msg_send![app_menu, addItem:separator_item];

            let _: *mut AnyObject = msg_send![app_menu,
                addItemWithTitle:NSString::from_str(format!("Hide {app_name}")),
                action:sel!(hide:),
                keyEquivalent:NSString::from_str("h")
            ];
            let hide_others_menu_item: *mut AnyObject = msg_send![app_menu,
                addItemWithTitle:NSString::from_str("Hide Others"),
                action:sel!(hideOtherApplications:),
                keyEquivalent:NSString::from_str("h")
            ];
            let _: () = msg_send![hide_others_menu_item, setKeyEquivalentModifierMask:NS_EVENT_MODIFIER_FLAG_OPTION | NS_EVENT_MODIFIER_FLAG_COMMAND];
            let _: *mut AnyObject = msg_send![app_menu,
                addItemWithTitle:NSString::from_str("Show All"),
                action:sel!(unhideAllApplications:),
                keyEquivalent:NSString::from_str("")];

            let separator_item: *mut AnyObject = msg_send![class!(NSMenuItem), separatorItem];
            let _: () = msg_send![app_menu, addItem:separator_item];

            let _: *mut AnyObject = msg_send![app_menu,
                addItemWithTitle:NSString::from_str(format!("Quit {app_name}")),
                action:sel!(terminate:),
                keyEquivalent:NSString::from_str("q")];

            // File menu
            let file_menu_item: *mut AnyObject = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![file_menu_item, setTitle:NSString::from_str("File")];
            let _: () = msg_send![menubar, addItem:file_menu_item];
            let file_menu: *mut AnyObject = msg_send![class!(NSMenu), new];
            let _: () = msg_send![file_menu_item, setSubmenu:file_menu];

            let _: *mut AnyObject = msg_send![file_menu,
                addItemWithTitle:NSString::from_str("Close Window"),
                action:sel!(performClose:),
                keyEquivalent:NSString::from_str("w")];

            // Edit menu
            let edit_menu_item: *mut AnyObject = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![edit_menu_item, setTitle:NSString::from_str("Edit")];
            let _: () = msg_send![menubar, addItem:edit_menu_item];
            let edit_menu: *mut AnyObject = msg_send![class!(NSMenu), new];
            let _: () = msg_send![edit_menu_item, setSubmenu:edit_menu];

            let _: *mut AnyObject = msg_send![edit_menu,
                addItemWithTitle:NSString::from_str("Undo"),
                action:sel!(undo:),
                keyEquivalent:NSString::from_str("z")];
            let _: *mut AnyObject = msg_send![edit_menu,
                addItemWithTitle:NSString::from_str("Redo"),
                action:sel!(redo:),
                keyEquivalent:NSString::from_str("Z")];

            let separator_item: *mut AnyObject = msg_send![class!(NSMenuItem), separatorItem];
            let _: () = msg_send![edit_menu, addItem:separator_item];

            let _: *mut AnyObject = msg_send![edit_menu,
                addItemWithTitle:NSString::from_str("Cut"),
                action:sel!(cut:),
                keyEquivalent:NSString::from_str("x")];
            let _: *mut AnyObject = msg_send![edit_menu,
                addItemWithTitle:NSString::from_str("Copy"),
                action:sel!(copy:),
                keyEquivalent:NSString::from_str("c")];
            let _: *mut AnyObject = msg_send![edit_menu,
                addItemWithTitle:NSString::from_str("Paste"),
                action:sel!(paste:),
                keyEquivalent:NSString::from_str("v")];
            let _: *mut AnyObject = msg_send![edit_menu,
                addItemWithTitle:NSString::from_str("Delete"),
                action:sel!(delete:),
                keyEquivalent:NSString::from_str("")];
            let _: *mut AnyObject = msg_send![edit_menu,
                addItemWithTitle:NSString::from_str("Select All"),
                action:sel!(selectAll:),
                keyEquivalent:NSString::from_str("a")];

            // Window menu
            let window_menu_item: *mut AnyObject = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![window_menu_item, setTitle:NSString::from_str("Window")];
            let _: () = msg_send![menubar, addItem:window_menu_item];
            let window_menu: *mut AnyObject = msg_send![class!(NSMenu), new];
            let _: () = msg_send![window_menu_item, setSubmenu:window_menu];
            let _: () = msg_send![application, setWindowsMenu:window_menu];

            let _: *mut AnyObject = msg_send![window_menu,
                addItemWithTitle:NSString::from_str("Minimize"),
                action:sel!(performMiniaturize:),
                keyEquivalent:NSString::from_str("m")];
            let _: *mut AnyObject = msg_send![window_menu,
                addItemWithTitle:NSString::from_str("Zoom"),
                action:sel!(performZoom:),
                keyEquivalent:NSString::from_str("")];

            // Help menu
            let help_menu_item: *mut AnyObject = msg_send![class!(NSMenuItem), new];
            let _: () = msg_send![help_menu_item, setTitle:NSString::from_str("Help")];
            let _: () = msg_send![menubar, addItem:help_menu_item];
            let help_menu: *mut AnyObject = msg_send![class!(NSMenu), new];
            let _: () = msg_send![help_menu_item, setSubmenu:help_menu];
            let _: () = msg_send![application, setHelpMenu:help_menu];
        }

        Self {
            application,
            event_handler: None,
        }
    }

    pub(crate) fn available_monitors(&self) -> Vec<PlatformMonitor> {
        let mut monitors = Vec::new();
        unsafe {
            let screens: *mut AnyObject = msg_send![class!(NSScreen), screens];
            let count: usize = msg_send![screens, count];
            for i in 0..count {
                let screen: *mut AnyObject = msg_send![screens, objectAtIndex:i];
                monitors.push(PlatformMonitor::new(screen));
            }
        }
        monitors
    }

    pub(crate) fn primary_monitor(&self) -> PlatformMonitor {
        unsafe {
            let screen: *mut AnyObject = msg_send![class!(NSScreen), mainScreen];
            PlatformMonitor::new(screen)
        }
    }

    pub(crate) fn run(mut self, event_handler: impl FnMut(Event) + 'static) -> ! {
        self.event_handler = Some(Box::new(event_handler));
        unsafe {
            let delegate: *mut AnyObject = msg_send![self.application, delegate];
            #[allow(deprecated)]
            let self_ptr = (*delegate).get_mut_ivar::<*const c_void>(IVAR_SELF);
            *self_ptr = &mut self as *mut Self as *const c_void;
        };
        let _: () = unsafe { msg_send![self.application, run] };
        unreachable!();
    }

    pub(crate) fn create_proxy(&self) -> PlatformEventLoopProxy {
        PlatformEventLoopProxy::new()
    }
}

pub(crate) fn send_event(event: Event) {
    let _self = unsafe {
        let app_delegate: *mut AnyObject = msg_send![NSApp, delegate];
        #[allow(deprecated)]
        &mut *(*(*app_delegate).get_ivar::<*const c_void>(IVAR_SELF) as *mut PlatformEventLoop)
    };

    if let Some(handler) = _self.event_handler.as_mut() {
        handler(event);
    }
}

extern "C" fn app_did_finish_launching(
    _this: *mut AnyObject,
    _sel: Sel,
    notification: *mut AnyObject,
) {
    // Focus windows
    unsafe {
        let application: *mut AnyObject = msg_send![notification, object];
        let _: Bool =
            msg_send![application, setActivationPolicy:NS_APPLICATION_ACTIVATION_POLICY_REGULAR];
        let _: () = msg_send![application, activateIgnoringOtherApps:true];

        let windows: *mut AnyObject = msg_send![application, windows];
        let windows_count: usize = msg_send![windows, count];
        for i in 0..windows_count {
            let window: *mut AnyObject = msg_send![windows, objectAtIndex:i];
            let _: () = msg_send![window, makeKeyAndOrderFront:null::<AnyObject>()];

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

extern "C" fn app_send_event(_this: *mut AnyObject, _sel: Sel, value: *mut AnyObject) {
    let ptr: *mut c_void = unsafe { msg_send![value, pointerValue] };
    let event = unsafe { Box::from_raw(ptr as *mut Event) };
    send_event(*event);
}

extern "C" fn app_open_about_dialog(_this: *mut AnyObject, _sel: Sel, _sender: *mut AnyObject) {
    let _: () = unsafe { msg_send![NSApp, orderFrontStandardAboutPanel:null::<AnyObject>()] };
}

// MARK: PlatformEventLoopProxy
pub(crate) struct PlatformEventLoopProxy;

impl PlatformEventLoopProxy {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn send_user_event(&self, data: Vec<u8>) {
        unsafe {
            let ptr = Box::leak(Box::new(Event::UserEvent(data))) as *mut Event as *mut c_void;
            let value: *mut AnyObject = msg_send![class!(NSValue), valueWithPointer:ptr];
            let app_delegate: *mut AnyObject = msg_send![NSApp, delegate];
            let _: () = msg_send![app_delegate, performSelectorOnMainThread:sel!(sendEvent:),
                       withObject:value,
                    waitUntilDone:Bool::NO];
        }
    }
}
