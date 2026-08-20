/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A native macOS document-based image viewer.

#![allow(unsafe_code)]

mod browse;
mod checkerboard;
mod cocoa;
mod scroll_view;
mod svg;
mod window_controller;

use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::null_mut;

use block2::RcBlock;
use browse::{file_url, neighbour_path, url_path};
use checkerboard::create_checkerboard_view;
use cocoa::*;
use macview_appkit::{
    Image, NS_VIEW_HEIGHT_SIZABLE, NS_VIEW_WIDTH_SIZABLE, OwnedString, Point, Rect, Size,
    create_image_view, create_tinyvg_view, decode_image, decode_image_data, decode_tinyvg,
    dispatch_async, dispatch_async_main, make_error, ns_string, preferred_content_size,
};
use objc2::ffi::{class_addMethod, objc_msgSendSuper, objc_super, object_getClass};
use objc2::rc::autoreleasepool;
use objc2::runtime::{AnyClass, AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};
use scroll_view::create_scroll_view;
use svg::{Svg, create_svg_view, is_svg, parse_svg};
use window_controller::{create_window_controller, show_media};

struct DocumentIvars {
    image: Cell<*mut Image>,
    tinyvg: Cell<*mut std::sync::Arc<tinyvg::Document>>,
    svg: Cell<*mut Svg>,
    svg_view: Cell<*mut Object>,
    browse_generation: Cell<u64>,
}

enum DecodedMedia {
    Image(Image),
    TinyVg(std::sync::Arc<tinyvg::Document>),
    Svg(Box<Svg>),
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

        #[unsafe(method(nextImage:))]
        fn _next_image(&self, _: *mut Object) {
            self.show_sibling(1);
        }

        #[unsafe(method(previousImage:))]
        fn _previous_image(&self, _: *mut Object) {
            self.show_sibling(-1);
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
        // SAFETY: AppKit supplied a valid NSData for the duration of this call.
        match unsafe { decode_document(data) } {
            Ok(media) => {
                self.install_media(media);
                Bool::YES
            }
            Err(error) => {
                set_error(error_out, &error);
                Bool::NO
            }
        }
    }

    fn install_media(&self, media: DecodedMedia) {
        match media {
            DecodedMedia::Image(image) => {
                self.set_media(Box::into_raw(Box::new(image)), null_mut(), null_mut());
            }
            DecodedMedia::TinyVg(document) => {
                self.set_media(null_mut(), Box::into_raw(Box::new(document)), null_mut());
            }
            DecodedMedia::Svg(document) => {
                self.set_media(null_mut(), null_mut(), Box::into_raw(document));
            }
        }
    }

    fn next_browse_generation(&self) -> u64 {
        let generation = self.ivars().browse_generation.get().wrapping_add(1);
        self.ivars().browse_generation.set(generation);
        generation
    }

    fn is_current_browse(&self, generation: u64) -> bool {
        self.ivars().browse_generation.get() == generation
    }

    fn show_sibling(&self, offset: isize) {
        // SAFETY: AppKit calls actions on the main thread. The retained document remains live
        // until the final main-queue continuation releases it.
        let (path, generation, retained, document_class) = unsafe {
            let this = self as *const Self as *mut Object;
            let url: *mut Object = msg_send![this, fileURL];
            let Some(path) = (!url.is_null()).then(|| url_path(url)).flatten() else {
                return;
            };
            let generation = self.next_browse_generation();
            let retained: *mut Object = msg_send![this, retain];
            (path, generation, retained as usize, Self::class() as usize)
        };

        dispatch_async(move || {
            // SAFETY: The document class is registered for the process lifetime.
            let neighbour = unsafe { neighbour_path(&path, offset, document_class as *mut Object) };
            dispatch_async_main(move || {
                // SAFETY: retained owns a live document until this continuation releases it.
                unsafe {
                    let this = retained as *mut Object;
                    let document = &*this.cast::<Document>();
                    if !document.is_current_browse(generation) {
                        let _: () = msg_send![this, release];
                        return;
                    }
                    let Some(path) = neighbour else {
                        let _: () = msg_send![this, release];
                        return;
                    };
                    document.load_sibling(path, generation);
                }
            });
        });
    }

