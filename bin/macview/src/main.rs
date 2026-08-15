/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A native macOS document-based image viewer.

#![allow(unsafe_code)]

mod cocoa;

use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::null_mut;

use cocoa::*;
use macview_appkit::{Point, Rect, Size, decode_image};
use objc2::ffi::{objc_msgSendSuper, objc_super};
use objc2::rc::autoreleasepool;
use objc2::runtime::{AnyClass, AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};

struct DocumentIvars {
    image: Cell<*mut Object>,
}

define_class!(
    #[unsafe(super(NSDocument))]
    #[name = "MacViewDocument"]
    #[ivars = DocumentIvars]
    struct Document;

    impl Document {
        #[unsafe(method(readFromData:ofType:error:))]
        fn _read_from_data(
            &self,
            data: *mut Object,
            _: *mut Object,
            error: *mut c_void,
        ) -> Bool {
            self.read_from_data(data, error)
        }

        #[unsafe(method(makeWindowControllers))]
        fn _make_window_controllers(&self) {
            self.make_window_controllers();
        }

        #[unsafe(method(dealloc))]
        fn _dealloc(&self) {
            self.dealloc();
        }
    }
);

impl Document {
    fn read_from_data(&self, data: *mut Object, error_out: *mut c_void) -> Bool {
        // SAFETY: AppKit supplied data as a valid NSData for the duration of this call.
        let (image, _) = match unsafe { decode_image(data) } {
            Ok(image) => image,
            Err(error) => {
                set_error(error_out, &error);
                return Bool::NO;
            }
        };
        let old_image = self.ivars().image.replace(image);
        if !old_image.is_null() {
            // SAFETY: The ivar owns the retained NSImage created by make_image.
            unsafe {
                let _: () = msg_send![old_image, release];
            }
        }
        Bool::YES
    }

    fn make_window_controllers(&self) {
        let image = self.ivars().image.get();
        if image.is_null() {
            return;
        }

        // SAFETY: All objects are valid AppKit instances, selectors use their documented ABI,
        // and ownership is balanced after the document retains its window controller.
        unsafe {
            let image_size: Size = msg_send![image, size];
            let content_size = Size {
                width: image_size.width.clamp(320.0, 1200.0),
                height: image_size.height.clamp(240.0, 800.0),
            };
            let rect = Rect {
                origin: Point { x: 0.0, y: 0.0 },
                size: content_size,
            };
            let style = NS_WINDOW_STYLE_MASK_TITLED
                | NS_WINDOW_STYLE_MASK_CLOSABLE
                | NS_WINDOW_STYLE_MASK_MINIATURIZABLE
                | NS_WINDOW_STYLE_MASK_RESIZABLE;
            let window: *mut Object = msg_send![class!(NSWindow), alloc];
            let window: *mut Object = msg_send![window,
                initWithContentRect: rect,
                styleMask: style,
                backing: NS_BACKING_STORE_BUFFERED,
                defer: Bool::NO
            ];
            let _: () = msg_send![window, setContentMinSize: Size { width: 240.0, height: 180.0 }];
            let _: () = msg_send![window, center];

            let image_view: *mut Object = msg_send![class!(NSImageView), alloc];
            let image_view: *mut Object = msg_send![image_view, initWithFrame: rect];
            let _: () = msg_send![image_view, setImage: image];
            let _: () = msg_send![image_view,
                setImageScaling: NS_IMAGE_SCALE_PROPORTIONALLY_UP_OR_DOWN
            ];
            let _: () = msg_send![image_view,
                setAutoresizingMask: NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE
            ];
            let _: () = msg_send![window, setContentView: image_view];
            let _: () = msg_send![image_view, release];

            let controller: *mut Object = msg_send![class!(NSWindowController), alloc];
            let controller: *mut Object = msg_send![controller, initWithWindow: window];
            let this = self as *const Self as *mut Object;
            let _: () = msg_send![this, addWindowController: controller];
            let _: () = msg_send![controller, release];
            let _: () = msg_send![window, release];

            let _: () = msg_send![image, release];
            self.ivars().image.set(null_mut());
        }
    }

