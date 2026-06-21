/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! BImg - a native macOS QOI image viewer

#![allow(unsafe_code)]
#![cfg(target_os = "macos")]

use std::cell::Cell;
use std::ffi::{c_char, c_void};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};

use objc2::rc::autoreleasepool;
use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{Encode, Encoding, class, define_class, extern_class, msg_send, sel};

// MARK: Cocoa bindings

#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    static __CFConstantStringClassReference: Object;
}

#[link(name = "Cocoa", kind = "framework")]
unsafe extern "C" {}

#[derive(Clone, Copy)]
#[repr(C)]
struct NSPoint {
    x: f64,
    y: f64,
}
impl NSPoint {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
// SAFETY: NSPoint/CGPoint is a C struct with two f64 fields; its ObjC encoding is {CGPoint=dd}.
unsafe impl Encode for NSPoint {
    const ENCODING: Encoding = Encoding::Struct("CGPoint", &[f64::ENCODING, f64::ENCODING]);
}

#[derive(Clone, Copy)]
#[repr(C)]
struct NSSize {
    width: f64,
    height: f64,
}
impl NSSize {
    fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}
// SAFETY: NSSize/CGSize is a C struct with two f64 fields; its ObjC encoding is {CGSize=dd}.
unsafe impl Encode for NSSize {
    const ENCODING: Encoding = Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
}

#[derive(Clone, Copy)]
#[repr(C)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}
impl NSRect {
    fn new(origin: NSPoint, size: NSSize) -> Self {
        Self { origin, size }
    }
}
// SAFETY: NSRect/CGRect is a C struct {origin: CGPoint, size: CGSize}; encoding is {CGRect={CGPoint=dd}{CGSize=dd}}.
unsafe impl Encode for NSRect {
    const ENCODING: Encoding =
        Encoding::Struct("CGRect", &[NSPoint::ENCODING, NSSize::ENCODING]);
}

const NS_APPLICATION_ACTIVATION_POLICY_REGULAR: i64 = 0;
const NS_WINDOW_STYLE_MASK_TITLED: u64 = 1 << 0;
const NS_WINDOW_STYLE_MASK_CLOSABLE: u64 = 1 << 1;
const NS_WINDOW_STYLE_MASK_MINIATURIZABLE: u64 = 1 << 2;
const NS_WINDOW_STYLE_MASK_RESIZABLE: u64 = 1 << 3;
const NS_BACKING_STORE_BUFFERED: u64 = 2;
const NS_EVENT_MODIFIER_FLAG_OPTION: u64 = 1 << 19;
const NS_EVENT_MODIFIER_FLAG_COMMAND: u64 = 1 << 20;
const NS_IMAGE_SCALING_NONE: u64 = 2;
const NS_UTF8_STRING_ENCODING: u64 = 4;
const NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_PRIMARY: u64 = 1 << 7;

// CFConstString mirrors the layout of Apple's __CFConstantString.
// Statics of this type in __DATA,__cfstring are recognised by dyld as NSString literals.
#[repr(C)]
struct CFConstString {
    isa: *const c_void,
    cfinfo: u32,
    #[cfg(target_pointer_width = "64")]
    _rc: u32,
    data: *const u8,
    len: usize,
}
// SAFETY: CFConstString is a static immutable NSString literal; it is safe to share across threads.
unsafe impl Send for CFConstString {}
// SAFETY: CFConstString is a static immutable NSString literal; it is safe to share across threads.
unsafe impl Sync for CFConstString {}

// Creates a zero-cost NSString literal. Must be ASCII with no NUL bytes.
// NOTE: Do not call inside closures - hoist to enclosing function scope (rustc bug: madsmtm/objc2#258).
macro_rules! ns_string {
    ($s:expr) => {{
        const INPUT: &str = $s;
        const BYTES: &[u8] = INPUT.as_bytes();
        const _: () = {
            let mut i = 0usize;
            while i < BYTES.len() {
                if !BYTES[i].is_ascii() || BYTES[i] == b'\0' {
                    panic!("ns_string! only supports ASCII strings without NUL bytes");
                }
                i += 1;
            }
        };
        #[unsafe(link_section = "__TEXT,__cstring,cstring_literals")]
        static DATA: [u8; BYTES.len() + 1] = {
            let mut arr = [0u8; BYTES.len() + 1];
            let mut i = 0usize;
            while i < BYTES.len() {
                arr[i] = BYTES[i];
                i += 1;
            }
            arr
        };
        // SAFETY: CFConstString layout matches Apple's __CFConstantString; isa/data/len are
        // valid static references initialized from compile-time constants.
        #[unsafe(link_section = "__DATA,__cfstring")]
        static CFSTRING: CFConstString = unsafe {
            CFConstString {
                isa: &__CFConstantStringClassReference as *const Object as *const c_void,
                cfinfo: 0x07C8,
                #[cfg(target_pointer_width = "64")]
                _rc: 0,
                data: DATA.as_ptr(),
                len: BYTES.len(),
            }
        };
        &CFSTRING as *const CFConstString as *mut Object
    }};
}

#[repr(transparent)]
struct NSString(*mut Object);

// SAFETY: NSString is an ObjC object pointer; its encoding is '@'.
unsafe impl Encode for NSString {
    const ENCODING: Encoding = Encoding::Object;
}

impl NSString {
    fn from_str(s: impl AsRef<str>) -> Self {
        let s = s.as_ref();
        // SAFETY: NSString is a valid Foundation class; initWithBytes:length:encoding: is a standard initializer.
        unsafe {
            let obj: *mut Object = msg_send![class!(NSString), alloc];
            let obj: *mut Object = msg_send![obj,
                initWithBytes: s.as_ptr() as *const c_void,
                length: s.len() as u64,
                encoding: NS_UTF8_STRING_ENCODING
            ];
            let obj: *mut Object = msg_send![obj, autorelease];
            Self(obj)
        }
    }

