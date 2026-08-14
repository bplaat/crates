/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::null_mut;

use objc2::runtime::{AnyClass, AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};

use super::cocoa::*;
use super::event_loop::{allow_termination_if_last_window, send_event, send_theme_change};
#[cfg(feature = "file_drop")]
use super::file_drop::{perform_file_drop, register_dragged_types};
use crate::{
    CloseRequest, Cursor, KeyboardEvent, LogicalPoint, LogicalSize, MacosTitlebarStyle, MouseEvent,
    Theme, WheelEvent, WindowBuilder, WindowEvent, WindowEvents,
};

#[link(name = "objc", kind = "dylib")]
unsafe extern "C" {
    fn class_getMethodImplementation(
        class: *const AnyClass,
        selector: *const c_void,
    ) -> *const c_void;
}

#[derive(Default)]
struct EventWindowIvars {
    events: Cell<u8>,
    mouse_inside: Cell<bool>,
    primary_button_down: Cell<bool>,
    modifier_keys: Cell<u16>,
}

define_class!(
    #[unsafe(super(NSWindow))]
    #[ivars = EventWindowIvars]
    struct EventWindow;

    impl EventWindow {
        #[unsafe(method(sendEvent:))]
        fn _send_event(&self, event: *mut Object) {
            let window = self as *const EventWindow as *mut Object;
            let events = WindowEvents(self.ivars().events.get());
            if events != WindowEvents::NONE {
                dispatch_input_event(
                    window,
                    event,
                    events,
                    &self.ivars().mouse_inside,
                    &self.ivars().primary_button_down,
                    &self.ivars().modifier_keys,
                );
            }
            let selector = sel!(sendEvent:);
            let implementation = unsafe {
                class_getMethodImplementation(class!(NSWindow).cast(), selector.0)
            };
            let send_event: unsafe extern "C" fn(*mut Object, *const c_void, *mut Object) =
                unsafe { std::mem::transmute(implementation) };
            unsafe { send_event(window, selector.0, event) };
            let event_type: u64 = unsafe { msg_send![event, type] };
            if matches!(event_type, 1..=9 | 22 | 25..=27) {
                let content_view: *mut Object = unsafe { msg_send![window, contentView] };
                let content_view = unsafe { &*(content_view as *const ThemeAwareView) };
                let _: () = unsafe {
                    msg_send![ns_cursor(cursor_from_tag(content_view.ivars().cursor.get())), set]
                };
            }
        }
    }
);

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
    events: Cell<u8>,
    tracking_area: Cell<*mut Object>,
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
                if WindowEvents(self.ivars().events.get()).contains(WindowEvents::THEME_CHANGE) {
                    send_theme_change(theme);
                }
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

        #[unsafe(method(updateTrackingAreas))]
        fn _update_tracking_areas(&self) {
            let view = self as *const ThemeAwareView as *mut Object;
            let old_tracking_area = self.ivars().tracking_area.replace(null_mut());
            if !old_tracking_area.is_null() {
                let _: () = unsafe { msg_send![view, removeTrackingArea:old_tracking_area] };
            }
            if !WindowEvents(self.ivars().events.get()).contains(WindowEvents::MOUSE) {
                return;
            }
            let options = NS_TRACKING_MOUSE_ENTERED_AND_EXITED
                | NS_TRACKING_MOUSE_MOVED
                | NS_TRACKING_ACTIVE_ALWAYS
                | NS_TRACKING_IN_VISIBLE_RECT;
            let tracking_area: *mut Object = unsafe { msg_send![class!(NSTrackingArea), alloc] };
            let tracking_area: *mut Object = unsafe {
                msg_send![tracking_area,
                    initWithRect:NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
                    options:options,
                    owner:view,
                    userInfo:null_mut::<Object>()
                ]
            };
            let _: () = unsafe { msg_send![view, addTrackingArea:tracking_area] };
            self.ivars().tracking_area.set(tracking_area);
            let _: () = unsafe { msg_send![tracking_area, release] };
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

fn macos_mouse_event(
    window: *mut Object,
    event: *mut Object,
    event_type: u64,
) -> (MouseEvent, bool) {
    unsafe {
        let content_view: *mut Object = msg_send![window, contentView];
        let window_point: NSPoint = msg_send![event, locationInWindow];
        let point: NSPoint =
            msg_send![content_view, convertPoint:window_point, fromView:null_mut::<Object>()];
        let bounds: NSRect = msg_send![content_view, bounds];
        let screen: NSPoint = msg_send![window, convertPointToScreen:window_point];
        // NSScreen.screens[0] owns the menu bar and provides a stable origin;
        // mainScreen follows the key window and would jump between displays.
        let screens: *mut Object = msg_send![class!(NSScreen), screens];
        let primary_screen: *mut Object = msg_send![screens, objectAtIndex:0usize];
        let primary_screen_frame: NSRect = msg_send![primary_screen, frame];
        let is_button_event = matches!(event_type, 1..=4 | 25 | 26);
        let button_number: i64 = if is_button_event {
            msg_send![event, buttonNumber]
        } else {
            -1
        };
        let click_count: i64 = if is_button_event {
            msg_send![event, clickCount]
        } else {
            0
        };
        let is_motion_event = matches!(event_type, 5..=7 | 27);
        let delta_x: f64 = if is_motion_event {
            msg_send![event, deltaX]
        } else {
            0.0
        };
        let delta_y: f64 = if is_motion_event {
            msg_send![event, deltaY]
        } else {
            0.0
        };
        let flags: u64 = msg_send![event, modifierFlags];
        let pressed: u64 = msg_send![class!(NSEvent), pressedMouseButtons];
        let mouse = MouseEvent {
            client_x: point.x as f32,
            client_y: (bounds.size.height - point.y) as f32,
            screen_x: screen.x as f32,
            screen_y: (primary_screen_frame.origin.y + primary_screen_frame.size.height - screen.y)
                as f32,
            movement_x: delta_x as f32,
            movement_y: delta_y as f32,
            button: match button_number {
                -1 => -1,
                0 => 0,
                1 => 2,
                2 => 1,
                number => number as i16,
            },
            buttons: ((pressed & 1) | ((pressed & 4) >> 1) | ((pressed & 2) << 1)) as u16,
            detail: click_count.min(i64::from(u16::MAX)) as u16,
            alt_key: flags & NS_EVENT_MODIFIER_FLAG_OPTION != 0,
            ctrl_key: flags & NS_EVENT_MODIFIER_FLAG_CONTROL != 0,
            meta_key: flags & NS_EVENT_MODIFIER_FLAG_COMMAND != 0,
            shift_key: flags & NS_EVENT_MODIFIER_FLAG_SHIFT != 0,
        };
        let inside = point.x >= bounds.origin.x
            && point.x < bounds.origin.x + bounds.size.width
            && point.y >= bounds.origin.y
            && point.y < bounds.origin.y + bounds.size.height;
        (mouse, inside)
    }
}

fn macos_keyboard_event(event: *mut Object) -> KeyboardEvent {
    unsafe {
        let key_code: u16 = msg_send![event, keyCode];
        let key = if let Some(key) = mac_named_key(key_code) {
            key.to_owned()
        } else {
            let chars: NSString = msg_send![event, characters];
            chars.to_string()
        };
        let flags: u64 = msg_send![event, modifierFlags];
        let repeat: Bool = msg_send![event, isARepeat];
        KeyboardEvent {
            key,
            code: mac_code(key_code).into(),
            location: mac_key_location(key_code),
            repeat: repeat == Bool::YES,
            is_composing: false,
            alt_key: flags & NS_EVENT_MODIFIER_FLAG_OPTION != 0,
            ctrl_key: flags & NS_EVENT_MODIFIER_FLAG_CONTROL != 0,
            meta_key: flags & NS_EVENT_MODIFIER_FLAG_COMMAND != 0,
            shift_key: flags & NS_EVENT_MODIFIER_FLAG_SHIFT != 0,
        }
    }
}

fn dispatch_input_event(
    window: *mut Object,
    event: *mut Object,
    events: WindowEvents,
    mouse_inside: &Cell<bool>,
    primary_button_down: &Cell<bool>,
    modifier_keys: &Cell<u16>,
) {
    let event_type: u64 = unsafe { msg_send![event, type] };
    let mouse = || macos_mouse_event(window, event, event_type);
    match event_type {
        1 | 3 | 25 if events.contains(WindowEvents::MOUSE) => {
            let (event, inside) = mouse();
            if inside {
                if event.button == 0 {
                    primary_button_down.set(true);
                }
                send_event(crate::Event::Window(WindowEvent::MouseDown(event)));
            }
        }
        2 | 4 | 26 if events.contains(WindowEvents::MOUSE) => {
            let (event, inside) = mouse();
            let should_click = event.button == 0 && primary_button_down.replace(false);
            if inside {
                send_event(crate::Event::Window(WindowEvent::MouseUp(event.clone())));
                if should_click {
                    send_event(crate::Event::Window(WindowEvent::Click(event)));
                }
            }
        }
        5 | 6 | 7 | 27 if events.contains(WindowEvents::MOUSE) => {
            let (event, inside) = mouse();
            if inside {
                if !mouse_inside.replace(true) {
                    send_event(crate::Event::Window(WindowEvent::MouseEnter(event.clone())));
                }
                send_event(crate::Event::Window(WindowEvent::MouseMove(event)));
            } else if mouse_inside.replace(false) {
                send_event(crate::Event::Window(WindowEvent::MouseLeave(event)));
            }
        }
        8 if events.contains(WindowEvents::MOUSE) => {
            let (event, inside) = mouse();
            if inside && !mouse_inside.replace(true) {
                send_event(crate::Event::Window(WindowEvent::MouseEnter(event)));
            }
        }
        9 if events.contains(WindowEvents::MOUSE) && mouse_inside.replace(false) => {
            send_event(crate::Event::Window(WindowEvent::MouseLeave(mouse().0)));
        }
        10 if events.contains(WindowEvents::KEYBOARD) => {
            send_event(crate::Event::Window(WindowEvent::KeyDown(
                macos_keyboard_event(event),
            )));
        }
        11 if events.contains(WindowEvents::KEYBOARD) => {
            send_event(crate::Event::Window(WindowEvent::KeyUp(
                macos_keyboard_event(event),
            )));
        }
        12 if events.contains(WindowEvents::KEYBOARD) => {
            let key_code: u16 = unsafe { msg_send![event, keyCode] };
            let event = macos_keyboard_event(event);
            let bit = modifier_key_bit(key_code);
            let pressed = modifier_keys.get() & bit == 0;
            modifier_keys.set(modifier_keys.get() ^ bit);
            send_event(crate::Event::Window(if pressed {
                WindowEvent::KeyDown(event)
            } else {
                WindowEvent::KeyUp(event)
            }));
        }
        22 if events.contains(WindowEvents::WHEEL) => {
            let (mouse, inside) = mouse();
            if !inside {
                return;
            }
            let delta_x: f64 = unsafe { msg_send![event, scrollingDeltaX] };
            let delta_y: f64 = unsafe { msg_send![event, scrollingDeltaY] };
            let precise: Bool = unsafe { msg_send![event, hasPreciseScrollingDeltas] };
            send_event(crate::Event::Window(WindowEvent::Wheel(WheelEvent {
                mouse,
                delta_x: -(delta_x as f32),
                delta_y: -(delta_y as f32),
                delta_z: 0.0,
                delta_mode: u32::from(precise == Bool::NO),
            })));
        }
        _ => {}
    }
}

const fn modifier_key_bit(code: u16) -> u16 {
    match code {
        54 => 1 << 0,
        55 => 1 << 1,
        56 => 1 << 2,
        57 => 1 << 3,
        58 => 1 << 4,
        59 => 1 << 5,
        60 => 1 << 6,
        61 => 1 << 7,
        62 => 1 << 8,
        63 => 1 << 9,
        _ => 0,
    }
}

const fn mac_named_key(code: u16) -> Option<&'static str> {
    Some(match code {
        36 => "Enter",
        48 => "Tab",
        49 => " ",
        51 => "Backspace",
        53 => "Escape",
        54 | 55 => "Meta",
        56 | 60 => "Shift",
        57 => "CapsLock",
        58 | 61 => "Alt",
        59 | 62 => "Control",
        63 => "Fn",
        123 => "ArrowLeft",
        124 => "ArrowRight",
        125 => "ArrowDown",
        126 => "ArrowUp",
        _ => return None,
    })
}