    /// Starts loading a neighbouring file after its folder scan has completed.
    unsafe fn load_sibling(&self, path: std::path::PathBuf, generation: u64) {
        let error_domain = ns_string!("nl.bplaat.MacView") as usize;
        // SAFETY: This runs on the main queue with a retained document from show_sibling.
        let prepared = unsafe {
            let this = self as *const Self as *mut Object;
            let url = file_url(&path);
            if url.is_null() {
                let _: () = msg_send![this, release];
                None
            } else {
                let controller: *mut Object =
                    msg_send![class!(NSDocumentController), sharedDocumentController];
                let open: *mut Object = msg_send![controller, documentForURL: url];
                if !open.is_null() {
                    let _: () = msg_send![open, showWindows];
                    let _: () = msg_send![this, release];
                    None
                } else {
                    let url: *mut Object = msg_send![url, retain];
                    Some((url as usize, this as usize))
                }
            }
        };
        let Some((url_address, this_address)) = prepared else {
            return;
        };

        dispatch_async(move || {
            // SAFETY: The URL is retained until the main-queue continuation releases it.
            let result = unsafe { load_document(url_address as *mut Object) };
            dispatch_async_main(move || {
                // SAFETY: All Objective-C objects were retained for this continuation.
                unsafe {
                    let this = this_address as *mut Object;
                    let document = &*this.cast::<Document>();
                    let url = url_address as *mut Object;
                    if document.is_current_browse(generation) {
                        match result {
                            Ok((media, kind)) => {
                                document.install_media(media);
                                let _: () = msg_send![this, setFileURL: url];
                                let _: () = msg_send![this, setFileType: kind.as_ptr()];
                                let controller: *mut Object = msg_send![
                                    class!(NSDocumentController),
                                    sharedDocumentController
                                ];
                                let _: () = msg_send![controller, noteNewRecentDocumentURL: url];
                                document.refresh_windows();
                            }
                            Err(description) => {
                                let error = make_error(error_domain as *mut Object, &description);
                                let _: () = msg_send![this, presentError: error];
                            }
                        }
                    }
                    let _: () = msg_send![url, release];
                    let _: () = msg_send![this, release];
                }
            });
        });
    }

    /// Replaces the loaded media, releasing whatever the document held before.
    ///
    /// The web view of the previous media goes with it, because it draws a page that belongs to
    /// media the document no longer holds.
    fn set_media(
        &self,
        image: *mut Image,
        tinyvg: *mut std::sync::Arc<tinyvg::Document>,
        svg: *mut Svg,
    ) {
        let old_image = self.ivars().image.replace(image);
        let old_tinyvg = self.ivars().tinyvg.replace(tinyvg);
        let old_svg = self.ivars().svg.replace(svg);
        let old_svg_view = self.ivars().svg_view.replace(null_mut());
        // SAFETY: The ivars own the retained objects and the Boxes converted into these pointers.
        unsafe {
            if !old_image.is_null() {
                drop(Box::from_raw(old_image));
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
        (!image.is_null()).then(|| unsafe { &*image }.size())
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
                return create_tinyvg_view(frame, (*tinyvg).clone());
            }
            if !svg.is_null() {
                let view = create_svg_view(frame, &*svg);
                let old_view = self.ivars().svg_view.replace(msg_send![view, retain]);
                if !old_view.is_null() {
                    let _: () = msg_send![old_view, release];
                }
                return view;
            }
            create_image_view(frame, (&*self.ivars().image.get()).as_ptr())
        }
    }

    /// Returns the size the window title shows, which is the stored size of a bitmap.
    fn title_size(&self, media_size: Size) -> Size {
        let image = self.ivars().image.get();
        if image.is_null() {
            media_size
        } else {
            // SAFETY: The ivar owns a live NSImage that keeps its representations alive.
            unsafe { image_pixel_size((&*image).as_ptr(), media_size) }
        }
    }

