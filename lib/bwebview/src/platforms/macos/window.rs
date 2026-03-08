/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;
use std::ptr::null_mut;

use objc2::runtime::{AnyClass, AnyObject as Object, Bool, ClassBuilder, Sel};
use objc2::{class, msg_send, sel};

use super::cocoa::*;
use super::event_loop::{IVAR_PTR, IVAR_PTR_CSTR, send_event};
use crate::{
    LogicalPoint, LogicalSize, MacosTitlebarStyle, Theme, WindowBuilder, WindowEvent, WindowId,
};

pub(super) struct PlatformWindowData {
    pub(super) window_id: WindowId,
    pub(super) window: *mut Object,
    pub(super) background_color: Option<u32>,
    pub(super) webview: *mut Object,
}

pub(crate) struct PlatformWindow(pub(super) Box<PlatformWindowData>);

impl PlatformWindow {
    pub(crate) fn new(window_id: WindowId, builder: &WindowBuilder) -> Self {
        // Register WindowDelegate class
        if AnyClass::get(c"WindowDelegate").is_none() {
            let mut decl = ClassBuilder::new(c"WindowDelegate", class!(NSObject))
                .expect("Can't create WindowDelegate class");
            decl.add_ivar::<*const c_void>(IVAR_PTR_CSTR);
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
                    sel!(windowWillEnterFullScreen:),
                    window_will_enter_fullscreen as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(windowWillExitFullScreen:),
                    window_will_exit_fullscreen as extern "C" fn(_, _, _),
                );
            }
            decl.register();
            let _: () =
                unsafe { msg_send![class!(NSWindow), setAllowsAutomaticWindowTabbing:Bool::NO] };
        }

        // Allocate window data box first so we have a stable ptr
        let mut window_data = Box::new(PlatformWindowData {
            window_id,
            window: std::ptr::null_mut(),
            background_color: builder.background_color,
            webview: std::ptr::null_mut(),
        });

        // Create WindowDelegate with _ptr pointing to the box
        let window_delegate: *mut Object = unsafe { msg_send![class!(WindowDelegate), new] };
        unsafe {
            #[allow(deprecated)]
            let ptr_ivar = (*window_delegate).get_mut_ivar::<*const c_void>(IVAR_PTR);
            *ptr_ivar = window_data.as_ref() as *const PlatformWindowData as *const c_void;
        };

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
        if builder.should_fullscreen {
            window_style_mask = 0;
        }

        let window = unsafe {
            let window: *mut Object = msg_send![class!(NSWindow), alloc];
            let window: *mut Object = msg_send![window, initWithContentRect:NSRect::new(NSPoint::new(0.0, 0.0), window_rect.size),
                styleMask:window_style_mask, backing:NS_BACKING_STORE_BUFFERED, defer:false];
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
                let _: () = msg_send![window, setMinSize:NSSize::new(min_size.width as f64, min_size.height as f64)];
            }
            #[cfg(feature = "remember_window_state")]
            if builder.remember_window_state {
                let _: Bool = msg_send![window, setFrameAutosaveName:NSString::from_str("window")];
            }
            let _: () = msg_send![window, setDelegate:window_delegate];
            window
        };

        window_data.window = window;
        PlatformWindow(window_data)
    }
}

impl crate::WindowInterface for PlatformWindow {
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
        let frame: NSRect = unsafe { msg_send![self.0.window, frame] };
        unsafe {
            msg_send![self.0.window, setFrame:NSRect::new(frame.origin, NSSize::new(size.width as f64, size.height as f64)), display:true]
        }
    }

    fn set_min_size(&mut self, min_size: LogicalSize) {
        unsafe {
            msg_send![self.0.window, setMinSize:NSSize::new(min_size.width as f64, min_size.height as f64)]
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
            if !self.0.webview.is_null() {
                let value: *mut Object = msg_send![class!(NSNumber), numberWithBool:false];
                let _: () = msg_send![self.0.webview, setValue:value, forKey:NSString::from_str("drawsBackground")];
            }
        }
    }

    fn macos_titlebar_size(&self) -> LogicalSize {
        let window_frame: NSRect = unsafe { msg_send![self.0.window, frame] };
        let content_rect: NSRect =
            unsafe { msg_send![self.0.window, contentRectForFrameRect:window_frame] };
        LogicalSize::new(
            window_frame.size.width as f32,
            (window_frame.size.height - content_rect.size.height) as f32,
        )
    }
}

pub(crate) fn get_window_id(this: *mut Object) -> WindowId {
    unsafe {
        #[allow(deprecated)]
        let ptr = *(*this).get_ivar::<*const c_void>(IVAR_PTR);
        (*(ptr as *const PlatformWindowData)).window_id
    }
}

extern "C" fn window_did_move(this: *mut Object, _sel: Sel, notification: *mut Object) {
    let window_id = get_window_id(this);
    let window: *mut Object = unsafe { msg_send![notification, object] };
    let frame: NSRect = unsafe { msg_send![window, frame] };
    send_event(crate::Event::Window(
        window_id,
        WindowEvent::Moved(LogicalPoint::new(
            frame.origin.x as f32,
            frame.origin.y as f32,
        )),
    ));
}

extern "C" fn window_did_resize(this: *mut Object, _sel: Sel, notification: *mut Object) {
    let window_id = get_window_id(this);
    let window: *mut Object = unsafe { msg_send![notification, object] };
    let content_view: *mut Object = unsafe { msg_send![window, contentView] };
    let frame: NSRect = unsafe { msg_send![content_view, frame] };
    send_event(crate::Event::Window(
        window_id,
        WindowEvent::Resized(LogicalSize::new(
            frame.size.width as f32,
            frame.size.height as f32,
        )),
    ));
}

extern "C" fn window_will_close(this: *mut Object, _sel: Sel, _notification: *mut Object) {
    let window_id = get_window_id(this);
    send_event(crate::Event::Window(window_id, WindowEvent::Closed));
}

extern "C" fn window_will_enter_fullscreen(
    this: *mut Object,
    _sel: Sel,
    _notification: *mut Object,
) {
    let window_id = get_window_id(this);
    send_event(crate::Event::Window(
        window_id,
        WindowEvent::MacosFullscreenChanged(true),
    ));
}

extern "C" fn window_will_exit_fullscreen(
    this: *mut Object,
    _sel: Sel,
    _notification: *mut Object,
) {
    let window_id = get_window_id(this);
    send_event(crate::Event::Window(
        window_id,
        WindowEvent::MacosFullscreenChanged(false),
    ));
}
