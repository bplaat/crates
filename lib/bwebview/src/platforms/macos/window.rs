/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;
use std::ptr::null_mut;

use block2::RcBlock;
use objc2::runtime::{AnyClass, AnyObject as Object, Bool, ClassBuilder, Sel};
use objc2::{class, msg_send, sel};

use super::cocoa::*;
use super::event_loop::{IVAR_PTR, IVAR_PTR_CSTR, send_event};
use crate::{
    KeyCode, LogicalPoint, LogicalSize, MacosTitlebarStyle, Modifiers, MouseButton, Theme,
    WindowBuilder, WindowEvent,
};

pub(super) struct PlatformWindowData {
    pub(super) window: *mut Object,
    pub(super) background_color: Option<u32>,
    pub(super) event_monitor: *mut Object,
}

pub(crate) struct PlatformWindow(pub(super) Box<PlatformWindowData>);

impl PlatformWindow {
    pub(crate) fn new(builder: &WindowBuilder) -> Self {
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
                decl.add_method(
                    sel!(windowDidBecomeKey:),
                    window_did_become_key as extern "C" fn(_, _, _),
                );
                decl.add_method(
                    sel!(windowDidResignKey:),
                    window_did_resign_key as extern "C" fn(_, _, _),
                );
            }
            decl.register();
            let _: () =
                unsafe { msg_send![class!(NSWindow), setAllowsAutomaticWindowTabbing:Bool::NO] };
        }

        // Allocate window data box first so we have a stable ptr
        let mut window_data = Box::new(PlatformWindowData {
            window: null_mut(),
            background_color: builder.background_color,
            event_monitor: null_mut(),
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

        // Add NSTrackingArea to content view for mouse enter/leave events
        unsafe {
            let content_view: *mut Object = msg_send![window, contentView];
            let tracking_options: u64 = NS_TRACKING_MOUSE_ENTERED_AND_EXITED
                | NS_TRACKING_MOUSE_MOVED
                | NS_TRACKING_ACTIVE_ALWAYS
                | NS_TRACKING_IN_VISIBLE_RECT;
            let tracking_area: *mut Object = msg_send![class!(NSTrackingArea), alloc];
            let tracking_area: *mut Object = msg_send![tracking_area,
                initWithRect:NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
                options:tracking_options,
                owner:content_view,
                userInfo:null_mut::<Object>()];
            let _: () = msg_send![content_view, addTrackingArea:tracking_area];
            let _: () = msg_send![tracking_area, release];
        }

        // Install local NSEvent monitor for keyboard and mouse events
        let monitor: *mut Object = unsafe {
            let mask: u64 = NS_EVENT_MASK_KEY_DOWN
                | NS_EVENT_MASK_KEY_UP
                | NS_EVENT_MASK_LEFT_MOUSE_DOWN
                | NS_EVENT_MASK_LEFT_MOUSE_UP
                | NS_EVENT_MASK_RIGHT_MOUSE_DOWN
                | NS_EVENT_MASK_RIGHT_MOUSE_UP
                | NS_EVENT_MASK_OTHER_MOUSE_DOWN
                | NS_EVENT_MASK_OTHER_MOUSE_UP
                | NS_EVENT_MASK_MOUSE_MOVED
                | NS_EVENT_MASK_LEFT_MOUSE_DRAGGED
                | NS_EVENT_MASK_RIGHT_MOUSE_DRAGGED
                | NS_EVENT_MASK_OTHER_MOUSE_DRAGGED
                | NS_EVENT_MASK_MOUSE_ENTERED
                | NS_EVENT_MASK_MOUSE_EXITED
                | NS_EVENT_MASK_SCROLL_WHEEL;

            // Wrap in ManuallyDrop so Rust never calls Drop (Box::from_raw/free) while ObjC
            // holds a reference. addLocalMonitorForEventsMatchingMask:handler: internally calls
            // _Block_copy which, with BLOCK_NEEDS_FREE set, just increments the refcount.
            let block = std::mem::ManuallyDrop::new(
                RcBlock::new_ret::<*mut Object, *mut Object>(|event: *mut Object| {
                    handle_ns_event(event);
                    event
                }),
            );
            let monitor: *mut Object = msg_send![class!(NSEvent), addLocalMonitorForEventsMatchingMask:mask, handler:&**block];
            if !monitor.is_null() {
                let _retained: *mut Object = msg_send![monitor, retain];
            }
            monitor
        };

        window_data.window = window;
        window_data.event_monitor = monitor;
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

fn handle_ns_event(event: *mut Object) {
    let event_type: i64 = unsafe { msg_send![event, type] };
    match event_type {
        NS_EVENT_TYPE_KEY_DOWN => {
            let key_code: u16 = unsafe { msg_send![event, keyCode] };
            let modifier_flags: u64 = unsafe { msg_send![event, modifierFlags] };
            let key = ns_keycode_to_keycode(key_code);
            let modifiers = ns_modifier_flags_to_modifiers(modifier_flags);
            send_event(crate::Event::Window(WindowEvent::KeyDown {
                key,
                modifiers,
            }));
            // Also send Char event
            let characters: NSString = unsafe { msg_send![event, characters] };
            let chars_str = characters.to_string();
            if let Some(ch) = chars_str.chars().next()
                && !ch.is_control()
            {
                send_event(crate::Event::Window(WindowEvent::Char(ch)));
            }
        }
        NS_EVENT_TYPE_KEY_UP => {
            let key_code: u16 = unsafe { msg_send![event, keyCode] };
            let modifier_flags: u64 = unsafe { msg_send![event, modifierFlags] };
            let key = ns_keycode_to_keycode(key_code);
            let modifiers = ns_modifier_flags_to_modifiers(modifier_flags);
            send_event(crate::Event::Window(WindowEvent::KeyUp { key, modifiers }));
        }
        NS_EVENT_TYPE_LEFT_MOUSE_DOWN => {
            let pos = ns_event_mouse_position(event);
            send_event(crate::Event::Window(WindowEvent::MouseDown {
                button: MouseButton::Left,
                position: pos,
            }));
        }
        NS_EVENT_TYPE_LEFT_MOUSE_UP => {
            let pos = ns_event_mouse_position(event);
            send_event(crate::Event::Window(WindowEvent::MouseUp {
                button: MouseButton::Left,
                position: pos,
            }));
        }
        NS_EVENT_TYPE_RIGHT_MOUSE_DOWN => {
            let pos = ns_event_mouse_position(event);
            send_event(crate::Event::Window(WindowEvent::MouseDown {
                button: MouseButton::Right,
                position: pos,
            }));
        }
        NS_EVENT_TYPE_RIGHT_MOUSE_UP => {
            let pos = ns_event_mouse_position(event);
            send_event(crate::Event::Window(WindowEvent::MouseUp {
                button: MouseButton::Right,
                position: pos,
            }));
        }
        NS_EVENT_TYPE_OTHER_MOUSE_DOWN => {
            let button_number: i64 = unsafe { msg_send![event, buttonNumber] };
            let button = ns_button_number_to_mouse_button(button_number);
            let pos = ns_event_mouse_position(event);
            send_event(crate::Event::Window(WindowEvent::MouseDown {
                button,
                position: pos,
            }));
        }
        NS_EVENT_TYPE_OTHER_MOUSE_UP => {
            let button_number: i64 = unsafe { msg_send![event, buttonNumber] };
            let button = ns_button_number_to_mouse_button(button_number);
            let pos = ns_event_mouse_position(event);
            send_event(crate::Event::Window(WindowEvent::MouseUp {
                button,
                position: pos,
            }));
        }
        NS_EVENT_TYPE_MOUSE_MOVED
        | NS_EVENT_TYPE_LEFT_MOUSE_DRAGGED
        | NS_EVENT_TYPE_RIGHT_MOUSE_DRAGGED
        | NS_EVENT_TYPE_OTHER_MOUSE_DRAGGED => {
            let pos = ns_event_mouse_position(event);
            send_event(crate::Event::Window(WindowEvent::MouseMove(pos)));
        }
        NS_EVENT_TYPE_MOUSE_ENTERED => {
            send_event(crate::Event::Window(WindowEvent::MouseEnter));
        }
        NS_EVENT_TYPE_MOUSE_EXITED => {
            send_event(crate::Event::Window(WindowEvent::MouseLeave));
        }
        NS_EVENT_TYPE_SCROLL_WHEEL => {
            let delta_x: f64 = unsafe { msg_send![event, scrollingDeltaX] };
            let delta_y: f64 = unsafe { msg_send![event, scrollingDeltaY] };
            send_event(crate::Event::Window(WindowEvent::MouseWheel {
                delta_x: -delta_x as f32,
                delta_y: -delta_y as f32,
            }));
        }
        _ => {}
    }
}

fn ns_event_mouse_position(event: *mut Object) -> LogicalPoint {
    let location: NSPoint = unsafe { msg_send![event, locationInWindow] };
    let ns_window: *mut Object = unsafe { msg_send![event, window] };
    if ns_window.is_null() {
        return LogicalPoint::new(location.x as f32, location.y as f32);
    }
    let content_view: *mut Object = unsafe { msg_send![ns_window, contentView] };
    let frame: NSRect = unsafe { msg_send![content_view, frame] };
    let y_flipped = frame.size.height - location.y;
    LogicalPoint::new(location.x as f32, y_flipped as f32)
}

fn ns_button_number_to_mouse_button(button: i64) -> MouseButton {
    match button {
        0 => MouseButton::Left,
        1 => MouseButton::Right,
        2 => MouseButton::Middle,
        3 => MouseButton::Back,
        4 => MouseButton::Forward,
        _ => MouseButton::Middle,
    }
}

fn ns_modifier_flags_to_modifiers(flags: u64) -> Modifiers {
    Modifiers {
        shift: (flags & NS_EVENT_MODIFIER_FLAG_SHIFT) != 0,
        ctrl: (flags & NS_EVENT_MODIFIER_FLAG_CONTROL) != 0,
        alt: (flags & NS_EVENT_MODIFIER_FLAG_OPTION) != 0,
        meta: (flags & NS_EVENT_MODIFIER_FLAG_COMMAND) != 0,
    }
}

fn ns_keycode_to_keycode(code: u16) -> KeyCode {
    match code {
        0 => KeyCode::A,
        11 => KeyCode::B,
        8 => KeyCode::C,
        2 => KeyCode::D,
        14 => KeyCode::E,
        3 => KeyCode::F,
        5 => KeyCode::G,
        4 => KeyCode::H,
        34 => KeyCode::I,
        38 => KeyCode::J,
        40 => KeyCode::K,
        37 => KeyCode::L,
        46 => KeyCode::M,
        45 => KeyCode::N,
        31 => KeyCode::O,
        35 => KeyCode::P,
        12 => KeyCode::Q,
        15 => KeyCode::R,
        1 => KeyCode::S,
        17 => KeyCode::T,
        32 => KeyCode::U,
        9 => KeyCode::V,
        13 => KeyCode::W,
        7 => KeyCode::X,
        16 => KeyCode::Y,
        6 => KeyCode::Z,
        29 => KeyCode::Key0,
        18 => KeyCode::Key1,
        19 => KeyCode::Key2,
        20 => KeyCode::Key3,
        21 => KeyCode::Key4,
        23 => KeyCode::Key5,
        22 => KeyCode::Key6,
        26 => KeyCode::Key7,
        28 => KeyCode::Key8,
        25 => KeyCode::Key9,
        122 => KeyCode::F1,
        120 => KeyCode::F2,
        99 => KeyCode::F3,
        118 => KeyCode::F4,
        96 => KeyCode::F5,
        97 => KeyCode::F6,
        98 => KeyCode::F7,
        100 => KeyCode::F8,
        101 => KeyCode::F9,
        109 => KeyCode::F10,
        103 => KeyCode::F11,
        111 => KeyCode::F12,
        51 => KeyCode::Backspace,
        48 => KeyCode::Tab,
        36 => KeyCode::Enter,
        53 => KeyCode::Escape,
        49 => KeyCode::Space,
        117 => KeyCode::Delete,
        114 => KeyCode::Insert,
        123 => KeyCode::Left,
        124 => KeyCode::Right,
        126 => KeyCode::Up,
        125 => KeyCode::Down,
        115 => KeyCode::Home,
        119 => KeyCode::End,
        116 => KeyCode::PageUp,
        121 => KeyCode::PageDown,
        56 | 60 => KeyCode::Shift,
        59 | 62 => KeyCode::Control,
        58 | 61 => KeyCode::Alt,
        55 | 54 => KeyCode::Meta,
        57 => KeyCode::CapsLock,
        _ => KeyCode::Unknown(code as u32),
    }
}

extern "C" fn window_did_move(_this: *mut Object, _sel: Sel, notification: *mut Object) {
    let window: *mut Object = unsafe { msg_send![notification, object] };
    let frame: NSRect = unsafe { msg_send![window, frame] };
    send_event(crate::Event::Window(WindowEvent::Move(LogicalPoint::new(
        frame.origin.x as f32,
        frame.origin.y as f32,
    ))));
}

extern "C" fn window_did_resize(_this: *mut Object, _sel: Sel, notification: *mut Object) {
    let window: *mut Object = unsafe { msg_send![notification, object] };
    let content_view: *mut Object = unsafe { msg_send![window, contentView] };
    let frame: NSRect = unsafe { msg_send![content_view, frame] };
    send_event(crate::Event::Window(WindowEvent::Resize(LogicalSize::new(
        frame.size.width as f32,
        frame.size.height as f32,
    ))));
}

extern "C" fn window_will_close(_this: *mut Object, _sel: Sel, _notification: *mut Object) {
    send_event(crate::Event::Window(WindowEvent::Close));
}

extern "C" fn window_will_enter_fullscreen(
    _this: *mut Object,
    _sel: Sel,
    _notification: *mut Object,
) {
    send_event(crate::Event::Window(WindowEvent::MacosFullscreenChange(
        true,
    )));
}

extern "C" fn window_will_exit_fullscreen(
    _this: *mut Object,
    _sel: Sel,
    _notification: *mut Object,
) {
    send_event(crate::Event::Window(WindowEvent::MacosFullscreenChange(
        false,
    )));
}

extern "C" fn window_did_become_key(_this: *mut Object, _sel: Sel, _notification: *mut Object) {
    send_event(crate::Event::Window(WindowEvent::Focus));
}

extern "C" fn window_did_resign_key(_this: *mut Object, _sel: Sel, _notification: *mut Object) {
    send_event(crate::Event::Window(WindowEvent::Unfocus));
}