    /// Shows the media this document holds now in the windows it opened before.
    fn refresh_windows(&self) {
        let Some(media_size) = self.media_size() else {
            return;
        };
        let title_size = self.title_size(media_size);
        // SAFETY: The document owns its window controllers, and each media view is released after
        // the scroll view of a window retains it.
        unsafe {
            let this = self as *const Self as *mut Object;
            let controllers: *mut Object = msg_send![this, windowControllers];
            let count: usize = msg_send![controllers, count];
            for index in 0..count {
                let controller: *mut Object = msg_send![controllers, objectAtIndex: index];
                let view = self.create_media_view(Rect {
                    origin: Point { x: 0.0, y: 0.0 },
                    size: media_size,
                });
                show_media(controller, view, title_size);
                let _: () = msg_send![view, release];
            }
        }
    }

    fn make_window_controllers(&self) {
        let Some(media_size) = self.media_size() else {
            return;
        };
        let title_size = self.title_size(media_size);

        // SAFETY: All objects are valid AppKit instances, selectors use their documented ABI,
        // and ownership is balanced after the document retains its window controller.
        unsafe {
            let content_size = preferred_content_size(media_size);
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
            // A window that is created in code takes no part in full screen until it says so,
            // which leaves the green button of the title bar zooming only.
            let behavior: u64 = msg_send![window, collectionBehavior];
            let _: () = msg_send![window,
                setCollectionBehavior: behavior | NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_PRIMARY
            ];
            let _: () = msg_send![window, center];

            // The media keeps its own size and the scroll view magnifies it, so that zooming
            // redraws the vector formats instead of scaling a picture of them.
            let checkerboard = create_checkerboard_view(rect);
            let media_view = self.create_media_view(Rect {
                origin: Point { x: 0.0, y: 0.0 },
                size: media_size,
            });
            let scroll_view = create_scroll_view(rect, media_view);
            let _: () = msg_send![media_view, release];
            // A window opens on the zoom the Zoom to Fit item sets, so that the media is shown
            // the same way however it got there.
            let _: () = msg_send![scroll_view, zoomToFit];
            let _: () = msg_send![checkerboard, addSubview: scroll_view];
            let _: () = msg_send![scroll_view, release];
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

/// Decodes document data without creating any views or touching window state.
///
/// # Safety
///
/// `data` must point to a valid `NSData` for the duration of this call.
unsafe fn decode_document(data: *mut Object) -> Result<DecodedMedia, String> {
    // SAFETY: NSData keeps its immutable byte buffer alive for this call.
    let bytes = unsafe {
        let length: usize = msg_send![data, length];
        let bytes: *const c_void = msg_send![data, bytes];
        std::slice::from_raw_parts(bytes.cast::<u8>(), length)
    };
    if tinyvg::is_tinyvg(bytes) {
        return decode_tinyvg(bytes)
            .map(std::sync::Arc::new)
            .map(DecodedMedia::TinyVg);
    }
    if is_svg(bytes) {
        return Ok(DecodedMedia::Svg(Box::new(parse_svg(bytes))));
    }

    // SAFETY: data remains live and is retained when AppKit needs it after this call.
    unsafe { decode_image_data(data) }.map(DecodedMedia::Image)
}

/// Decodes bytes read by Rust without creating views or touching window state.
fn decode_document_bytes(bytes: Vec<u8>) -> Result<DecodedMedia, String> {
    if tinyvg::is_tinyvg(&bytes) {
        return decode_tinyvg(&bytes)
            .map(std::sync::Arc::new)
            .map(DecodedMedia::TinyVg);
    }
    if is_svg(&bytes) {
        return Ok(DecodedMedia::Svg(Box::new(parse_svg(&bytes))));
    }
    decode_image(bytes).map(DecodedMedia::Image)
}

/// Reads and decodes a document URL on a background queue.
///
/// # Safety
///
/// `url` must point to a retained `NSURL` for the duration of this call.
unsafe fn load_document(url: *mut Object) -> Result<(DecodedMedia, OwnedString), String> {
    let result = std::sync::Arc::new(std::sync::Mutex::new(None));
    let accessor_result = result.clone();
    let accessor = RcBlock::new::<*mut Object>(move |coordinated_url| {
        // SAFETY: NSFileCoordinator supplies a live coordinated file URL for this synchronous
        // accessor invocation. Catching a panic here prevents unwinding across the Objective-C
        // block ABI.
        let decoded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            decode_document_url(coordinated_url)
        }))
        .unwrap_or_else(|_| Err(String::from("Could not decode the image")));
        *accessor_result
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(decoded);
    });