const fn mac_code(code: u16) -> &'static str {
    match code {
        0 => "KeyA",
        1 => "KeyS",
        2 => "KeyD",
        3 => "KeyF",
        4 => "KeyH",
        5 => "KeyG",
        6 => "KeyZ",
        7 => "KeyX",
        8 => "KeyC",
        9 => "KeyV",
        11 => "KeyB",
        12 => "KeyQ",
        13 => "KeyW",
        14 => "KeyE",
        15 => "KeyR",
        16 => "KeyY",
        17 => "KeyT",
        18 => "Digit1",
        19 => "Digit2",
        20 => "Digit3",
        21 => "Digit4",
        22 => "Digit6",
        23 => "Digit5",
        25 => "Digit9",
        26 => "Digit7",
        28 => "Digit8",
        29 => "Digit0",
        31 => "KeyO",
        32 => "KeyU",
        34 => "KeyI",
        35 => "KeyP",
        37 => "KeyL",
        38 => "KeyJ",
        40 => "KeyK",
        45 => "KeyN",
        46 => "KeyM",
        36 => "Enter",
        48 => "Tab",
        49 => "Space",
        51 => "Backspace",
        53 => "Escape",
        54 => "MetaRight",
        55 => "MetaLeft",
        56 => "ShiftLeft",
        57 => "CapsLock",
        58 => "AltLeft",
        59 => "ControlLeft",
        60 => "ShiftRight",
        61 => "AltRight",
        62 => "ControlRight",
        63 => "Fn",
        123 => "ArrowLeft",
        124 => "ArrowRight",
        125 => "ArrowDown",
        126 => "ArrowUp",
        _ => "Unidentified",
    }
}

