/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::Cell;
use std::ptr::null_mut;

use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send};

use super::cocoa::*;
use super::event_loop::{allow_termination_if_last_window, send_event, send_theme_change};
#[cfg(feature = "file_drop")]
use super::file_drop::{perform_file_drop, register_dragged_types};
use crate::{
    CloseRequest, Cursor, LogicalPoint, LogicalSize, MacosTitlebarStyle, Theme, WindowBuilder,
    WindowEvent,
};

define_class!(
    #[unsafe(super(NSView))]
    pub(super) struct DraggableView;

    impl DraggableView {
        #[unsafe(method(mouseDown:))]
        fn _mouse_down(&self, event: *mut Object) {
            let this = self as *const DraggableView as *mut Object;
            let window: *mut Object = unsafe { msg_send![this, window] };
            if !window.is_null() {
                let _: () = unsafe { msg_send![window, performWindowDragWithEvent:event] };
            }
        }
    }
);

struct ThemeAwareViewIvars {
    theme: Cell<i64>,
    cursor: Cell<i64>,
}

define_class!(
    #[unsafe(super(NSView))]
    #[ivars = ThemeAwareViewIvars]
    struct ThemeAwareView;

    impl ThemeAwareView {
        #[unsafe(method(viewDidChangeEffectiveAppearance))]
        fn _view_did_change_effective_appearance(&self) {
            let view = self as *const ThemeAwareView as *mut Object;
            let old_theme = self.ivars().theme.get();
            // A zero tag means the window has an explicit appearance or has
            // not completed initialization yet.
            if old_theme == 0 {
                return;
            }
            let theme = effective_theme(view);
            let new_theme = theme_tag(theme);
            if new_theme != old_theme {
                self.ivars().theme.set(new_theme);
                send_theme_change(theme);
                let _: () = unsafe { msg_send![view, setNeedsDisplay:Bool::YES] };
            }
        }

        #[unsafe(method(resetCursorRects))]
        fn _reset_cursor_rects(&self) {
            let view = self as *const ThemeAwareView as *mut Object;
            let bounds: NSRect = unsafe { msg_send![view, bounds] };
            let cursor = ns_cursor(cursor_from_tag(self.ivars().cursor.get()));
            let _: () = unsafe { msg_send![view, addCursorRect:bounds, cursor:cursor] };
        }
    }
);