    fn to_string_lossy(&self) -> String {
        // SAFETY: self.0 is a valid NSString; UTF8String returns a C string pointer valid until
        // the NSString is deallocated.
        unsafe {
            let bytes: *const c_char = msg_send![self.0, UTF8String];
            let len: usize =
                msg_send![self.0, lengthOfBytesUsingEncoding: NS_UTF8_STRING_ENCODING];
            String::from_utf8_lossy(std::slice::from_raw_parts(bytes as *const u8, len))
                .into_owned()
        }
    }
}

// MARK: Extern class declarations

extern_class!(
    #[unsafe(super(NSObject))]
    #[name = "NSClipView"]
    struct NSClipView;
);

extern_class!(
    #[unsafe(super(NSObject))]
    #[name = "NSWindowController"]
    struct NSWindowController;
);

extern_class!(
    #[unsafe(super(NSObject))]
    #[name = "NSDocument"]
    struct NSDocument;
);

// MARK: Open Recent menu

// Pointer to the Open Recent submenu; set once during menu construction so that
// menuWillOpen: can identify it and populate it from NSDocumentController.
static OPEN_RECENT_MENU: AtomicPtr<Object> = AtomicPtr::new(null_mut());

// MARK: Image creation

fn create_nsimage(pixels: &[u8], width: u32, height: u32) -> *mut Object {
    // SAFETY: NSBitmapImageRep is a valid AppKit class. initWithBitmapDataPlanes:NULL allocates
    // an internal pixel buffer; we then copy our pixels into it via bitmapData.
    unsafe {
        let rep: *mut Object = msg_send![class!(NSBitmapImageRep), alloc];
        let rep: *mut Object = msg_send![rep,
            initWithBitmapDataPlanes: null_mut::<c_void>(),
            pixelsWide: width as i64,
            pixelsHigh: height as i64,
            bitsPerSample: 8i64,
            samplesPerPixel: 4i64,
            hasAlpha: Bool::YES,
            isPlanar: Bool::NO,
            colorSpaceName: NSString::from_str("NSCalibratedRGBColorSpace"),
            bytesPerRow: (width as i64) * 4,
            bitsPerPixel: 32i64
        ];
        let bitmap_data: *mut c_void = msg_send![rep, bitmapData];
        std::ptr::copy_nonoverlapping(pixels.as_ptr(), bitmap_data as *mut u8, pixels.len());

        let image: *mut Object = msg_send![class!(NSImage), alloc];
        let image: *mut Object =
            msg_send![image, initWithSize: NSSize::new(width as f64, height as f64)];
        let _: () = msg_send![image, addRepresentation: rep];
        let _: () = msg_send![rep, release];
        image
    }
}

// MARK: CenteringClipView

// Overrides NSClipView.constrainBoundsRect: to center the document view when it is smaller
// than the visible area instead of pinning it to the top-left corner.
define_class!(
    #[unsafe(super(NSClipView))]
    struct CenteringClipView;

    impl CenteringClipView {
        #[unsafe(method(constrainBoundsRect:))]
        fn _constrain_bounds_rect(&self, proposed: NSRect) -> NSRect {
            self.constrain_bounds_rect(proposed)
        }
    }
);

impl CenteringClipView {
    fn constrain_bounds_rect(&self, proposed: NSRect) -> NSRect {
        // SAFETY: self is a valid CenteringClipView (NSClipView subclass); documentView and frame
        // are standard NSView properties that are safe to read at any time.
        unsafe {
            let this = self as *const Self as *mut Object;
            let doc_view: *mut Object = msg_send![this, documentView];
            if doc_view.is_null() {
                return proposed;
            }
            let doc: NSRect = msg_send![doc_view, frame];
            let mut b = proposed;
            if b.size.width > doc.size.width {
                b.origin.x = ((doc.size.width - b.size.width) / 2.0).floor();
            } else {
                b.origin.x = b.origin.x.max(0.0).min(doc.size.width - b.size.width);
            }
            if b.size.height > doc.size.height {
                b.origin.y = ((doc.size.height - b.size.height) / 2.0).floor();
            } else {
                b.origin.y = b.origin.y.max(0.0).min(doc.size.height - b.size.height);
            }
            b
        }
    }
}

// MARK: QoiWindowController

struct QoiWindowControllerIvars {
    scroll_view: Cell<*mut Object>,
    image_width: Cell<f64>,
    image_height: Cell<f64>,
    fit_mode: Cell<bool>,
}

define_class!(
    #[unsafe(super(NSWindowController))]
    #[name = "QoiWindowController"]
    #[ivars = QoiWindowControllerIvars]
    struct QoiWindowController;

    impl QoiWindowController {
        #[unsafe(method(windowDidResize:))]
        fn _window_did_resize(&self, notification: *mut Object) {
            self.window_did_resize(notification);
        }

        #[unsafe(method(windowDidEndLiveResize:))]
        fn _window_did_end_live_resize(&self, _notification: *mut Object) {
            if self.ivars().fit_mode.get() {
                zoom_to_fit(self.ivars());
            }
        }

        #[unsafe(method(zoomIn:))]
        fn _zoom_in(&self, _sender: *mut Object) {
            // SAFETY: scroll_view is a valid NSScrollView set in QoiDocument::make_window_controllers.
            unsafe {
                let sv = self.ivars().scroll_view.get();
                if sv.is_null() {
                    return;
                }
                let current: f64 = msg_send![sv, magnification];
                let _: () = msg_send![sv, setMagnification: (current * 1.5_f64).min(32.0_f64)];
                self.ivars().fit_mode.set(false);
            }
        }

        #[unsafe(method(zoomOut:))]
        fn _zoom_out(&self, _sender: *mut Object) {
            // SAFETY: scroll_view is a valid NSScrollView set in QoiDocument::make_window_controllers.
            unsafe {
                let sv = self.ivars().scroll_view.get();
                if sv.is_null() {
                    return;
                }
                let current: f64 = msg_send![sv, magnification];
                let _: () = msg_send![sv, setMagnification: (current / 1.5_f64).max(0.02_f64)];
                self.ivars().fit_mode.set(false);
            }
        }

        #[unsafe(method(actualSize:))]
        fn _actual_size(&self, _sender: *mut Object) {
            // SAFETY: scroll_view is a valid NSScrollView set in QoiDocument::make_window_controllers.
            unsafe {
                let sv = self.ivars().scroll_view.get();
                if sv.is_null() {
                    return;
                }
                let _: () = msg_send![sv, setMagnification: 1.0_f64];
                let clip: *mut Object = msg_send![sv, contentView];
                let bounds: NSRect = msg_send![clip, bounds];
                let cx = ((self.ivars().image_width.get() - bounds.size.width) / 2.0).max(0.0);
                let cy = ((self.ivars().image_height.get() - bounds.size.height) / 2.0).max(0.0);
                let _: () = msg_send![clip, scrollToPoint: NSPoint::new(cx, cy)];
                let _: () = msg_send![sv, reflectScrolledClipView: clip];
                self.ivars().fit_mode.set(false);
            }
        }

        #[unsafe(method(zoomToFit:))]
        fn _zoom_to_fit(&self, _sender: *mut Object) {
            zoom_to_fit(self.ivars());
        }
    }
);

impl QoiWindowController {
    fn window_did_resize(&self, notification: *mut Object) {
        // SAFETY: notification.object is the NSWindow; inLiveResize is a safe NSWindow property.
        let in_live: i8 = unsafe {
            let window: *mut Object = msg_send![notification, object];
            msg_send![window, inLiveResize]
        };
        if in_live != 0 {
            return;
        }
        if self.ivars().fit_mode.get() {
            zoom_to_fit(self.ivars());
        }
    }
}

fn zoom_to_fit(ivars: &QoiWindowControllerIvars) {
    let scroll_view = ivars.scroll_view.get();
    if scroll_view.is_null() {
        return;
    }
    // SAFETY: scroll_view is a valid NSScrollView retained by the window's view hierarchy.
    unsafe {
        let window: *mut Object = msg_send![scroll_view, window];
        if window.is_null() {
            return;
        }
        let frame: NSRect = msg_send![window, frame];
        let content: NSRect = msg_send![window, contentRectForFrameRect: frame];
        let vw = content.size.width;
        let vh = content.size.height;
        let iw = ivars.image_width.get();
        let ih = ivars.image_height.get();
        if vw <= 0.0 || vh <= 0.0 || iw <= 0.0 || ih <= 0.0 {
            return;
        }
        let scale = (vw / iw).min(vh / ih).clamp(0.02, 32.0);
        let _: () = msg_send![scroll_view, setMagnification: scale];
        // Trigger constrainBoundsRect: so CenteringClipView re-centers.
        let clip: *mut Object = msg_send![scroll_view, contentView];
        let _: () = msg_send![clip, scrollToPoint: NSPoint::new(0.0, 0.0)];
        let _: () = msg_send![scroll_view, reflectScrolledClipView: clip];
        ivars.fit_mode.set(true);
    }
}

// MARK: QoiDocument

struct QoiDocumentIvars {
    image: Cell<*mut Object>,
}

define_class!(
    #[unsafe(super(NSDocument))]
    #[name = "QoiDocument"]
    #[ivars = QoiDocumentIvars]
    struct QoiDocument;

    impl QoiDocument {
        // NSDocument override: decode the raw file data into an NSImage stored in ivars.
        // outError encoding ^^@ is matched via the ^v relaxation in enc_match.
        #[unsafe(method(readFromData:ofType:error:))]
        fn _read_from_data(
            &self,
            data: *mut Object,
            _type_name: *mut Object,
            _out_error: *mut c_void,
        ) -> Bool {
            self.read_from_data(data)
        }

        // NSDocument override: create the window and window controller for this document.
        #[unsafe(method(makeWindowControllers))]
        fn _make_window_controllers(&self) {
            self.make_window_controllers();
        }
    }
);

impl QoiDocument {
    fn read_from_data(&self, data: *mut Object) -> Bool {
        // SAFETY: data is a valid NSData object; bytes/length are standard NSData accessors.
        let (pixels, width, height) = unsafe {
            let bytes: *const c_void = msg_send![data, bytes];
            let length: usize = msg_send![data, length];
            let slice = std::slice::from_raw_parts(bytes as *const u8, length);
            match qoi::decode_to_vec(slice) {
                Err(_) => return Bool::NO,
                Ok((header, pixels_raw)) => {
                    let pixels: Vec<u8> = if header.channels == qoi::Channels::Rgb {
                        pixels_raw
                            .chunks_exact(3)
                            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                            .collect()
                    } else {
                        pixels_raw
                    };
                    (pixels, header.width, header.height)
                }
            }
        };
        let ns_image = create_nsimage(&pixels, width, height);
        self.ivars().image.set(ns_image);
        Bool::YES
    }