const fn mac_key_location(code: u16) -> u32 {
    match code {
        54 | 60..=62 => 2,
        55 | 56 | 58 | 59 => 1,
        65 | 67 | 69 | 71 | 75 | 76 | 78 | 81 | 82..=92 => 3,
        _ => 0,
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

        #[unsafe(method(windowDidBecomeKey:))]
        fn _window_did_become_key(&self, notification: *mut Object) {
            if notification_window_events(notification).contains(WindowEvents::FOCUS) {
                send_event(crate::Event::Window(WindowEvent::Focus));
            }
        }

        #[unsafe(method(windowDidResignKey:))]
        fn _window_did_resign_key(&self, notification: *mut Object) {
            let window: *mut Object = unsafe { msg_send![notification, object] };
            let window = unsafe { &*(window as *const EventWindow) };
            window.ivars().modifier_keys.set(0);
            window.ivars().primary_button_down.set(false);
            if notification_window_events(notification).contains(WindowEvents::FOCUS) {
                send_event(crate::Event::Window(WindowEvent::Blur));
            }
        }

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

fn notification_window_events(notification: *mut Object) -> WindowEvents {
    let window: *mut Object = unsafe { msg_send![notification, object] };
    let window = unsafe { &*(window as *const EventWindow) };
    WindowEvents(window.ivars().events.get())
}

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
        if !notification_window_events(notification).contains(WindowEvents::MOVE) {
            return;
        }
        let window: *mut Object = unsafe { msg_send![notification, object] };
        let frame: NSRect = unsafe { msg_send![window, frame] };
        send_event(crate::Event::Window(WindowEvent::Move(LogicalPoint::new(
            frame.origin.x as f32,
            frame.origin.y as f32,
        ))));
    }

    fn window_did_resize(&self, notification: *mut Object) {
        if !notification_window_events(notification).contains(WindowEvents::RESIZE) {
            return;
        }
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
    fullscreen: bool,
    windowed_frame: NSRect,
    windowed_style_mask: u64,
    windowed_level: i64,
    macos_titlebar_style: MacosTitlebarStyle,
    drag_view_added: bool,
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
            fullscreen: builder.should_fullscreen,
            windowed_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
            windowed_style_mask: 0,
            windowed_level: 0,
            macos_titlebar_style: builder.macos_titlebar_style,
            drag_view_added: false,
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
        let windowed_rect = NSRect::new(
            if let Some(position) = builder.position {
                NSPoint::new(
                    screen_rect.origin.x + position.x as f64,
                    screen_rect.origin.y + (screen_rect.size.height - builder.size.height as f64)
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
        );
        let window_rect = if builder.should_fullscreen {
            screen_rect
        } else {
            windowed_rect
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

        window_data.windowed_frame = windowed_rect;
        window_data.windowed_style_mask = window_style_mask;
        if builder.should_fullscreen {
            window_data.windowed_style_mask = NS_WINDOW_STYLE_MASK_TITLED
                | NS_WINDOW_STYLE_MASK_CLOSABLE
                | NS_WINDOW_STYLE_MASK_MINIATURIZABLE
                | if builder.resizable {
                    NS_WINDOW_STYLE_MASK_RESIZABLE
                } else {
                    0
                }
                | if builder.macos_titlebar_style == MacosTitlebarStyle::Transparent
                    || builder.macos_titlebar_style == MacosTitlebarStyle::Hidden
                {
                    NS_WINDOW_STYLE_MASK_FULL_SIZE_CONTENT_VIEW
                } else {
                    0
                };
        }

        let window = unsafe {
            let window: *mut Object = msg_send![EventWindow::class(), alloc];
            let window: *mut Object = msg_send![window, initWithContentRect:NSRect::new(NSPoint::new(0.0, 0.0), window_rect.size),
                styleMask:window_style_mask, backing:NS_BACKING_STORE_BUFFERED, defer:false];
            let content_view: *mut Object = msg_send![ThemeAwareView::class(), alloc];
            let content_view: *mut Object = msg_send![content_view, initWithFrame:NSRect::new(NSPoint::new(0.0, 0.0), window_rect.size)];
            let _: () = msg_send![content_view, setAutoresizingMask:NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE];
            let _: () = msg_send![window, setContentView:content_view];
            let theme_view = &*(content_view as *const ThemeAwareView);
            theme_view.ivars().cursor.set(cursor_tag(builder.cursor));
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
            window_data.drag_view_added = true;
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

    fn set_min_size(&mut self, min_size: Option<LogicalSize>) {
        let min_size = min_size.unwrap_or(LogicalSize::new(0.0, 0.0));
        unsafe {
            msg_send![self.0.window, setContentMinSize:NSSize::new(min_size.width as f64, min_size.height as f64)]
        }
    }

    fn set_resizable(&mut self, resizable: bool) {
        if self.0.fullscreen {
            if resizable {
                self.0.windowed_style_mask |= NS_WINDOW_STYLE_MASK_RESIZABLE;
            } else {
                self.0.windowed_style_mask &= !NS_WINDOW_STYLE_MASK_RESIZABLE;
            }
            return;
        }
        let mut style_mask: u64 = unsafe { msg_send![self.0.window, styleMask] };
        if resizable {
            style_mask |= NS_WINDOW_STYLE_MASK_RESIZABLE;
        } else {
            style_mask &= !NS_WINDOW_STYLE_MASK_RESIZABLE;
        }
        unsafe { msg_send![self.0.window, setStyleMask:style_mask] }
    }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        let style_mask: u64 = unsafe { msg_send![self.0.window, styleMask] };
        let native_fullscreen = style_mask & NS_WINDOW_STYLE_MASK_FULL_SCREEN != 0;
        if native_fullscreen {
            if !fullscreen {
                let _: () =
                    unsafe { msg_send![self.0.window, toggleFullScreen:null_mut::<Object>()] };
            }
            return;
        }
        if fullscreen == self.0.fullscreen {
            return;
        }
        unsafe {
            if fullscreen {
                self.0.windowed_frame = msg_send![self.0.window, frame];
                self.0.windowed_style_mask = msg_send![self.0.window, styleMask];
                self.0.windowed_level = msg_send![self.0.window, level];
                let screen: *mut Object = msg_send![self.0.window, screen];
                let screen_frame: NSRect = msg_send![screen, frame];
                let _: () = msg_send![self.0.window, setStyleMask:0u64];
                let _: () = msg_send![self.0.window, setLevel:25i64];
                let _: () = msg_send![self.0.window, setFrame:screen_frame, display:Bool::YES];
            } else {
                let _: () = msg_send![self.0.window, setLevel:self.0.windowed_level];
                let _: () = msg_send![self.0.window, setStyleMask:self.0.windowed_style_mask];
                let _: () =
                    msg_send![self.0.window, setFrame:self.0.windowed_frame, display:Bool::YES];
                if !self.0.drag_view_added
                    && self.0.macos_titlebar_style != MacosTitlebarStyle::Default
                {
                    let content_view: *mut Object = msg_send![self.0.window, contentView];
                    add_drag_view(self.0.window, content_view);
                    self.0.drag_view_added = true;
                }
            }
        }
        self.0.fullscreen = fullscreen;
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

    fn follow_system_theme(&mut self) {
        unsafe {
            let _: () = msg_send![self.0.window, setAppearance:null_mut::<Object>()];
            let content_view: *mut Object = msg_send![self.0.window, contentView];
            let theme_view = &*(content_view as *const ThemeAwareView);
            theme_view
                .ivars()
                .theme
                .set(theme_tag(effective_theme(self.0.window)));
            let _: () = msg_send![content_view, setNeedsDisplay:Bool::YES];
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

    fn enable_events(&mut self, events: WindowEvents) {
        let window = unsafe { &*(self.0.window as *const EventWindow) };
        let old_events = WindowEvents(window.ivars().events.get());
        window.ivars().events.set((old_events | events).0);

        let content_view: *mut Object = unsafe { msg_send![self.0.window, contentView] };
        let content_view = unsafe { &*(content_view as *const ThemeAwareView) };
        content_view
            .ivars()
            .events
            .set((WindowEvents(content_view.ivars().events.get()) | events).0);

        if events.contains(WindowEvents::MOUSE) && !old_events.contains(WindowEvents::MOUSE) {
            let _: () = unsafe { msg_send![self.0.window, setAcceptsMouseMovedEvents:Bool::YES] };
            let view = content_view as *const ThemeAwareView as *mut Object;
            let _: () = unsafe { msg_send![view, updateTrackingAreas] };
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

#[cfg(test)]
mod tests {
    use super::{mac_code, mac_key_location, mac_named_key};

    #[test]
    fn keyboard_metadata_matches_dom_conventions() {
        assert_eq!(mac_code(0), "KeyA");
        assert_eq!(mac_code(18), "Digit1");
        assert_eq!(mac_code(60), "ShiftRight");
        assert_eq!(mac_key_location(60), 2);
        assert_eq!(mac_key_location(82), 3);
        assert_eq!(mac_named_key(123), Some("ArrowLeft"));
    }
}