    fn dealloc(&self) {
        let image = self.ivars().image.get();
        // SAFETY: The ivar owns image when non-null. objc_msgSendSuper invokes NSDocument's
        // dealloc with the exact Objective-C deallocation ABI.
        unsafe {
            if !image.is_null() {
                let _: () = msg_send![image, release];
            }
            let super_info = objc_super {
                receiver: self as *const Self as *mut Object,
                super_class: class!(NSDocument).cast::<AnyClass>(),
            };
            let send: unsafe extern "C" fn(*const objc_super, *const c_void) =
                std::mem::transmute(objc_msgSendSuper as *const c_void);
            send(&super_info, sel!(dealloc).0);
        }
    }
}

define_class!(
    #[unsafe(super(NSObject))]
    struct AppDelegate;

    impl AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn _did_finish_launching(&self, notification: *mut Object) {
            self.did_finish_launching(notification);
        }

        #[unsafe(method(applicationShouldOpenUntitledFile:))]
        const fn _should_open_untitled_file(&self, _: *mut Object) -> Bool {
            Bool::NO
        }

        #[unsafe(method(applicationShouldTerminateAfterLastWindowClosed:))]
        const fn _should_terminate_after_last_window(&self, _: *mut Object) -> Bool {
            Bool::NO
        }
    }
);

impl AppDelegate {
    fn did_finish_launching(&self, notification: *mut Object) {
        // SAFETY: AppKit supplies a valid notification containing NSApplication.
        unsafe {
            let application: *mut Object = msg_send![notification, object];
            let _: () = msg_send![application, activateIgnoringOtherApps: Bool::YES];
        }
    }
}

fn set_error(error_out: *mut c_void, description: &str) {
    if error_out.is_null() {
        return;
    }
    // SAFETY: error_out is a non-null NSError** supplied by NSDocument. All Objective-C
    // objects are autoreleased and remain valid for the current event cycle.
    unsafe {
        let domain = ns_string("nl.bplaat.MacView");
        let description_key = ns_string("NSLocalizedDescription");
        let description = ns_string(description);
        let user_info: *mut Object = msg_send![class!(NSDictionary),
            dictionaryWithObject: description,
            forKey: description_key
        ];
        let error: *mut Object = msg_send![class!(NSError),
            errorWithDomain: domain,
            code: 1isize,
            userInfo: user_info
        ];
        error_out.cast::<*mut Object>().write(error);
    }
}

fn menu_item(
    title: &str,
    action: objc2::runtime::Sel,
    key: &str,
    modifiers: u64,
    target: *mut Object,
) -> *mut Object {
    // SAFETY: NSMenuItem's designated initializer accepts these NSStrings and selector.
    unsafe {
        let item: *mut Object = msg_send![class!(NSMenuItem), alloc];
        let item: *mut Object = msg_send![item,
            initWithTitle: ns_string(title),
            action: action,
            keyEquivalent: ns_string(key)
        ];
        let _: () = msg_send![item, setKeyEquivalentModifierMask: modifiers];
        if !target.is_null() {
            let _: () = msg_send![item, setTarget: target];
        }
        msg_send![item, autorelease]
    }
}

fn add_menu(main_menu: *mut Object, title: &str) -> *mut Object {
    // SAFETY: main_menu is a valid NSMenu. It retains the item, which retains the submenu.
    unsafe {
        let item: *mut Object = msg_send![class!(NSMenuItem), new];
        if !title.is_empty() {
            let _: () = msg_send![item, setTitle: ns_string(title)];
        }
        let menu: *mut Object = msg_send![class!(NSMenu), alloc];
        let menu: *mut Object = msg_send![menu, initWithTitle: ns_string(title)];
        let _: () = msg_send![item, setSubmenu: menu];
        let _: () = msg_send![main_menu, addItem: item];
        let _: () = msg_send![item, release];
        menu
    }
}

fn add_item(
    menu: *mut Object,
    title: &str,
    action: objc2::runtime::Sel,
    key: &str,
    modifiers: u64,
    target: *mut Object,
) {
    // SAFETY: menu is a valid NSMenu and retains the autoreleased menu item.
    unsafe {
        let _: () = msg_send![menu,
            addItem: menu_item(title, action, key, modifiers, target)
        ];
    }
}

fn add_separator(menu: *mut Object) {
    // SAFETY: menu is a valid NSMenu and separatorItem returns a valid shared menu item.
    unsafe {
        let separator: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![menu, addItem: separator];
    }
}