    fn make_window_controllers(&self) {
        // SAFETY: All ObjC classes used here are valid AppKit classes. Objects created with
        // alloc/init have retain count 1; once passed to Cocoa (setContentView:, addWindowController:,
        // etc.) the framework retains them, and we release our initial +1.
        unsafe {
            let this = self as *const QoiDocument as *mut Object;

            let ns_image = self.ivars().image.get();
            if ns_image.is_null() {
                return;
            }
            let size: NSSize = msg_send![ns_image, size];
            let image_width = size.width;
            let image_height = size.height;

            // Compute window size: clamp to 85% of the visible screen, min 480x360.
            let screen: *mut Object = msg_send![class!(NSScreen), mainScreen];
            let screen_frame: NSRect = msg_send![screen, visibleFrame];
            let max_w = (screen_frame.size.width * 0.85).min(1440.0);
            let max_h = (screen_frame.size.height * 0.85).min(1080.0);
            let win_w = image_width.max(480.0).min(max_w);
            let win_h = image_height.max(360.0).min(max_h);

            let style = NS_WINDOW_STYLE_MASK_TITLED
                | NS_WINDOW_STYLE_MASK_CLOSABLE
                | NS_WINDOW_STYLE_MASK_MINIATURIZABLE
                | NS_WINDOW_STYLE_MASK_RESIZABLE;
            let win_frame =
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(win_w, win_h));
            let window: *mut Object = msg_send![class!(NSWindow), alloc];
            let window: *mut Object = msg_send![window,
                initWithContentRect: win_frame,
                styleMask: style,
                backing: NS_BACKING_STORE_BUFFERED,
                defer: Bool::NO
            ];
            // setTitleWithRepresentedFilename: sets both the title and the proxy icon.
            let file_url: *mut Object = msg_send![this, fileURL];
            if !file_url.is_null() {
                let path: NSString = msg_send![file_url, path];
                let _: () = msg_send![window, setTitleWithRepresentedFilename: path];
            }
            let _: () = msg_send![window, center];
            let _: () = msg_send![window,
                setCollectionBehavior: NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_PRIMARY
            ];

            // Build scroll view with centering clip view and image view.
            let content_rect: NSRect = msg_send![window, contentRectForFrameRect: win_frame];
            let scroll_frame = NSRect::new(NSPoint::new(0.0, 0.0), content_rect.size);

            let scroll_view: *mut Object = msg_send![class!(NSScrollView), alloc];
            let scroll_view: *mut Object =
                msg_send![scroll_view, initWithFrame: scroll_frame];
            let _: () = msg_send![scroll_view, setHasHorizontalScroller: Bool::YES];
            let _: () = msg_send![scroll_view, setHasVerticalScroller: Bool::YES];
            let _: () = msg_send![scroll_view, setAutohidesScrollers: Bool::YES];
            let _: () = msg_send![scroll_view, setAllowsMagnification: Bool::YES];
            let _: () = msg_send![scroll_view, setMinMagnification: 0.02f64];
            let _: () = msg_send![scroll_view, setMaxMagnification: 32.0f64];
            let bg: *mut Object =
                msg_send![class!(NSColor), colorWithWhite: 0.18f64, alpha: 1.0f64];
            let _: () = msg_send![scroll_view, setBackgroundColor: bg];

            // Replace the default NSClipView with our centering subclass.
            let clip_frame: NSRect = msg_send![scroll_view, bounds];
            let centering_clip: *mut Object = msg_send![CenteringClipView::class(), alloc];
            let centering_clip: *mut Object =
                msg_send![centering_clip, initWithFrame: clip_frame];
            let _: () = msg_send![scroll_view, setContentView: centering_clip];
            let _: () = msg_send![centering_clip, release];

            let img_frame = NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(image_width, image_height),
            );
            let image_view: *mut Object = msg_send![class!(NSImageView), alloc];
            let image_view: *mut Object = msg_send![image_view, initWithFrame: img_frame];
            let _: () = msg_send![image_view, setImage: ns_image];
            let _: () = msg_send![image_view, setImageScaling: NS_IMAGE_SCALING_NONE];
            let _: () = msg_send![scroll_view, setDocumentView: image_view];
            let _: () = msg_send![image_view, release];