    // SAFETY: url is retained by the caller. The coordinator invokes the copied accessor before
    // returning, and it coordinates the read against file presenters and writers.
    unsafe {
        let scoped: Bool = msg_send![url, startAccessingSecurityScopedResource];
        let coordinator: *mut Object = msg_send![class!(NSFileCoordinator), alloc];
        let coordinator: *mut Object = msg_send![coordinator,
            initWithFilePresenter: null_mut::<Object>()
        ];
        let _: () = msg_send![coordinator,
            coordinateReadingItemAtURL: url,
            options: 0usize,
            error: null_mut::<c_void>(),
            byAccessor: &*accessor
        ];
        let _: () = msg_send![coordinator, release];
        if scoped.as_bool() {
            let _: () = msg_send![url, stopAccessingSecurityScopedResource];
        }
    }

    result
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .take()
        .unwrap_or_else(|| Err(String::from("Could not coordinate reading the image")))
}

/// Reads and decodes a URL supplied by `NSFileCoordinator`.
///
/// # Safety
///
/// `url` must be a valid coordinated file URL for this call.
unsafe fn decode_document_url(url: *mut Object) -> Result<(DecodedMedia, OwnedString), String> {
    // SAFETY: The caller supplies a live coordinated file URL.
    let path = unsafe { url_path(url) }.ok_or_else(|| String::from("Could not read the image"))?;
    let bytes = std::fs::read(&path).map_err(|_| String::from("Could not read the image"))?;
    let extension = path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| String::from("Unsupported image type"))?;
    // SAFETY: UTType and NSString are immutable and safe on this worker.
    let identifier = unsafe { type_identifier(extension) }?;
    decode_document_bytes(bytes).map(|media| (media, identifier))
}

/// Returns the type identifier MacView declares for an extension.
///
/// # Safety
///
/// This calls thread-safe immutable Uniform Type Identifier APIs.
unsafe fn type_identifier(extension: &str) -> Result<OwnedString, String> {
    let declared = if extension.eq_ignore_ascii_case("qoi") {
        Some("org.qoiformat.qoi")
    } else if extension.eq_ignore_ascii_case("tvg") {
        Some("org.tinyvg.tvg")
    } else if extension.eq_ignore_ascii_case("tvgt") {
        Some("org.tinyvg.tvgt")
    } else {
        None
    };
    if let Some(identifier) = declared {
        return Ok(OwnedString::new(identifier));
    }

    // SAFETY: UTType returns immutable autoreleased objects, and OwnedString retains the result.
    unsafe {
        let kind: *mut Object = msg_send![class!(UTType),
            typeWithFilenameExtension: ns_string(extension)
        ];
        if kind.is_null() {
            return Err(String::from("Unsupported image type"));
        }
        let identifier: *mut Object = msg_send![kind, identifier];
        Ok(OwnedString::retain(identifier))
    }
}

const extern "C" fn can_concurrently_read_documents(
    _: *mut Object,
    _: objc2::runtime::Sel,
    _: *mut Object,
) -> Bool {
    Bool::YES
}

