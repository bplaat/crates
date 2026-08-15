/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A native macOS document-based image viewer.

#![allow(unsafe_code)]

mod checkerboard;
mod cocoa;
mod svg;
mod window_controller;

use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::null_mut;

use checkerboard::create_checkerboard_view;
use cocoa::*;
use macview_appkit::{Point, Rect, Size, create_tinyvg_view, decode_image, decode_tinyvg};
use objc2::ffi::{objc_msgSendSuper, objc_super};
use objc2::rc::autoreleasepool;
use objc2::runtime::{AnyClass, AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};
use svg::{Svg, create_svg_view, is_svg, parse_svg};
use window_controller::create_window_controller;

struct DocumentIvars {
    image: Cell<*mut Object>,
    tinyvg: Cell<*mut tinyvg::Document>,
    svg: Cell<*mut Svg>,
    svg_view: Cell<*mut Object>,
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

        #[unsafe(method(printOperationWithSettings:error:))]
        fn _print_operation_with_settings(
            &self,
            settings: *mut Object,
            _: *mut c_void,
        ) -> *mut Object {
            self.print_operation(settings)
        }

        #[unsafe(method(dealloc))]
        fn _dealloc(&self) {
            self.dealloc();
        }
    }
);

impl Document {
    fn read_from_data(&self, data: *mut Object, error_out: *mut c_void) -> Bool {
        // SAFETY: AppKit passes a valid NSData object whose bytes remain alive for this call.
        let bytes = unsafe {
            let length: usize = msg_send![data, length];
            let bytes: *const c_void = msg_send![data, bytes];
            std::slice::from_raw_parts(bytes.cast::<u8>(), length)
        };
        if tinyvg::is_tinyvg(bytes) {
            // SAFETY: AppKit supplied data as a valid NSData for the duration of this call.
            let document = match unsafe { decode_tinyvg(data) } {
                Ok(document) => document,
                Err(error) => {
                    set_error(error_out, &error.to_string());
                    return Bool::NO;
                }
            };
            self.set_media(null_mut(), Box::into_raw(Box::new(document)), null_mut());
            return Bool::YES;
        }
        if is_svg(bytes) {
            self.set_media(
                null_mut(),
                null_mut(),
                Box::into_raw(Box::new(parse_svg(bytes))),
            );
            return Bool::YES;
        }

        // SAFETY: AppKit supplied data as a valid NSData for the duration of this call.
        let (image, _) = match unsafe { decode_image(data) } {
            Ok(image) => image,
            Err(error) => {
                set_error(error_out, &error);
                return Bool::NO;
            }
        };
        self.set_media(image, null_mut(), null_mut());
        Bool::YES
    }

    /// Replaces the loaded media, releasing whatever the document held before.
    ///
    /// The web view of the previous media goes with it, because it draws a page that belongs to
    /// media the document no longer holds.
    fn set_media(&self, image: *mut Object, tinyvg: *mut tinyvg::Document, svg: *mut Svg) {
        let old_image = self.ivars().image.replace(image);
        let old_tinyvg = self.ivars().tinyvg.replace(tinyvg);
        let old_svg = self.ivars().svg.replace(svg);
        let old_svg_view = self.ivars().svg_view.replace(null_mut());
        // SAFETY: The ivars own the retained objects and the Boxes converted into these pointers.
        unsafe {
            if !old_image.is_null() {
                let _: () = msg_send![old_image, release];
            }
            if !old_svg_view.is_null() {
                let _: () = msg_send![old_svg_view, release];
            }
            if !old_tinyvg.is_null() {
                drop(Box::from_raw(old_tinyvg));
            }
            if !old_svg.is_null() {
                drop(Box::from_raw(old_svg));
            }
        }
    }

    /// Returns the natural size of the loaded media, or `None` when the document is empty.
    fn media_size(&self) -> Option<Size> {
        let tinyvg = self.ivars().tinyvg.get();
        if !tinyvg.is_null() {
            // SAFETY: The ivar owns the TinyVG document until dealloc.
            let document = unsafe { &*tinyvg };
            return Some(Size {
                width: document.size.width,
                height: document.size.height,
            });
        }
        let svg = self.ivars().svg.get();
        if !svg.is_null() {
            // SAFETY: The ivar owns the SVG document until dealloc.
            return Some(unsafe { &*svg }.size);
        }
        let image = self.ivars().image.get();
        // SAFETY: The ivar owns a live NSImage when it is not null.
        (!image.is_null()).then(|| unsafe { msg_send![image, size] })
    }

    /// Creates an owned view that draws the loaded media inside `frame`.
    ///
    /// The caller owns the returned view and must send it `release`.
    fn create_media_view(&self, frame: Rect) -> *mut Object {
        let tinyvg = self.ivars().tinyvg.get();
        let svg = self.ivars().svg.get();
        // SAFETY: The ivars own the media, and NSImageView retains the image it is given. The web
        // view is retained a second time because printing draws the loaded page again.
        unsafe {
            if !tinyvg.is_null() {
                return create_tinyvg_view(frame, Box::new((*tinyvg).clone()));
            }
            if !svg.is_null() {
                let view = create_svg_view(frame, &*svg);
                let old_view = self.ivars().svg_view.replace(msg_send![view, retain]);
                if !old_view.is_null() {
                    let _: () = msg_send![old_view, release];
                }
                return view;
            }
            let image_view: *mut Object = msg_send![class!(NSImageView), alloc];
            let image_view: *mut Object = msg_send![image_view, initWithFrame: frame];
            let _: () = msg_send![image_view, setImage: self.ivars().image.get()];
            let _: () = msg_send![image_view,
                setImageScaling: NS_IMAGE_SCALE_PROPORTIONALLY_UP_OR_DOWN
            ];
            image_view
        }
    }