            // NSImageView now retains ns_image; release our +1 from create_nsimage.
            let _: () = msg_send![ns_image, release];
            self.ivars().image.set(null_mut());

            let _: () = msg_send![window, setContentView: scroll_view];
            let _: () = msg_send![scroll_view, release]; // window retains it

            // Create QoiWindowController and wire it up.
            let wc: *mut Object = msg_send![QoiWindowController::class(), alloc];
            let wc: *mut Object = msg_send![wc, initWithWindow: window]; // NSWindowController retains window
            // NSWindowController.initWithWindow: does NOT set itself as window delegate.
            let _: () = msg_send![window, setDelegate: wc];
            let _: () = msg_send![window, release]; // our +1; NSWindowController holds +1

            let wc_ref = &*(wc as *const QoiWindowController);
            wc_ref.ivars().scroll_view.set(scroll_view);
            wc_ref.ivars().image_width.set(image_width);
            wc_ref.ivars().image_height.set(image_height);
            wc_ref.ivars().fit_mode.set(true);

            // addWindowController: causes the document to retain wc and call showWindow: later.
            let _: () = msg_send![this, addWindowController: wc];
            let _: () = msg_send![wc, release]; // document holds +1

            // Initial zoom to fit before the window is shown.
            zoom_to_fit(wc_ref.ivars());
        }
    }
}

