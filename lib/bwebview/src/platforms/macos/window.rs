/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;
use std::mem::ManuallyDrop;

use objc2::runtime::{AnyClass, AnyObject as Object, Bool, ClassBuilder, Sel};
use objc2::{class, msg_send, sel};

use super::cocoa::*;
use super::event_loop::{IVAR_PTR, IVAR_PTR_CSTR};
use crate::{
    Key, LogicalPoint, LogicalSize, MacosTitlebarStyle, Modifiers, MouseButton, Theme,
    WindowBuilder, WindowHandler, WindowId,
};

pub(super) struct PlatformWindowData {
    pub(super) window_id: WindowId,
    pub(super) window: *mut Object,
    pub(super) background_color: Option<u32>,
    #[cfg(feature = "webview")]
    pub(super) webview: *mut Object,
    pub(super) window_handler: Option<*mut dyn WindowHandler>,
    #[cfg(feature = "webview")]
    pub(super) webview_handler: Option<*mut dyn crate::WebviewHandler>,
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
                    sel!(windowShouldClose:),
                    window_should_close as extern "C" fn(_, _, _) -> Bool,
                );
                decl.add_method(
                    sel!(windowDidMove:),
                    window_did_move as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(windowDidResize:),
                    window_did_resize as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(windowDidBecomeKey:),
                    window_did_become_key as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(windowDidResignKey:),
                    window_did_resign_key as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(keyDown:),
                    window_key_down as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(keyUp:),
                    window_key_up as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(mouseDown:),
                    window_mouse_down as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(rightMouseDown:),
                    window_right_mouse_down as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(otherMouseDown:),
                    window_other_mouse_down as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(mouseUp:),
                    window_mouse_up as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(rightMouseUp:),
                    window_right_mouse_up as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(otherMouseUp:),
                    window_other_mouse_up as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(mouseMoved:),
                    window_mouse_moved as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(mouseDragged:),
                    window_mouse_moved as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(rightMouseDragged:),
                    window_mouse_moved as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(otherMouseDragged:),
                    window_mouse_moved as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(scrollWheel:),
                    window_scroll_wheel as extern "C" fn(_, _, _),
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
            #[cfg(feature = "webview")]
            webview: std::ptr::null_mut(),
            window_handler: builder.window_handler,
            #[cfg(feature = "webview")]
            webview_handler: None,
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
            let _: () = msg_send![window, setAcceptsMouseMovedEvents:Bool::YES];
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
            #[cfg(feature = "webview")]
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

// --- Helper: get data pointer from delegate ivar ---
unsafe fn get_data(this: *mut Object) -> *mut PlatformWindowData {
    #[allow(deprecated)]
    let ptr = unsafe { *(*this).get_ivar::<*const c_void>(IVAR_PTR) };
    ptr as *mut PlatformWindowData
}

// --- Helper: construct a temporary ManuallyDrop<Window> from raw data pointer ---
unsafe fn make_temp_window(data: *mut PlatformWindowData) -> ManuallyDrop<crate::Window> {
    ManuallyDrop::new(crate::Window {
        id: unsafe { (*data).window_id },
        platform: PlatformWindow(unsafe { Box::from_raw(data) }),
        window_handler: unsafe { (*data).window_handler },
    })
}

// --- Helper: NSEvent keyCode -> Key ---
fn keycode_to_key(keycode: u16) -> Key {
    match keycode {
        0 => Key::A,
        1 => Key::S,
        2 => Key::D,
        3 => Key::F,
        4 => Key::H,
        5 => Key::G,
        6 => Key::Z,
        7 => Key::X,
        8 => Key::C,
        9 => Key::V,
        11 => Key::B,
        12 => Key::Q,
        13 => Key::W,
        14 => Key::E,
        15 => Key::R,
        16 => Key::Y,
        17 => Key::T,
        18 => Key::Digit1,
        19 => Key::Digit2,
        20 => Key::Digit3,
        21 => Key::Digit4,
        22 => Key::Digit6,
        23 => Key::Digit5,
        24 => Key::Equal,
        25 => Key::Digit9,
        26 => Key::Digit7,
        27 => Key::Minus,
        28 => Key::Digit8,
        29 => Key::Digit0,
        30 => Key::BracketRight,
        31 => Key::O,
        32 => Key::U,
        33 => Key::BracketLeft,
        34 => Key::I,
        35 => Key::P,
        36 => Key::Enter,
        37 => Key::L,
        38 => Key::J,
        39 => Key::Quote,
        40 => Key::K,
        41 => Key::Semicolon,
        42 => Key::Backslash,
        43 => Key::Comma,
        44 => Key::Slash,
        45 => Key::N,
        46 => Key::M,
        47 => Key::Period,
        48 => Key::Tab,
        49 => Key::Space,
        50 => Key::Backtick,
        51 => Key::Backspace,
        53 => Key::Escape,
        54 | 55 => Key::Meta,
        56 | 60 => Key::Shift,
        57 => Key::CapsLock,
        58 | 61 => Key::Alt,
        59 | 62 => Key::Control,
        65 => Key::NumpadDecimal,
        67 => Key::NumpadMultiply,
        69 => Key::NumpadAdd,
        75 => Key::NumpadDivide,
        76 => Key::NumpadEnter,
        78 => Key::NumpadSubtract,
        82 => Key::Numpad0,
        83 => Key::Numpad1,
        84 => Key::Numpad2,
        85 => Key::Numpad3,
        86 => Key::Numpad4,
        87 => Key::Numpad5,
        88 => Key::Numpad6,
        89 => Key::Numpad7,
        91 => Key::Numpad8,
        92 => Key::Numpad9,
        96 => Key::F5,
        97 => Key::F6,
        98 => Key::F7,
        99 => Key::F3,
        100 => Key::F8,
        101 => Key::F9,
        103 => Key::F11,
        105 => Key::Unknown, // F13
        107 => Key::Unknown, // F14
        109 => Key::F10,
        111 => Key::F12,
        114 => Key::Insert,
        115 => Key::Home,
        116 => Key::PageUp,
        117 => Key::Delete,
        118 => Key::F4,
        119 => Key::End,
        120 => Key::F2,
        121 => Key::PageDown,
        122 => Key::F1,
        123 => Key::ArrowLeft,
        124 => Key::ArrowRight,
        125 => Key::ArrowDown,
        126 => Key::ArrowUp,
        _ => Key::Unknown,
    }
}

// NSEventModifierFlags constants
const NS_EVENT_MODIFIER_FLAG_SHIFT: u64 = 1 << 17;
const NS_EVENT_MODIFIER_FLAG_CONTROL_KEY: u64 = 1 << 18;
const NS_EVENT_MODIFIER_FLAG_OPTION: u64 = 1 << 19;
const NS_EVENT_MODIFIER_FLAG_COMMAND: u64 = 1 << 20;

fn modifiers_from_flags(flags: u64) -> Modifiers {
    let mut mods = Modifiers::empty();
    if flags & NS_EVENT_MODIFIER_FLAG_SHIFT != 0 {
        mods = mods | Modifiers::SHIFT;
    }
    if flags & NS_EVENT_MODIFIER_FLAG_CONTROL_KEY != 0 {
        mods = mods | Modifiers::CTRL;
    }
    if flags & NS_EVENT_MODIFIER_FLAG_OPTION != 0 {
        mods = mods | Modifiers::ALT;
    }
    if flags & NS_EVENT_MODIFIER_FLAG_COMMAND != 0 {
        mods = mods | Modifiers::META;
    }
    mods
}

fn button_number_to_mouse_button(btn: u64) -> MouseButton {
    match btn {
        0 => MouseButton::Left,
        1 => MouseButton::Right,
        2 => MouseButton::Middle,
        3 => MouseButton::Back,
        4 => MouseButton::Forward,
        n => MouseButton::Other(n as u8),
    }
}

// --- Delegate callbacks ---

extern "C" fn window_should_close(this: *mut Object, _sel: Sel, _sender: *mut Object) -> Bool {
    unsafe {
        let data = get_data(this);
        if let Some(h_ptr) = (*data).window_handler {
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(data);
            let allow = handler.on_close(&mut window);
            std::mem::forget(ManuallyDrop::into_inner(window).platform.0);
            if !allow {
                return Bool::NO;
            }
        }
        Bool::YES
    }
}

extern "C" fn window_did_move(this: *mut Object, _sel: Sel, notification: *mut Object) {
    unsafe {
        let data = get_data(this);
        if let Some(h_ptr) = (*data).window_handler {
            let window_obj: *mut Object = msg_send![notification, object];
            let frame: NSRect = msg_send![window_obj, frame];
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(data);
            handler.on_move(&mut window, frame.origin.x as i32, frame.origin.y as i32);
            std::mem::forget(ManuallyDrop::into_inner(window).platform.0);
        }
    }
}

extern "C" fn window_did_resize(this: *mut Object, _sel: Sel, notification: *mut Object) {
    unsafe {
        let data = get_data(this);
        if let Some(h_ptr) = (*data).window_handler {
            let window_obj: *mut Object = msg_send![notification, object];
            let content_view: *mut Object = msg_send![window_obj, contentView];
            let frame: NSRect = msg_send![content_view, frame];
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(data);
            handler.on_resize(&mut window, frame.size.width as u32, frame.size.height as u32);
            std::mem::forget(ManuallyDrop::into_inner(window).platform.0);
        }
    }
}

extern "C" fn window_did_become_key(this: *mut Object, _sel: Sel, _notification: *mut Object) {
    unsafe {
        let data = get_data(this);
        if let Some(h_ptr) = (*data).window_handler {
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(data);
            handler.on_focus(&mut window);
            std::mem::forget(ManuallyDrop::into_inner(window).platform.0);
        }
    }
}

extern "C" fn window_did_resign_key(this: *mut Object, _sel: Sel, _notification: *mut Object) {
    unsafe {
        let data = get_data(this);
        if let Some(h_ptr) = (*data).window_handler {
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(data);
            handler.on_blur(&mut window);
            std::mem::forget(ManuallyDrop::into_inner(window).platform.0);
        }
    }
}

extern "C" fn window_key_down(this: *mut Object, _sel: Sel, event: *mut Object) {
    unsafe {
        let data = get_data(this);
        if let Some(h_ptr) = (*data).window_handler {
            let keycode: u16 = msg_send![event, keyCode];
            let flags: u64 = msg_send![event, modifierFlags];
            let key = keycode_to_key(keycode);
            let mods = modifiers_from_flags(flags);
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(data);
            handler.on_key_down(&mut window, key, mods);
            std::mem::forget(ManuallyDrop::into_inner(window).platform.0);
        }
    }
}

extern "C" fn window_key_up(this: *mut Object, _sel: Sel, event: *mut Object) {
    unsafe {
        let data = get_data(this);
        if let Some(h_ptr) = (*data).window_handler {
            let keycode: u16 = msg_send![event, keyCode];
            let flags: u64 = msg_send![event, modifierFlags];
            let key = keycode_to_key(keycode);
            let mods = modifiers_from_flags(flags);
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(data);
            handler.on_key_up(&mut window, key, mods);
            std::mem::forget(ManuallyDrop::into_inner(window).platform.0);
        }
    }
}

fn dispatch_mouse_down(data: *mut PlatformWindowData, event: *mut Object, button: MouseButton) {
    unsafe {
        if let Some(h_ptr) = (*data).window_handler {
            let loc: NSPoint = msg_send![event, locationInWindow];
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(data);
            handler.on_mouse_down(&mut window, button, loc.x, loc.y);
            std::mem::forget(ManuallyDrop::into_inner(window).platform.0);
        }
    }
}

fn dispatch_mouse_up(data: *mut PlatformWindowData, event: *mut Object, button: MouseButton) {
    unsafe {
        if let Some(h_ptr) = (*data).window_handler {
            let loc: NSPoint = msg_send![event, locationInWindow];
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(data);
            handler.on_mouse_up(&mut window, button, loc.x, loc.y);
            std::mem::forget(ManuallyDrop::into_inner(window).platform.0);
        }
    }
}

extern "C" fn window_mouse_down(this: *mut Object, _sel: Sel, event: *mut Object) {
    let data = unsafe { get_data(this) };
    dispatch_mouse_down(data, event, MouseButton::Left);
}

extern "C" fn window_right_mouse_down(this: *mut Object, _sel: Sel, event: *mut Object) {
    let data = unsafe { get_data(this) };
    dispatch_mouse_down(data, event, MouseButton::Right);
}

extern "C" fn window_other_mouse_down(this: *mut Object, _sel: Sel, event: *mut Object) {
    let data = unsafe { get_data(this) };
    let btn: u64 = unsafe { msg_send![event, buttonNumber] };
    dispatch_mouse_down(data, event, button_number_to_mouse_button(btn));
}

extern "C" fn window_mouse_up(this: *mut Object, _sel: Sel, event: *mut Object) {
    let data = unsafe { get_data(this) };
    dispatch_mouse_up(data, event, MouseButton::Left);
}

extern "C" fn window_right_mouse_up(this: *mut Object, _sel: Sel, event: *mut Object) {
    let data = unsafe { get_data(this) };
    dispatch_mouse_up(data, event, MouseButton::Right);
}

extern "C" fn window_other_mouse_up(this: *mut Object, _sel: Sel, event: *mut Object) {
    let data = unsafe { get_data(this) };
    let btn: u64 = unsafe { msg_send![event, buttonNumber] };
    dispatch_mouse_up(data, event, button_number_to_mouse_button(btn));
}

extern "C" fn window_mouse_moved(this: *mut Object, _sel: Sel, event: *mut Object) {
    unsafe {
        let data = get_data(this);
        if let Some(h_ptr) = (*data).window_handler {
            let loc: NSPoint = msg_send![event, locationInWindow];
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(data);
            handler.on_mouse_move(&mut window, loc.x, loc.y);
            std::mem::forget(ManuallyDrop::into_inner(window).platform.0);
        }
    }
}

extern "C" fn window_scroll_wheel(this: *mut Object, _sel: Sel, event: *mut Object) {
    unsafe {
        let data = get_data(this);
        if let Some(h_ptr) = (*data).window_handler {
            let delta_x: f64 = msg_send![event, scrollingDeltaX];
            let delta_y: f64 = msg_send![event, scrollingDeltaY];
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(data);
            handler.on_wheel(&mut window, delta_x, delta_y);
            std::mem::forget(ManuallyDrop::into_inner(window).platform.0);
        }
    }
}

extern "C" fn window_will_enter_fullscreen(
    this: *mut Object,
    _sel: Sel,
    _notification: *mut Object,
) {
    unsafe {
        let data = get_data(this);
        if let Some(h_ptr) = (*data).window_handler {
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(data);
            handler.on_fullscreen_change(&mut window, true);
            std::mem::forget(ManuallyDrop::into_inner(window).platform.0);
        }
    }
}

extern "C" fn window_will_exit_fullscreen(
    this: *mut Object,
    _sel: Sel,
    _notification: *mut Object,
) {
    unsafe {
        let data = get_data(this);
        if let Some(h_ptr) = (*data).window_handler {
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(data);
            handler.on_fullscreen_change(&mut window, false);
            std::mem::forget(ManuallyDrop::into_inner(window).platform.0);
        }
    }
}