fn effective_theme(object: *mut Object) -> Theme {
    unsafe {
        let appearance: *mut Object = msg_send![object, effectiveAppearance];
        let name: NSString = msg_send![appearance, name];
        if name.to_string().contains("Dark") {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

const fn theme_tag(theme: Theme) -> i64 {
    match theme {
        Theme::Light => 1,
        Theme::Dark => 2,
    }
}

const fn cursor_tag(cursor: Cursor) -> i64 {
    match cursor {
        Cursor::Default => 0,
        Cursor::Pointer => 1,
        Cursor::Crosshair => 2,
        Cursor::Text => 3,
        Cursor::Grab => 4,
        Cursor::Grabbing => 5,
    }
}

const fn cursor_from_tag(tag: i64) -> Cursor {
    match tag {
        1 => Cursor::Pointer,
        2 => Cursor::Crosshair,
        3 => Cursor::Text,
        4 => Cursor::Grab,
        5 => Cursor::Grabbing,
        _ => Cursor::Default,
    }
}

fn ns_cursor(cursor: Cursor) -> *mut Object {
    unsafe {
        match cursor {
            Cursor::Default => msg_send![class!(NSCursor), arrowCursor],
            Cursor::Pointer => msg_send![class!(NSCursor), pointingHandCursor],
            Cursor::Crosshair => msg_send![class!(NSCursor), crosshairCursor],
            Cursor::Text => msg_send![class!(NSCursor), IBeamCursor],
            Cursor::Grab => msg_send![class!(NSCursor), openHandCursor],
            Cursor::Grabbing => msg_send![class!(NSCursor), closedHandCursor],
        }
    }
}

// MARK: WindowDelegate
define_class!(
    #[unsafe(super(NSObject))]
    struct WindowDelegate;

    impl WindowDelegate {
        #[unsafe(method(windowShouldClose:))]
        fn _window_should_close(&self, window: *mut Object) -> Bool { self.window_should_close(window) }

        #[unsafe(method(windowDidMove:))]
        fn _window_did_move(&self, notification: *mut Object) { self.window_did_move(notification); }

        #[unsafe(method(windowDidResize:))]
        fn _window_did_resize(&self, notification: *mut Object) { self.window_did_resize(notification); }

        #[unsafe(method(windowWillEnterFullScreen:))]
        fn _window_will_enter_fullscreen(&self, notification: *mut Object) { self.window_will_enter_fullscreen(notification); }

        #[unsafe(method(windowWillExitFullScreen:))]
        fn _window_will_exit_fullscreen(&self, _: *mut Object) { self.window_will_exit_fullscreen(); }

        #[unsafe(method(windowDidExitFullScreen:))]
        fn _window_did_exit_fullscreen(&self, notification: *mut Object) { self.window_did_exit_fullscreen(notification); }

        #[unsafe(method(windowDidFailToEnterFullScreen:))]
        fn _window_did_fail_to_enter_fullscreen(&self, window: *mut Object) { self.window_did_fail_to_enter_fullscreen(window); }

        #[unsafe(method(windowDidFailToExitFullScreen:))]
        fn _window_did_fail_to_exit_fullscreen(&self, window: *mut Object) { self.window_did_fail_to_exit_fullscreen(window); }

        #[cfg(feature = "file_drop")]
        #[unsafe(method(draggingEntered:))]
        const fn _dragging_entered(&self, _: *mut Object) -> u64 { NS_DRAG_OPERATION_COPY }

        #[cfg(feature = "file_drop")]
        #[unsafe(method(draggingUpdated:))]
        const fn _dragging_updated(&self, _: *mut Object) -> u64 { NS_DRAG_OPERATION_COPY }

        #[cfg(feature = "file_drop")]
        #[unsafe(method(prepareForDragOperation:))]
        const fn _prepare_for_drag_operation(&self, _: *mut Object) -> Bool { Bool::YES }

        #[cfg(feature = "file_drop")]
        #[unsafe(method(performDragOperation:))]
        fn _perform_drag_operation(&self, sender: *mut Object) -> Bool { perform_file_drop(sender) }
    }
);

impl WindowDelegate {
    fn window_should_close(&self, window: *mut Object) -> Bool {
        let request = CloseRequest::new();
        send_event(crate::Event::Window(WindowEvent::CloseRequested(
            request.clone(),
        )));
        if request.is_prevented() {
            Bool::NO
        } else {
            allow_termination_if_last_window(window);
            Bool::YES
        }
    }

    fn window_did_move(&self, notification: *mut Object) {
        let window: *mut Object = unsafe { msg_send![notification, object] };
        let frame: NSRect = unsafe { msg_send![window, frame] };
        send_event(crate::Event::Window(WindowEvent::Move(LogicalPoint::new(
            frame.origin.x as f32,
            frame.origin.y as f32,
        ))));
    }

    fn window_did_resize(&self, notification: *mut Object) {
        let window: *mut Object = unsafe { msg_send![notification, object] };
        let content_view: *mut Object = unsafe { msg_send![window, contentView] };
        let frame: NSRect = unsafe { msg_send![content_view, frame] };
        send_event(crate::Event::Window(WindowEvent::Resize(LogicalSize::new(
            frame.size.width as f32,
            frame.size.height as f32,
        ))));
    }

    fn window_will_enter_fullscreen(&self, notification: *mut Object) {
        let window: *mut Object = unsafe { msg_send![notification, object] };
        set_drag_view_hidden(window, true);
        send_event(crate::Event::Window(WindowEvent::MacosFullscreenChange(
            true,
        )));
    }

    fn window_will_exit_fullscreen(&self) {
        send_event(crate::Event::Window(WindowEvent::MacosFullscreenChange(
            false,
        )));
    }

    fn window_did_exit_fullscreen(&self, notification: *mut Object) {
        let window: *mut Object = unsafe { msg_send![notification, object] };
        set_drag_view_hidden(window, false);
    }

    fn window_did_fail_to_enter_fullscreen(&self, window: *mut Object) {
        set_drag_view_hidden(window, false);
        send_event(crate::Event::Window(WindowEvent::MacosFullscreenChange(
            false,
        )));
    }

    fn window_did_fail_to_exit_fullscreen(&self, window: *mut Object) {
        set_drag_view_hidden(window, true);
        send_event(crate::Event::Window(WindowEvent::MacosFullscreenChange(
            true,
        )));
    }
}

fn set_drag_view_hidden(window: *mut Object, hidden: bool) {
    let has_transparent_titlebar: Bool = unsafe { msg_send![window, titlebarAppearsTransparent] };
    if has_transparent_titlebar == Bool::NO {
        return;
    }
    let content_view: *mut Object = unsafe { msg_send![window, contentView] };
    let subviews: *mut Object = unsafe { msg_send![content_view, subviews] };
    let drag_view: *mut Object = unsafe { msg_send![subviews, lastObject] };
    if drag_view.is_null() {
        return;
    }
    let hidden = if hidden { Bool::YES } else { Bool::NO };
    let _: () = unsafe { msg_send![drag_view, setHidden:hidden] };
}

fn add_drag_view(window: *mut Object, content_view: *mut Object) {
    let drag_view: *mut Object = unsafe { msg_send![DraggableView::class(), new] };
    let bounds: NSRect = unsafe { msg_send![content_view, bounds] };
    let content_layout_rect: NSRect = unsafe { msg_send![window, contentLayoutRect] };
    let content_layout_rect: NSRect = unsafe {
        msg_send![content_view, convertRect:content_layout_rect, fromView:null_mut::<Object>()]
    };
    let titlebar_height =
        bounds.size.height - content_layout_rect.origin.y - content_layout_rect.size.height;
    let _: () = unsafe {
        msg_send![drag_view, setFrame:NSRect::new(
            NSPoint::new(bounds.origin.x, bounds.origin.y + bounds.size.height - titlebar_height),
            NSSize::new(bounds.size.width, titlebar_height),
        )]
    };
    let _: () = unsafe {
        msg_send![drag_view, setAutoresizingMask:NS_VIEW_WIDTH_SIZABLE | NS_VIEW_MIN_Y_MARGIN]
    };
    let _: () = unsafe { msg_send![content_view, addSubview:drag_view] };
}

pub(super) struct PlatformWindowData {
    pub(super) window: *mut Object,
    pub(super) background_color: Option<u32>,
    #[cfg(feature = "file_drop")]
    pub(super) allow_file_drop: bool,
}

pub(crate) struct PlatformWindow(pub(super) Box<PlatformWindowData>);

impl PlatformWindow {
    pub(crate) fn new(builder: &WindowBuilder) -> Self {
        // Register WindowDelegate class and configure NSWindow (idempotent)
        let _: () =
            unsafe { msg_send![class!(NSWindow), setAllowsAutomaticWindowTabbing:Bool::NO] };

        // Allocate window data box first so we have a stable ptr
        let mut window_data = Box::new(PlatformWindowData {
            window: null_mut(),
            background_color: builder.background_color,
            #[cfg(feature = "file_drop")]
            allow_file_drop: builder.allow_file_drop,
        });

        // Create WindowDelegate instance
        let window_delegate: *mut Object = unsafe { msg_send![WindowDelegate::class(), new] };

        // Create window
        let screen_rect: NSRect = if let Some(monitor) = builder.monitor {
            unsafe { msg_send![monitor.screen, frame] }
        } else {
            let screen: *mut Object = unsafe { msg_send![class!(NSScreen), mainScreen] };
            unsafe { msg_send![screen, frame] }
        };
        let window_rect = if builder.should_fullscreen {
            screen_rect
        } else {
            NSRect::new(
                if let Some(position) = builder.position {
                    NSPoint::new(
                        screen_rect.origin.x + position.x as f64,
                        screen_rect.origin.y
                            + (screen_rect.size.height - builder.size.height as f64)
                            - position.y as f64,
                    )
                } else {
                    NSPoint::new(
                        screen_rect.origin.x
                            + (screen_rect.size.width - builder.size.width as f64) / 2.0,
                        screen_rect.origin.y
                            + (screen_rect.size.height - builder.size.height as f64) / 2.0,
                    )
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
        if builder.macos_titlebar_style == MacosTitlebarStyle::Transparent
            || builder.macos_titlebar_style == MacosTitlebarStyle::Hidden
        {
            window_style_mask |= NS_WINDOW_STYLE_MASK_FULL_SIZE_CONTENT_VIEW;
        }
        if builder.should_fullscreen {
            window_style_mask = 0;
        }

        let window = unsafe {
            let window: *mut Object = msg_send![class!(NSWindow), alloc];
            let window: *mut Object = msg_send![window, initWithContentRect:NSRect::new(NSPoint::new(0.0, 0.0), window_rect.size),
                styleMask:window_style_mask, backing:NS_BACKING_STORE_BUFFERED, defer:false];
            let content_view: *mut Object = msg_send![ThemeAwareView::class(), alloc];
            let content_view: *mut Object = msg_send![content_view, initWithFrame:NSRect::new(NSPoint::new(0.0, 0.0), window_rect.size)];
            let _: () = msg_send![content_view, setAutoresizingMask:NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE];
            let _: () = msg_send![window, setContentView:content_view];
            let _: () = msg_send![content_view, release];
            let _: () = msg_send![window, setFrameOrigin:window_rect.origin];
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
                let _: () = msg_send![window, setContentMinSize:NSSize::new(min_size.width as f64, min_size.height as f64)];
            }
            #[cfg(feature = "remember_window_state")]
            if builder.remember_window_state {
                let _: Bool = msg_send![window, setFrameAutosaveName:ns_string!("window")];
            }
            let _: () = msg_send![window, setDelegate:window_delegate];
            #[cfg(feature = "file_drop")]
            if builder.allow_file_drop {
                register_dragged_types(window);
            }
            window
        };

        if builder.theme.is_none() {
            let content_view: *mut Object = unsafe { msg_send![window, contentView] };
            let content_view = unsafe { &*(content_view as *const ThemeAwareView) };
            content_view
                .ivars()
                .theme
                .set(theme_tag(effective_theme(window)));
        }

        window_data.window = window;
        if !builder.should_fullscreen
            && (builder.macos_titlebar_style == MacosTitlebarStyle::Transparent
                || builder.macos_titlebar_style == MacosTitlebarStyle::Hidden)
        {
            let content_view: *mut Object = unsafe { msg_send![window, contentView] };
            add_drag_view(window, content_view);
        }
        PlatformWindow(window_data)
    }
}

impl crate::WindowInterface for PlatformWindow {
    fn close(&mut self) {
        allow_termination_if_last_window(self.0.window);
        let _: () = unsafe { msg_send![self.0.window, close] };
    }

    fn set_title(&mut self, title: impl AsRef<str>) {
        unsafe { msg_send![self.0.window, setTitle:NSString::from_str(title)] }
    }

    fn position(&self) -> LogicalPoint {
        let frame: NSRect = unsafe { msg_send![self.0.window, frame] };
        LogicalPoint::new(frame.origin.x as f32, frame.origin.y as f32)
    }

    fn size(&self) -> LogicalSize {
        let content_view: *mut Object = unsafe { msg_send![self.0.window, contentView] };
        let frame: NSRect = unsafe { msg_send![content_view, frame] };
        LogicalSize::new(frame.size.width as f32, frame.size.height as f32)
    }

    fn set_position(&mut self, point: LogicalPoint) {
        unsafe {
            msg_send![self.0.window, setFrameTopLeftPoint:NSPoint::new(point.x as f64, point.y as f64)]
        }
    }

    fn set_size(&mut self, size: LogicalSize) {
        unsafe {
            msg_send![self.0.window, setContentSize:NSSize::new(size.width as f64, size.height as f64)]
        }
    }

    fn set_min_size(&mut self, min_size: LogicalSize) {
        unsafe {
            msg_send![self.0.window, setContentMinSize:NSSize::new(min_size.width as f64, min_size.height as f64)]
        }
    }

    fn set_resizable(&mut self, resizable: bool) {
        let mut style_mask: u64 = unsafe { msg_send![self.0.window, styleMask] };
        if resizable {
            style_mask |= NS_WINDOW_STYLE_MASK_RESIZABLE;
        } else {
            style_mask &= !NS_WINDOW_STYLE_MASK_RESIZABLE;
        }
        unsafe { msg_send![self.0.window, setStyleMask:style_mask] }
    }

    fn set_theme(&mut self, theme: Theme) {
        unsafe {
            let content_view: *mut Object = msg_send![self.0.window, contentView];
            let content_view = &*(content_view as *const ThemeAwareView);
            content_view.ivars().theme.set(0);
            let appearance: *mut Object = msg_send![class!(NSAppearance), appearanceNamed:match theme {
                Theme::Light => NSAppearanceNameAqua,
                Theme::Dark => NSAppearanceNameDarkAqua,
            }];
            let _: () = msg_send![self.0.window, setAppearance:appearance];
        }
    }

    fn set_background_color(&mut self, color: u32) {
        self.0.background_color = Some(color);
        unsafe {
            let color_obj: *mut Object = msg_send![class!(NSColor), colorWithRed:((color >> 16) & 0xFF) as f64 / 255.0,
                green:((color >> 8) & 0xFF) as f64 / 255.0,
                blue:(color & 0xFF) as f64 / 255.0, alpha:1.0];
            let _: () = msg_send![self.0.window, setBackgroundColor:color_obj];
        }
    }

    fn set_cursor(&mut self, cursor: Cursor) {
        unsafe {
            let content_view: *mut Object = msg_send![self.0.window, contentView];
            let theme_view = &*(content_view as *const ThemeAwareView);
            theme_view.ivars().cursor.set(cursor_tag(cursor));
            let _: () = msg_send![self.0.window, invalidateCursorRectsForView:content_view];
            let _: () = msg_send![ns_cursor(cursor), set];
        }
    }

    fn macos_titlebar_size(&self) -> LogicalSize {
        let window_frame: NSRect = unsafe { msg_send![self.0.window, frame] };
        let content_layout_rect: NSRect = unsafe { msg_send![self.0.window, contentLayoutRect] };
        LogicalSize::new(
            window_frame.size.width as f32,
            (window_frame.size.height - content_layout_rect.size.height) as f32,
        )
    }

    fn macos_set_document_edited(&mut self, edited: bool) {
        let _: () = unsafe { msg_send![self.0.window, setDocumentEdited:edited] };
    }
}