// MARK: AppDelegate

define_class!(
    #[unsafe(super(NSObject))]
    struct AppDelegate;

    impl AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn _did_finish_launching(&self, notification: *mut Object) {
            self.did_finish_launching(notification);
        }

        #[unsafe(method(applicationShouldTerminateAfterLastWindowClosed:))]
        fn _should_terminate_after_last_window_closed(&self, _sender: *mut Object) -> Bool {
            Bool::YES
        }

        #[unsafe(method(applicationShouldOpenUntitledFile:))]
        fn _should_open_untitled_file(&self, _app: *mut Object) -> Bool {
            Bool::YES
        }

        #[unsafe(method(applicationOpenUntitledFile:))]
        fn _open_untitled_file(&self, _app: *mut Object) -> Bool {
            // SAFETY: NSDocumentController is a valid AppKit singleton.
            unsafe {
                let dc: *mut Object =
                    msg_send![class!(NSDocumentController), sharedDocumentController];
                let _: () = msg_send![dc, openDocument: null_mut::<Object>()];
            }
            Bool::YES
        }

        #[unsafe(method(application:openFile:))]
        fn _open_file(&self, _app: *mut Object, filename: *mut Object) -> Bool {
            // SAFETY: NSURL and NSDocumentController are valid Foundation/AppKit classes.
            unsafe {
                let url: *mut Object =
                    msg_send![class!(NSURL), fileURLWithPath: NSString(filename)];
                let dc: *mut Object =
                    msg_send![class!(NSDocumentController), sharedDocumentController];
                // openDocumentWithContentsOfURL:display:error: (deprecated but block-free).
                let _: *mut Object = msg_send![dc,
                    openDocumentWithContentsOfURL: url,
                    display: Bool::YES,
                    error: null_mut::<c_void>()
                ];
            }
            Bool::YES
        }

        // NSMenuDelegate: populate the Open Recent submenu just before it opens.
        #[unsafe(method(menuWillOpen:))]
        fn _menu_will_open(&self, menu: *mut Object) {
            self.menu_will_open(menu);
        }

        // Action for each item in the Open Recent submenu.
        #[unsafe(method(openRecentDocument:))]
        fn _open_recent_document(&self, sender: *mut Object) {
            // SAFETY: sender is the NSMenuItem; representedObject is the NSURL set in menu_will_open.
            unsafe {
                let url: *mut Object = msg_send![sender, representedObject];
                if url.is_null() {
                    return;
                }
                let dc: *mut Object =
                    msg_send![class!(NSDocumentController), sharedDocumentController];
                let _: *mut Object = msg_send![dc,
                    openDocumentWithContentsOfURL: url,
                    display: Bool::YES,
                    error: null_mut::<c_void>()
                ];
            }
        }
    }
);