    fn make_window_controllers(&self) {
        let Some(media_size) = self.media_size() else {
            return;
        };
        let image = self.ivars().image.get();
        let title_size = if image.is_null() {
            media_size
        } else {
            // SAFETY: The ivar owns a live NSImage that keeps its representations alive.
            unsafe { image_pixel_size(image, media_size) }
        };

        // SAFETY: All objects are valid AppKit instances, selectors use their documented ABI,
        // and ownership is balanced after the document retains its window controller.
        unsafe {
            let content_size = Size {
                width: media_size.width.clamp(320.0, 1200.0),
                height: media_size.height.clamp(240.0, 800.0),
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

            let checkerboard = create_checkerboard_view(rect);
            let media_view = self.create_media_view(rect);
            let _: () = msg_send![media_view,
                setAutoresizingMask: NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE
            ];
            let _: () = msg_send![checkerboard, addSubview: media_view];
            let _: () = msg_send![media_view, release];
            let _: () = msg_send![checkerboard,
                setAutoresizingMask: NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE
            ];
            let _: () = msg_send![window, setContentView: checkerboard];
            let _: () = msg_send![checkerboard, release];

            let controller = create_window_controller(window, title_size);
            let this = self as *const Self as *mut Object;
            let _: () = msg_send![this, addWindowController: controller];
            let _: () = msg_send![controller, release];
            let _: () = msg_send![window, release];
        }
    }

    /// Creates a print operation that draws the media at its natural size, scaled to fit one page.
    fn print_operation(&self, settings: *mut Object) -> *mut Object {
        let Some(media_size) = self.media_size() else {
            return null_mut();
        };
        // SAFETY: All objects are valid AppKit instances. The print info copy and the media view
        // are released after the returned operation retains them.
        unsafe {
            let this = self as *const Self as *mut Object;
            let print_info: *mut Object = msg_send![this, printInfo];
            let print_info: *mut Object = msg_send![print_info, copy];
            let attributes: *mut Object = msg_send![print_info, dictionary];
            let _: () = msg_send![attributes, addEntriesFromDictionary: settings];
            let _: () = msg_send![print_info,
                setHorizontalPagination: NS_PRINTING_PAGINATION_MODE_FIT
            ];
            let _: () =
                msg_send![print_info, setVerticalPagination: NS_PRINTING_PAGINATION_MODE_FIT];
            let _: () = msg_send![print_info, setHorizontallyCentered: Bool::YES];
            let _: () = msg_send![print_info, setVerticallyCentered: Bool::YES];

            // WebKit paginates the loaded page itself, and only the view in the window has it.
            let svg_view = self.ivars().svg_view.get();
            if !svg_view.is_null() {
                let operation: *mut Object =
                    msg_send![svg_view, printOperationWithPrintInfo: print_info];
                let _: () = msg_send![print_info, release];
                return operation;
            }

            let view = self.create_media_view(Rect {
                origin: Point { x: 0.0, y: 0.0 },
                size: media_size,
            });
            let operation: *mut Object = msg_send![class!(NSPrintOperation),
                printOperationWithView: view,
                printInfo: print_info
            ];
            let _: () = msg_send![view, release];
            let _: () = msg_send![print_info, release];
            operation
        }
    }

    fn dealloc(&self) {
        self.set_media(null_mut(), null_mut(), null_mut());
        // SAFETY: objc_msgSendSuper invokes NSDocument's dealloc with the exact Objective-C
        // deallocation ABI.
        unsafe {
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

/// Returns the pixel dimensions of the largest representation of an image.
///
/// An `NSImage` reports its size in points, which is smaller than the stored pixels for images
/// that carry a resolution above 72 dpi, so the title shows the representation sizes instead.
///
/// # Safety
///
/// `image` must point to a valid `NSImage` for the duration of this call.
unsafe fn image_pixel_size(image: *mut Object, fallback: Size) -> Size {
    // SAFETY: The caller supplies a live NSImage that owns its representations.
    unsafe {
        let representations: *mut Object = msg_send![image, representations];
        let count: usize = msg_send![representations, count];
        let mut size = Size {
            width: 0.0,
            height: 0.0,
        };
        for index in 0..count {
            let representation: *mut Object = msg_send![representations, objectAtIndex: index];
            let width: isize = msg_send![representation, pixelsWide];
            let height: isize = msg_send![representation, pixelsHigh];
            if width > 0 && height > 0 && (width * height) as f64 > size.width * size.height {
                size = Size {
                    width: width as f64,
                    height: height as f64,
                };
            }
        }
        if size.width > 0.0 { size } else { fallback }
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
        add_separator(file_menu);
        add_item(
            file_menu,
            "Print...",
            sel!(printDocument:),
            "p",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        add_separator(file_menu);
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