/// Tells `NSDocumentController` that document decoding is safe on its background queue.
unsafe fn enable_concurrent_document_reading() {
    // SAFETY: Document is registered, object_getClass returns its metaclass, and the function
    // signature matches +canConcurrentlyReadDocumentsOfType: (BOOL, Class, SEL, NSString *).
    unsafe {
        let metaclass = object_getClass(Document::class());
        let added = class_addMethod(
            metaclass,
            sel!(canConcurrentlyReadDocumentsOfType:).0,
            can_concurrently_read_documents as *const c_void,
            c"c@:@".as_ptr(),
        );
        assert!(added, "failed to enable concurrent document reading");
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
    let domain = ns_string!("nl.bplaat.MacView");
    // SAFETY: error_out is a non-null NSError** supplied by NSDocument. The error is
    // autoreleased and remains valid for the current event cycle.
    unsafe {
        error_out
            .cast::<*mut Object>()
            .write(make_error(domain, description));
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

/// Returns the key equivalent of a function key, which AppKit spells as a single character.
fn function_key(code: u16) -> String {
    char::from_u32(u32::from(code))
        .expect("function keys are characters of the private use area")
        .to_string()
}

/// Adds the submenu that `NSDocumentController` fills with the files that were opened before.
///
/// A menu says what it is by its name, which is how the document controller finds this one and
/// keeps it up to date. The name is set through a method AppKit does not document, so the menu
/// stays an ordinary one when a future release drops it.
fn add_open_recent_menu(menu: *mut Object) {
    // SAFETY: menu is a valid NSMenu that retains the item, which retains the submenu.
    unsafe {
        let item: *mut Object = msg_send![class!(NSMenuItem), new];
        let _: () = msg_send![item, setTitle: ns_string!("Open Recent")];
        let recent_menu: *mut Object = msg_send![class!(NSMenu), alloc];
        let recent_menu: *mut Object =
            msg_send![recent_menu, initWithTitle: ns_string!("Open Recent")];
        let named: Bool = msg_send![recent_menu, respondsToSelector: sel!(_setMenuName:)];
        if named.as_bool() {
            let _: () = msg_send![recent_menu, _setMenuName: ns_string!("NSRecentDocumentsMenu")];
        }
        add_item(
            recent_menu,
            "Clear Menu",
            sel!(clearRecentDocuments:),
            "",
            0,
            null_mut(),
        );
        let _: () = msg_send![item, setSubmenu: recent_menu];
        let _: () = msg_send![recent_menu, release];
        let _: () = msg_send![menu, addItem: item];
        let _: () = msg_send![item, release];
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
        let _: () = msg_send![services_item, setTitle: ns_string!("Services")];
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
        add_open_recent_menu(file_menu);
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

        let view_menu = add_menu(main_menu, "View");
        add_item(
            view_menu,
            "Zoom In",
            sel!(zoomIn:),
            "+",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        add_item(
            view_menu,
            "Zoom Out",
            sel!(zoomOut:),
            "-",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        add_separator(view_menu);
        add_item(
            view_menu,
            "Actual Size",
            sel!(actualSize:),
            "0",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        add_item(
            view_menu,
            "Zoom to Fit",
            sel!(zoomToFit:),
            "9",
            NS_EVENT_MODIFIER_FLAG_COMMAND,
            null_mut(),
        );
        add_separator(view_menu);
        add_item(
            view_menu,
            "Previous Image",
            sel!(previousImage:),
            &function_key(NS_LEFT_ARROW_FUNCTION_KEY),
            0,
            null_mut(),
        );
        add_item(
            view_menu,
            "Next Image",
            sel!(nextImage:),
            &function_key(NS_RIGHT_ARROW_FUNCTION_KEY),
            0,
            null_mut(),
        );
        add_separator(view_menu);
        // AppKit renames the item to Exit Full Screen while a window is in full screen itself.
        add_item(
            view_menu,
            "Enter Full Screen",
            sel!(toggleFullScreen:),
            "f",
            NS_EVENT_MODIFIER_FLAG_COMMAND | NS_EVENT_MODIFIER_FLAG_CONTROL,
            null_mut(),
        );
        let _: () = msg_send![view_menu, release];

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
        enable_concurrent_document_reading();
        let application: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        // Every document gets a window of its own, so the tab items AppKit adds to the Window
        // menu would control something this application does not have.
        let _: () = msg_send![class!(NSWindow), setAllowsAutomaticWindowTabbing: Bool::NO];
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn coordinated_load_reads_and_classifies_an_image() {
        autoreleasepool(|_| {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/qoi_logo.qoi");
            // SAFETY: path names an existing image and the autorelease pool keeps its URL alive.
            let (media, kind) = unsafe {
                let url = file_url(&path);
                load_document(url).expect("example image should load")
            };
            assert!(matches!(media, DecodedMedia::Image(_)));
            // SAFETY: kind owns a live NSString for this scope.
            let identifier: *const std::ffi::c_char =
                unsafe { msg_send![kind.as_ptr(), UTF8String] };
            // SAFETY: NSString keeps its UTF-8 representation alive while kind is alive.
            let identifier = unsafe { std::ffi::CStr::from_ptr(identifier) };
            assert_eq!(identifier.to_bytes(), b"org.qoiformat.qoi");
        });
    }
}