impl AppDelegate {
    fn did_finish_launching(&self, notification: *mut Object) {
        // SAFETY: notification is a valid NSNotification; object returns the NSApplication.
        unsafe {
            let app: *mut Object = msg_send![notification, object];
            let _: Bool =
                msg_send![app, setActivationPolicy: NS_APPLICATION_ACTIVATION_POLICY_REGULAR];
            let _: () = msg_send![app, activateIgnoringOtherApps: Bool::YES];
            // Initialize NSDocumentController so it intercepts Apple Events for file opens.
            let _: *mut Object =
                msg_send![class!(NSDocumentController), sharedDocumentController];
        }
    }

    fn menu_will_open(&self, menu: *mut Object) {
        // SAFETY: menu is a valid NSMenu; NSDocumentController.recentDocumentURLs is safe to call.
        unsafe {
            if menu != OPEN_RECENT_MENU.load(Ordering::Relaxed) {
                return;
            }
            // Remove all dynamic items (everything before the trailing separator + "Clear Menu").
            let total: usize = msg_send![menu, numberOfItems];
            let to_remove = total.saturating_sub(2);
            for _ in 0..to_remove {
                let _: () = msg_send![menu, removeItemAtIndex: 0u64];
            }
            let dc: *mut Object =
                msg_send![class!(NSDocumentController), sharedDocumentController];
            let urls: *mut Object = msg_send![dc, recentDocumentURLs];
            let count: usize = msg_send![urls, count];
            if count == 0 {
                return;
            }
            for i in 0..count {
                let url: *mut Object = msg_send![urls, objectAtIndex: i as u64];
                let path_ns: NSString = msg_send![url, path];
                let path_str = path_ns.to_string_lossy();
                let name = path_str.rsplit('/').next().unwrap_or(&path_str).to_owned();
                let item: *mut Object = msg_send![class!(NSMenuItem), alloc];
                let item: *mut Object = msg_send![item,
                    initWithTitle: NSString::from_str(name),
                    action: sel!(openRecentDocument:),
                    keyEquivalent: ns_string!("")
                ];
                let _: () = msg_send![item, setRepresentedObject: url];
                let _: () = msg_send![menu, insertItem: item, atIndex: i as u64];
                let _: () = msg_send![item, release];
            }
        }
    }
}