fn create_menu(application: *mut Object) {
    // SAFETY: application is the shared NSApplication and all menu ownership transfers follow
    // AppKit's retain conventions.
    unsafe {
        let main_menu: *mut Object = msg_send![class!(NSMenu), new];

        let app_menu = add_menu(main_menu, "");
        add_item(
            app_menu,
            "About MacView",
            sel!(orderFrontStandardAboutPanel:),
            "",
            0,
            application,
        );
        add_separator(app_menu);
        let services_item: *mut Object = msg_send![class!(NSMenuItem), new];
        let _: () = msg_send![services_item, setTitle: ns_string("Services")];
        let services_menu: *mut Object = msg_send![class!(NSMenu), new];
        let _: () = msg_send![services_item, setSubmenu: services_menu];
        let _: () = msg_send![app_menu, addItem: services_item];
        let _: () = msg_send![application, setServicesMenu: services_menu];
        let _: () = msg_send![services_menu, release];
        let _: () = msg_send![services_item, release];
        add_separator(app_menu);
        add_item(
            app_menu,
            "Hide MacView",
            sel!(hide:),
            "h",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        add_item(
            app_menu,
            "Hide Others",
            sel!(hideOtherApplications:),
            "h",
            NS_EVENT_MODIFIER_FLAG_COMMAND | NS_EVENT_MODIFIER_FLAG_OPTION,
            null_mut(),
        );
        add_item(
            app_menu,
            "Show All",
            sel!(unhideAllApplications:),
            "",
            0,
            null_mut(),
        );
        add_separator(app_menu);
        add_item(
            app_menu,
            "Quit MacView",
            sel!(terminate:),
            "q",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        let _: () = msg_send![app_menu, release];

        let file_menu = add_menu(main_menu, "File");
        add_item(
            file_menu,
            "Open...",
            sel!(openDocument:),
            "o",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        add_item(
            file_menu,
            "Close Window",
            sel!(performClose:),
            "w",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        let _: () = msg_send![file_menu, release];

        let edit_menu = add_menu(main_menu, "Edit");
        add_item(
            edit_menu,
            "Undo",
            sel!(undo:),
            "z",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        add_item(
            edit_menu,
            "Redo",
            sel!(redo:),
            "z",
            NS_EVENT_MODIFIER_FLAG_COMMAND | NS_EVENT_MODIFIER_FLAG_SHIFT,
            null_mut(),
        );
        add_separator(edit_menu);
        add_item(
            edit_menu,
            "Cut",
            sel!(cut:),
            "x",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        add_item(
            edit_menu,
            "Copy",
            sel!(copy:),
            "c",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        add_item(
            edit_menu,
            "Paste",
            sel!(paste:),
            "v",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        add_item(edit_menu, "Delete", sel!(delete:), "", 0, null_mut());
        add_item(
            edit_menu,
            "Select All",
            sel!(selectAll:),
            "a",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        let _: () = msg_send![edit_menu, release];

        let window_menu = add_menu(main_menu, "Window");
        add_item(
            window_menu,
            "Minimize",
            sel!(performMiniaturize:),
            "m",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        add_item(window_menu, "Zoom", sel!(performZoom:), "", 0, null_mut());
        let _: () = msg_send![application, setWindowsMenu: window_menu];
        let _: () = msg_send![window_menu, release];

        let help_menu = add_menu(main_menu, "Help");
        let _: () = msg_send![application, setHelpMenu: help_menu];
        let _: () = msg_send![help_menu, release];

        let _: () = msg_send![application, setMainMenu: main_menu];
        let _: () = msg_send![main_menu, release];
    }
}

fn main() {
    // SAFETY: The shared application and registered delegate/document classes remain alive for
    // the entire AppKit run loop. Selectors use their documented Objective-C signatures.
    autoreleasepool(|_| unsafe {
        let _ = Document::class();
        let application: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let _: Bool = msg_send![application,
            setActivationPolicy: NS_APPLICATION_ACTIVATION_POLICY_REGULAR
        ];
        let delegate: *mut Object = msg_send![AppDelegate::class(), new];
        let _: () = msg_send![application, setDelegate: delegate];
        create_menu(application);
        let _: () = msg_send![application, run];
        let _: () = msg_send![delegate, release];
    });
}