// MARK: Menu

fn build_menu(app: *mut Object, app_delegate: *mut Object) {
    // SAFETY: All NSMenu/NSMenuItem calls use standard AppKit APIs.
    unsafe {
        let app_name: NSString = msg_send![app, valueForKey: ns_string!("name")];
        let app_name_str = app_name.to_string_lossy();

        let menubar: *mut Object = msg_send![class!(NSMenu), new];
        let _: () = msg_send![app, setMainMenu: menubar];

        // App menu
        let app_menu_item: *mut Object = msg_send![class!(NSMenuItem), new];
        let _: () = msg_send![menubar, addItem: app_menu_item];
        let app_menu: *mut Object = msg_send![class!(NSMenu), new];
        let _: () = msg_send![app_menu_item, setSubmenu: app_menu];

        let _: *mut Object = msg_send![app_menu,
            addItemWithTitle: ns_string!("About BImg"),
            action: sel!(orderFrontStandardAboutPanel:),
            keyEquivalent: ns_string!("")
        ];

        let sep: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![app_menu, addItem: sep];

        let services_item: *mut Object = msg_send![class!(NSMenuItem), new];
        let _: () = msg_send![services_item, setTitle: ns_string!("Services")];
        let _: () = msg_send![app_menu, addItem: services_item];
        let services_menu: *mut Object = msg_send![class!(NSMenu), new];
        let _: () = msg_send![services_item, setSubmenu: services_menu];
        let _: () = msg_send![app, setServicesMenu: services_menu];

        let sep2: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![app_menu, addItem: sep2];

        let hide_title = NSString::from_str(format!("Hide {app_name_str}"));
        let _: *mut Object = msg_send![app_menu,
            addItemWithTitle: hide_title,
            action: sel!(hide:),
            keyEquivalent: ns_string!("h")
        ];
        let hide_others: *mut Object = msg_send![app_menu,
            addItemWithTitle: ns_string!("Hide Others"),
            action: sel!(hideOtherApplications:),
            keyEquivalent: ns_string!("h")
        ];
        let _: () = msg_send![hide_others,
            setKeyEquivalentModifierMask:
                NS_EVENT_MODIFIER_FLAG_OPTION | NS_EVENT_MODIFIER_FLAG_COMMAND
        ];
        let _: *mut Object = msg_send![app_menu,
            addItemWithTitle: ns_string!("Show All"),
            action: sel!(unhideAllApplications:),
            keyEquivalent: ns_string!("")
        ];

        let sep3: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![app_menu, addItem: sep3];

        let quit_title = NSString::from_str(format!("Quit {app_name_str}"));
        let _: *mut Object = msg_send![app_menu,
            addItemWithTitle: quit_title,
            action: sel!(terminate:),
            keyEquivalent: ns_string!("q")
        ];

        // File menu
        let file_menu_item: *mut Object = msg_send![class!(NSMenuItem), new];
        let _: () = msg_send![file_menu_item, setTitle: ns_string!("File")];
        let _: () = msg_send![menubar, addItem: file_menu_item];
        let file_menu: *mut Object = msg_send![class!(NSMenu), new];
        let _: () = msg_send![file_menu_item, setSubmenu: file_menu];

        // openDocument: travels the responder chain and is handled by NSDocumentController.
        let _: *mut Object = msg_send![file_menu,
            addItemWithTitle: ns_string!("Open..."),
            action: sel!(openDocument:),
            keyEquivalent: ns_string!("o")
        ];

        // Open Recent submenu: AppDelegate (as NSMenuDelegate) populates it in menuWillOpen:.
        let open_recent_item: *mut Object = msg_send![class!(NSMenuItem), new];
        let _: () = msg_send![open_recent_item, setTitle: ns_string!("Open Recent")];
        let _: () = msg_send![file_menu, addItem: open_recent_item];
        let open_recent_menu: *mut Object = msg_send![class!(NSMenu), new];
        let _: () = msg_send![open_recent_menu, setTitle: ns_string!("Open Recent")];
        let _: () = msg_send![open_recent_menu, setDelegate: app_delegate];
        // Trailing items always present; dynamic items are inserted above them.
        let sep_or: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![open_recent_menu, addItem: sep_or];
        let _: *mut Object = msg_send![open_recent_menu,
            addItemWithTitle: ns_string!("Clear Menu"),
            action: sel!(clearRecentDocuments:),
            keyEquivalent: ns_string!("")
        ];
        let _: () = msg_send![open_recent_item, setSubmenu: open_recent_menu];
        OPEN_RECENT_MENU.store(open_recent_menu, Ordering::Relaxed);

        let sep_f: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![file_menu, addItem: sep_f];

        let _: *mut Object = msg_send![file_menu,
            addItemWithTitle: ns_string!("Close Window"),
            action: sel!(performClose:),
            keyEquivalent: ns_string!("w")
        ];

        // View menu
        let view_menu_item: *mut Object = msg_send![class!(NSMenuItem), new];
        let _: () = msg_send![view_menu_item, setTitle: ns_string!("View")];
        let _: () = msg_send![menubar, addItem: view_menu_item];
        let view_menu: *mut Object = msg_send![class!(NSMenu), new];
        let _: () = msg_send![view_menu_item, setSubmenu: view_menu];

        let _: *mut Object = msg_send![view_menu,
            addItemWithTitle: ns_string!("Zoom In"),
            action: sel!(zoomIn:),
            keyEquivalent: ns_string!("=")
        ];
        let _: *mut Object = msg_send![view_menu,
            addItemWithTitle: ns_string!("Zoom Out"),
            action: sel!(zoomOut:),
            keyEquivalent: ns_string!("-")
        ];
        let _: *mut Object = msg_send![view_menu,
            addItemWithTitle: ns_string!("Actual Size"),
            action: sel!(actualSize:),
            keyEquivalent: ns_string!("0")
        ];
        let zoom_fit_item: *mut Object = msg_send![view_menu,
            addItemWithTitle: ns_string!("Zoom to Fit"),
            action: sel!(zoomToFit:),
            keyEquivalent: ns_string!("0")
        ];
        let _: () = msg_send![zoom_fit_item,
            setKeyEquivalentModifierMask:
                NS_EVENT_MODIFIER_FLAG_OPTION | NS_EVENT_MODIFIER_FLAG_COMMAND
        ];

        // Window menu
        let window_menu_item: *mut Object = msg_send![class!(NSMenuItem), new];
        let _: () = msg_send![window_menu_item, setTitle: ns_string!("Window")];
        let _: () = msg_send![menubar, addItem: window_menu_item];
        let window_menu: *mut Object = msg_send![class!(NSMenu), new];
        let _: () = msg_send![window_menu_item, setSubmenu: window_menu];
        let _: () = msg_send![app, setWindowsMenu: window_menu];

        let _: *mut Object = msg_send![window_menu,
            addItemWithTitle: ns_string!("Minimize"),
            action: sel!(performMiniaturize:),
            keyEquivalent: ns_string!("m")
        ];
        let _: *mut Object = msg_send![window_menu,
            addItemWithTitle: ns_string!("Zoom"),
            action: sel!(performZoom:),
            keyEquivalent: ns_string!("")
        ];

        let sep_w: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![window_menu, addItem: sep_w];

        let _: *mut Object = msg_send![window_menu,
            addItemWithTitle: ns_string!("Bring All to Front"),
            action: sel!(arrangeInFront:),
            keyEquivalent: ns_string!("")
        ];

        // Help menu
        let help_menu_item: *mut Object = msg_send![class!(NSMenuItem), new];
        let _: () = msg_send![help_menu_item, setTitle: ns_string!("Help")];
        let _: () = msg_send![menubar, addItem: help_menu_item];
        let help_menu: *mut Object = msg_send![class!(NSMenu), new];
        let _: () = msg_send![help_menu_item, setSubmenu: help_menu];
        let _: () = msg_send![app, setHelpMenu: help_menu];
    }
}

// MARK: Main

fn main() {
    autoreleasepool(|_| {
        // Pre-register custom ObjC classes before the event loop starts so that
        // NSDocumentController can look up QoiDocument by name when opening files.
        let _ = CenteringClipView::class();
        let _ = QoiWindowController::class();
        let _ = QoiDocument::class();

        // SAFETY: NSApplication.sharedApplication initialises the app singleton; run drives the
        // main event loop and never returns.
        unsafe {
            let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let delegate: *mut Object = msg_send![AppDelegate::class(), new];
            let _: () = msg_send![app, setDelegate: delegate];
            build_menu(app, delegate);
            let _: () = msg_send![app, run];
        }
    });
}
