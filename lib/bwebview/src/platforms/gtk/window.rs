/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashSet;
use std::ffi::{CStr, CString, c_void};
use std::ptr::{null, null_mut};

use super::event_loop::{primary_monitor_rect, send_event, send_theme_change, system_theme};
use super::headers::*;
#[cfg(feature = "progress_bar")]
use super::progress_bar::update_progress_bar;
#[cfg(feature = "remember_window_state")]
use super::window_state::{load_window_state, save_window_state};
use crate::{
    CloseRequest, Cursor, KeyboardEvent, LogicalPoint, LogicalSize, MouseEvent, Theme, WheelEvent,
    WindowBuilder, WindowEvent, WindowEvents,
};

pub(super) struct WindowData {
    pub(super) window: *mut GtkWindow,
    pub(super) background_color: Option<u32>,
    pub(super) theme: Theme,
    pub(super) follows_system_theme: bool,
    pub(super) is_wayland: bool,
    pub(super) cursor: Cursor,
    pub(super) events: WindowEvents,
    pub(super) last_mouse: Option<LogicalPoint>,
    primary_button_down: bool,
    pressed_keys: HashSet<u16>,
    pub(super) webview: *mut GtkWidget,
    webview_events: WindowEvents,
    #[cfg(feature = "remember_window_state")]
    pub(super) remember_window_state: bool,
    #[cfg(feature = "file_drop")]
    pub(super) allow_file_drop: bool,
}

pub(crate) struct PlatformWindow(pub(super) Box<WindowData>);

impl PlatformWindow {
    pub(crate) fn new(builder: &WindowBuilder) -> Self {
        let is_wayland = unsafe {
            CStr::from_ptr(gdk_display_get_name(gdk_display_get_default()))
                .to_string_lossy()
                .contains("wayland")
        };

        let settings = unsafe { gtk_settings_get_default() };

        // Apply an explicit application theme preference
        if let Some(theme) = builder.theme {
            unsafe {
                g_object_set(
                    settings as *mut GObject,
                    c"gtk-application-prefer-dark-theme".as_ptr(),
                    if theme == Theme::Dark { 1 } else { 0 } as *const c_void,
                    null::<c_void>(),
                );
            }
        }

        let mut window_data = Box::new(WindowData {
            window: null_mut(),
            background_color: builder.background_color,
            theme: builder.theme.unwrap_or_else(system_theme),
            follows_system_theme: builder.theme.is_none(),
            is_wayland,
            cursor: builder.cursor,
            events: WindowEvents::NONE,
            last_mouse: None,
            primary_button_down: false,
            pressed_keys: HashSet::new(),
            webview: null_mut(),
            webview_events: WindowEvents::NONE,
            #[cfg(feature = "remember_window_state")]
            remember_window_state: builder.remember_window_state,
            #[cfg(feature = "file_drop")]
            allow_file_drop: builder.allow_file_drop,
        });

        // Create window
        let window = unsafe {
            let window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
            let title = CString::new(builder.title.clone()).expect("Can't convert to CString");
            gtk_window_set_title(window, title.as_ptr());
            gtk_window_set_default_size(
                window,
                builder.size.width as i32,
                builder.size.height as i32,
            );
            gtk_window_set_resizable(window, builder.resizable);
            if let Some(min_size) = builder.min_size {
                gtk_widget_set_size_request(
                    window as *mut GtkWidget,
                    min_size.width as i32,
                    min_size.height as i32,
                );
            }
            if let Some(color) = builder.background_color {
                let rgba = GdkRGBA {
                    red: ((color >> 16) & 0xFF) as f64 / 255.0,
                    green: ((color >> 8) & 0xFF) as f64 / 255.0,
                    blue: (color & 0xFF) as f64 / 255.0,
                    alpha: 1.0,
                };
                gtk_widget_override_background_color(
                    window as *mut GtkWidget,
                    GTK_STATE_FLAG_NORMAL,
                    &rgba,
                );
            }
            let monitor_rect = if let Some(monitor) = builder.monitor {
                monitor.rect()
            } else {
                primary_monitor_rect()
            };
            if let Some(position) = builder.position {
                gtk_window_move(
                    window,
                    position.x as i32 + monitor_rect.x,
                    position.y as i32 + monitor_rect.y,
                );
            }
            if builder.should_fullscreen {
                gtk_window_move(window, monitor_rect.x, monitor_rect.y);
                gtk_window_fullscreen(window);
            }
            if builder.should_center {
                if !is_wayland {
                    gtk_window_move(
                        window,
                        monitor_rect.x + (monitor_rect.width - builder.size.width as i32) / 2,
                        monitor_rect.y + (monitor_rect.height - builder.size.height as i32) / 2,
                    );
                } else {
                    gtk_window_set_position(window, GTK_WIN_POS_CENTER);
                }
            }
            #[cfg(feature = "remember_window_state")]
            if builder.remember_window_state {
                load_window_state(window);
            }

            g_signal_connect_data(
                window as *mut GObject,
                c"destroy".as_ptr(),
                gtk_main_quit as *const c_void,
                null(),
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window as *mut GObject,
                c"delete-event".as_ptr(),
                window_on_close as *const c_void,
                window_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window as *mut GObject,
                c"realize".as_ptr(),
                window_on_realize as *const c_void,
                window_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            window
        };

        window_data.window = window;
        unsafe { gtk_widget_show(window as *mut GtkWidget) };
        PlatformWindow(window_data)
    }
}

impl crate::WindowInterface for PlatformWindow {
    fn close(&mut self) {
        #[cfg(feature = "remember_window_state")]
        if self.0.remember_window_state {
            save_window_state(self.0.window);
        }
        unsafe { gtk_widget_destroy(self.0.window as *mut GtkWidget) };
    }

    fn set_title(&mut self, title: impl AsRef<str>) {
        let title = CString::new(title.as_ref()).expect("Can't convert to CString");
        unsafe { gtk_window_set_title(self.0.window, title.as_ptr()) };
    }

    fn position(&self) -> LogicalPoint {
        let mut x = 0;
        let mut y = 0;
        unsafe { gtk_window_get_position(self.0.window, &mut x, &mut y) };
        LogicalPoint::new(x as f32, y as f32)
    }

    fn size(&self) -> LogicalSize {
        let mut width = 0;
        let mut height = 0;
        unsafe { gtk_window_get_size(self.0.window, &mut width, &mut height) };
        LogicalSize::new(width as f32, height as f32)
    }

    fn set_position(&mut self, point: LogicalPoint) {
        let primary_monitor_rect = primary_monitor_rect();
        unsafe {
            gtk_window_move(
                self.0.window,
                point.x as i32 + primary_monitor_rect.x,
                point.y as i32 + primary_monitor_rect.y,
            )
        }
    }

    fn set_size(&mut self, size: LogicalSize) {
        unsafe { gtk_window_set_default_size(self.0.window, size.width as i32, size.height as i32) }
    }

    fn set_min_size(&mut self, min_size: Option<LogicalSize>) {
        let min_size = min_size.unwrap_or(LogicalSize::new(-1.0, -1.0));
        unsafe {
            gtk_widget_set_size_request(
                self.0.window as *mut GtkWidget,
                min_size.width as i32,
                min_size.height as i32,
            )
        }
    }

    fn set_resizable(&mut self, resizable: bool) {
        unsafe { gtk_window_set_resizable(self.0.window, resizable) }
    }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        unsafe {
            if fullscreen {
                gtk_window_fullscreen(self.0.window);
            } else {
                gtk_window_unfullscreen(self.0.window);
            }
        }
    }

    fn set_theme(&mut self, theme: Theme) {
        self.0.theme = theme;
        self.0.follows_system_theme = false;
        unsafe {
            let settings = gtk_settings_get_default();
            g_object_set(
                settings as *mut GObject,
                c"gtk-application-prefer-dark-theme".as_ptr(),
                if theme == Theme::Dark { 1 } else { 0 } as *const c_void,
                null::<c_void>(),
            );
        }
    }

    fn follow_system_theme(&mut self) {
        self.0.follows_system_theme = true;
        unsafe {
            g_object_set(
                gtk_settings_get_default() as *mut GObject,
                c"gtk-application-prefer-dark-theme".as_ptr(),
                0 as *const c_void,
                null::<c_void>(),
            );
        }
        self.0.theme = system_theme();
    }

    fn set_background_color(&mut self, color: u32) {
        self.0.background_color = Some(color);
        unsafe {
            let rgba = GdkRGBA {
                red: ((color >> 16) & 0xFF) as f64 / 255.0,
                green: ((color >> 8) & 0xFF) as f64 / 255.0,
                blue: (color & 0xFF) as f64 / 255.0,
                alpha: 1.0,
            };
            gtk_widget_override_background_color(
                self.0.window as *mut GtkWidget,
                GTK_STATE_FLAG_NORMAL,
                &rgba,
            );
        }
    }

    fn set_cursor(&mut self, cursor: Cursor) {
        self.0.cursor = cursor;
        apply_cursor(&self.0);
    }

    fn enable_events(&mut self, events: WindowEvents) {
        let data = self.0.as_mut() as *mut WindowData as *const c_void;
        let window = self.0.window;
        unsafe {
            if events.contains(WindowEvents::MOVE)
                && !self.0.events.contains(WindowEvents::MOVE)
                && !self.0.is_wayland
            {
                connect_event(
                    window,
                    c"configure-event",
                    window_on_move as *const c_void,
                    data,
                );
            }
            if events.contains(WindowEvents::RESIZE)
                && !self.0.events.contains(WindowEvents::RESIZE)
            {
                connect_event(
                    window,
                    c"size-allocate",
                    window_on_resize as *const c_void,
                    data,
                );
            }
            if events.contains(WindowEvents::THEME_CHANGE)
                && !self.0.events.contains(WindowEvents::THEME_CHANGE)
            {
                connect_event(
                    window,
                    c"style-updated",
                    window_on_theme_change as *const c_void,
                    data,
                );
            }
            if events.contains(WindowEvents::FOCUS) && !self.0.events.contains(WindowEvents::FOCUS)
            {
                connect_event(
                    window,
                    c"focus-in-event",
                    window_on_focus as *const c_void,
                    data,
                );
                connect_event(
                    window,
                    c"focus-out-event",
                    window_on_blur as *const c_void,
                    data,
                );
            }
            if events.contains(WindowEvents::MOUSE) && !self.0.events.contains(WindowEvents::MOUSE)
            {
                // Preserve GTK's existing mask and support enabling events after realization.
                gtk_widget_add_events(
                    window as *mut GtkWidget,
                    GDK_POINTER_MOTION_MASK
                        | GDK_BUTTON_PRESS_MASK
                        | GDK_BUTTON_RELEASE_MASK
                        | GDK_ENTER_NOTIFY_MASK
                        | GDK_LEAVE_NOTIFY_MASK,
                );
                connect_event(
                    window,
                    c"button-press-event",
                    window_on_mouse_down as *const c_void,
                    data,
                );
                connect_event(
                    window,
                    c"button-release-event",
                    window_on_mouse_up as *const c_void,
                    data,
                );
                connect_event(
                    window,
                    c"motion-notify-event",
                    window_on_mouse_move as *const c_void,
                    data,
                );
                connect_event(
                    window,
                    c"enter-notify-event",
                    window_on_mouse_enter as *const c_void,
                    data,
                );
                connect_event(
                    window,
                    c"leave-notify-event",
                    window_on_mouse_leave as *const c_void,
                    data,
                );
            }
            if events.contains(WindowEvents::WHEEL) && !self.0.events.contains(WindowEvents::WHEEL)
            {
                gtk_widget_add_events(
                    window as *mut GtkWidget,
                    GDK_SCROLL_MASK | GDK_SMOOTH_SCROLL_MASK,
                );
                connect_event(
                    window,
                    c"scroll-event",
                    window_on_wheel as *const c_void,
                    data,
                );
            }
            if events.contains(WindowEvents::KEYBOARD)
                && !self.0.events.contains(WindowEvents::KEYBOARD)
            {
                connect_event(
                    window,
                    c"key-press-event",
                    window_on_key_down as *const c_void,
                    data,
                );
                connect_event(
                    window,
                    c"key-release-event",
                    window_on_key_up as *const c_void,
                    data,
                );
            }
        }
        self.0.events |= events;
        enable_webview_events(self.0.as_mut());
    }

    #[cfg(feature = "progress_bar")]
    fn gtk_set_progress_bar(&mut self, progress: Option<f32>) {
        update_progress_bar(progress);
    }
}

pub(super) fn register_webview(window: *mut WindowData, webview: *mut GtkWidget) {
    let window = unsafe { &mut *window };
    window.webview = webview;
    unsafe {
        connect_widget_event(
            webview,
            c"realize",
            window_on_realize as *const c_void,
            window as *mut WindowData as *const c_void,
        );
    }
    enable_webview_events(window);
}

fn enable_webview_events(window: &mut WindowData) {
    if window.webview.is_null() {
        return;
    }
    let data = window as *mut WindowData as *const c_void;
    let widget = window.webview;
    unsafe {
        if window.events.contains(WindowEvents::MOUSE)
            && !window.webview_events.contains(WindowEvents::MOUSE)
        {
            gtk_widget_add_events(
                widget,
                GDK_POINTER_MOTION_MASK
                    | GDK_BUTTON_PRESS_MASK
                    | GDK_BUTTON_RELEASE_MASK
                    | GDK_ENTER_NOTIFY_MASK
                    | GDK_LEAVE_NOTIFY_MASK,
            );
            connect_widget_event(
                widget,
                c"button-press-event",
                window_on_mouse_down as *const c_void,
                data,
            );
            connect_widget_event(
                widget,
                c"button-release-event",
                window_on_mouse_up as *const c_void,
                data,
            );
            connect_widget_event(
                widget,
                c"motion-notify-event",
                window_on_mouse_move as *const c_void,
                data,
            );
            connect_widget_event(
                widget,
                c"enter-notify-event",
                window_on_mouse_enter as *const c_void,
                data,
            );
            connect_widget_event(
                widget,
                c"leave-notify-event",
                window_on_mouse_leave as *const c_void,
                data,
            );
        }
        if window.events.contains(WindowEvents::WHEEL)
            && !window.webview_events.contains(WindowEvents::WHEEL)
        {
            gtk_widget_add_events(widget, GDK_SCROLL_MASK | GDK_SMOOTH_SCROLL_MASK);
            connect_widget_event(
                widget,
                c"scroll-event",
                window_on_wheel as *const c_void,
                data,
            );
        }
        if window.events.contains(WindowEvents::KEYBOARD)
            && !window.webview_events.contains(WindowEvents::KEYBOARD)
        {
            connect_widget_event(
                widget,
                c"key-press-event",
                window_on_key_down as *const c_void,
                data,
            );
            connect_widget_event(
                widget,
                c"key-release-event",
                window_on_key_up as *const c_void,
                data,
            );
        }
    }
    window.webview_events |= window.events;
}

fn apply_cursor(window: &WindowData) {
    let name = match window.cursor {
        Cursor::Default => c"default",
        Cursor::Pointer => c"pointer",
        Cursor::Crosshair => c"crosshair",
        Cursor::Text => c"text",
        Cursor::Grab => c"grab",
        Cursor::Grabbing => c"grabbing",
    };
    for widget in [window.window as *mut GtkWidget, window.webview] {
        if widget.is_null() {
            continue;
        }
        unsafe {
            let gdk_window = gtk_widget_get_window(widget);
            if gdk_window.is_null() {
                continue;
            }
            let cursor = gdk_cursor_new_from_name(gdk_display_get_default(), name.as_ptr());
            gdk_window_set_cursor(gdk_window, cursor);
            if !cursor.is_null() {
                g_object_unref(cursor as *mut GObject);
            }
        }
    }
}

unsafe fn connect_event(
    window: *mut GtkWindow,
    signal: &CStr,
    callback: *const c_void,
    data: *const c_void,
) {
    unsafe {
        g_signal_connect_data(
            window as *mut GObject,
            signal.as_ptr(),
            callback,
            data,
            null(),
            G_CONNECT_DEFAULT,
        );
    }
}

unsafe fn connect_widget_event(
    widget: *mut GtkWidget,
    signal: &CStr,
    callback: *const c_void,
    data: *const c_void,
) {
    unsafe {
        g_signal_connect_data(
            widget as *mut GObject,
            signal.as_ptr(),
            callback,
            data,
            null(),
            G_CONNECT_DEFAULT,
        );
    }
}

extern "C" fn window_on_realize(_widget: *mut GtkWidget, data: *mut c_void) {
    apply_cursor(unsafe { &*data.cast::<WindowData>() });
}

fn modifiers(state: u32) -> (bool, bool, bool, bool) {
    (
        state & (1 << 3) != 0,
        state & (1 << 2) != 0,
        state & (1 << 26) != 0,
        state & 1 != 0,
    )
}

fn buttons(state: u32) -> u16 {
    ((state >> 8) & 1) as u16 | (((state >> 10) & 1) as u16) << 1 | (((state >> 9) & 1) as u16) << 2
}

fn mouse_event(
    window: &mut WindowData,
    x: f64,
    y: f64,
    root_x: f64,
    root_y: f64,
    state: u32,
    button: i16,
    detail: u16,
) -> MouseEvent {
    let point = LogicalPoint::new(x as f32, y as f32);
    let movement = window
        .last_mouse
        .map_or(LogicalPoint::new(0.0, 0.0), |last| {
            LogicalPoint::new(point.x - last.x, point.y - last.y)
        });
    window.last_mouse = Some(point);
    let (alt_key, ctrl_key, meta_key, shift_key) = modifiers(state);
    MouseEvent {
        client_x: point.x,
        client_y: point.y,
        screen_x: root_x as f32,
        screen_y: root_y as f32,
        movement_x: movement.x,
        movement_y: movement.y,
        button,
        buttons: buttons(state),
        detail,
        alt_key,
        ctrl_key,
        meta_key,
        shift_key,
    }
}

fn button_event(window: &mut WindowData, event: &GdkEventButton, pressed: bool) -> MouseEvent {
    let button = match event.button {
        1 => 0,
        2 => 1,
        3 => 2,
        button => button as i16,
    };
    let mut event = mouse_event(
        window,
        event.x,
        event.y,
        event.x_root,
        event.y_root,
        event.state,
        button,
        1,
    );
    if button < 16 {
        let mask = 1 << button;
        if pressed {
            event.buttons |= mask;
        } else {
            event.buttons &= !mask;
        }
    }
    event
}

fn event_targets_widget(
    window: &WindowData,
    widget: *mut GtkWidget,
    event_window: *mut GdkWindow,
) -> bool {
    widget == window.webview || unsafe { gtk_widget_get_window(widget) == event_window }
}

extern "C" fn window_on_focus(_: *mut GtkWidget, _: *mut c_void, _: *mut c_void) -> bool {
    send_event(crate::Event::Window(WindowEvent::Focus));
    false
}

extern "C" fn window_on_blur(_: *mut GtkWidget, _: *mut c_void, data: *mut c_void) -> bool {
    let window = unsafe { &mut *data.cast::<WindowData>() };
    window.primary_button_down = false;
    window.pressed_keys.clear();
    send_event(crate::Event::Window(WindowEvent::Blur));
    false
}

extern "C" fn window_on_mouse_down(
    widget: *mut GtkWidget,
    event: &GdkEventButton,
    data: *mut c_void,
) -> bool {
    let window = unsafe { &mut *data.cast::<WindowData>() };
    if !event_targets_widget(window, widget, event.window) {
        return false;
    }
    let event = button_event(window, event, true);
    if event.button == 0 {
        window.primary_button_down = true;
    }
    send_event(crate::Event::Window(WindowEvent::MouseDown(event)));
    false
}

extern "C" fn window_on_mouse_up(
    widget: *mut GtkWidget,
    event: &GdkEventButton,
    data: *mut c_void,
) -> bool {
    let window = unsafe { &mut *data.cast::<WindowData>() };
    if !event_targets_widget(window, widget, event.window) {
        return false;
    }
    let event = button_event(window, event, false);
    send_event(crate::Event::Window(WindowEvent::MouseUp(event.clone())));
    if event.button == 0 && std::mem::take(&mut window.primary_button_down) {
        send_event(crate::Event::Window(WindowEvent::Click(event)));
    }
    false
}

extern "C" fn window_on_mouse_move(
    widget: *mut GtkWidget,
    event: &GdkEventMotion,
    data: *mut c_void,
) -> bool {
    let window = unsafe { &mut *data.cast::<WindowData>() };
    if !event_targets_widget(window, widget, event.window) {
        return false;
    }
    let event = mouse_event(
        window,
        event.x,
        event.y,
        event.x_root,
        event.y_root,
        event.state,
        -1,
        0,
    );
    send_event(crate::Event::Window(WindowEvent::MouseMove(event)));
    false
}

fn crossing_event(window: &mut WindowData, event: &GdkEventCrossing) -> MouseEvent {
    mouse_event(
        window,
        event.x,
        event.y,
        event.x_root,
        event.y_root,
        event.state,
        -1,
        0,
    )
}

extern "C" fn window_on_mouse_enter(
    widget: *mut GtkWidget,
    event: &GdkEventCrossing,
    data: *mut c_void,
) -> bool {
    let window = unsafe { &mut *data.cast::<WindowData>() };
    if !event_targets_widget(window, widget, event.window) {
        return false;
    }
    let event = crossing_event(window, event);
    send_event(crate::Event::Window(WindowEvent::MouseEnter(event)));
    false
}

extern "C" fn window_on_mouse_leave(
    widget: *mut GtkWidget,
    event: &GdkEventCrossing,
    data: *mut c_void,
) -> bool {
    let window = unsafe { &mut *data.cast::<WindowData>() };
    if !event_targets_widget(window, widget, event.window) {
        return false;
    }
    let event = crossing_event(window, event);
    window.last_mouse = None;
    send_event(crate::Event::Window(WindowEvent::MouseLeave(event)));
    false
}

extern "C" fn window_on_wheel(
    widget: *mut GtkWidget,
    event: &GdkEventScroll,
    data: *mut c_void,
) -> bool {
    let window = unsafe { &mut *data.cast::<WindowData>() };
    if !event_targets_widget(window, widget, event.window) {
        return false;
    }
    let mouse = mouse_event(
        window,
        event.x,
        event.y,
        event.x_root,
        event.y_root,
        event.state,
        -1,
        0,
    );
    let (delta_x, delta_y) = match event.direction {
        0 => (0.0, -1.0),
        1 => (0.0, 1.0),
        2 => (-1.0, 0.0),
        3 => (1.0, 0.0),
        _ => (event.delta_x as f32, event.delta_y as f32),
    };
    send_event(crate::Event::Window(WindowEvent::Wheel(WheelEvent {
        mouse,
        delta_x,
        delta_y,
        delta_z: 0.0,
        delta_mode: 1,
    })));
    false
}

fn key_event(event: &GdkEventKey, repeat: bool) -> KeyboardEvent {
    let (alt_key, ctrl_key, meta_key, shift_key) = modifiers(event.state);
    let unicode = unsafe { gdk_keyval_to_unicode(event.keyval) };
    let name = unsafe {
        let name = gdk_keyval_name(event.keyval);
        if name.is_null() {
            "Unidentified".into()
        } else {
            CStr::from_ptr(name).to_string_lossy().into_owned()
        }
    };
    let key = char::from_u32(unicode).map_or_else(
        || match name.as_str() {
            "Return" => "Enter".into(),
            "Shift_L" | "Shift_R" => "Shift".into(),
            "Control_L" | "Control_R" => "Control".into(),
            "Alt_L" | "Alt_R" => "Alt".into(),
            "Super_L" | "Super_R" => "Meta".into(),
            "Left" => "ArrowLeft".into(),
            "Right" => "ArrowRight".into(),
            "Up" => "ArrowUp".into(),
            "Down" => "ArrowDown".into(),
            value => value.into(),
        },
        |value| value.to_string(),
    );
    let physical_code = gtk_physical_code(event.hardware_keycode);
    let code = if physical_code != "Unidentified" {
        physical_code.into()
    } else {
        match name.as_str() {
            "Return" | "KP_Enter" => "Enter".into(),
            "BackSpace" => "Backspace".into(),
            "space" => "Space".into(),
            "Shift_L" => "ShiftLeft".into(),
            "Shift_R" => "ShiftRight".into(),
            "Control_L" => "ControlLeft".into(),
            "Control_R" => "ControlRight".into(),
            "Alt_L" => "AltLeft".into(),
            "Alt_R" => "AltRight".into(),
            "Super_L" => "MetaLeft".into(),
            "Super_R" => "MetaRight".into(),
            "Left" => "ArrowLeft".into(),
            "Up" => "ArrowUp".into(),
            "Right" => "ArrowRight".into(),
            "Down" => "ArrowDown".into(),
            _ if key.len() == 1 && key.chars().next().is_some_and(char::is_alphabetic) => {
                format!("Key{}", key.to_uppercase())
            }
            _ if key.len() == 1
                && key
                    .chars()
                    .next()
                    .is_some_and(|value| value.is_ascii_digit()) =>
            {
                format!("Digit{key}")
            }
            _ => name.clone(),
        }
    };
    let location = if name.starts_with("KP_") {
        3
    } else if name.ends_with("_L") {
        1
    } else if name.ends_with("_R") {
        2
    } else {
        0
    };
    KeyboardEvent {
        key,
        code,
        location,
        repeat,
        is_composing: false,
        alt_key,
        ctrl_key,
        meta_key,
        shift_key,
    }
}

const fn gtk_physical_code(hardware_keycode: u16) -> &'static str {
    match hardware_keycode.saturating_sub(8) {
        1 => "Escape",
        2 => "Digit1",
        3 => "Digit2",
        4 => "Digit3",
        5 => "Digit4",
        6 => "Digit5",
        7 => "Digit6",
        8 => "Digit7",
        9 => "Digit8",
        10 => "Digit9",
        11 => "Digit0",
        12 => "Minus",
        13 => "Equal",
        14 => "Backspace",
        15 => "Tab",
        16 => "KeyQ",
        17 => "KeyW",
        18 => "KeyE",
        19 => "KeyR",
        20 => "KeyT",
        21 => "KeyY",
        22 => "KeyU",
        23 => "KeyI",
        24 => "KeyO",
        25 => "KeyP",
        26 => "BracketLeft",
        27 => "BracketRight",
        28 => "Enter",
        29 => "ControlLeft",
        30 => "KeyA",
        31 => "KeyS",
        32 => "KeyD",
        33 => "KeyF",
        34 => "KeyG",
        35 => "KeyH",
        36 => "KeyJ",
        37 => "KeyK",
        38 => "KeyL",
        39 => "Semicolon",
        40 => "Quote",
        41 => "Backquote",
        42 => "ShiftLeft",
        43 => "Backslash",
        44 => "KeyZ",
        45 => "KeyX",
        46 => "KeyC",
        47 => "KeyV",
        48 => "KeyB",
        49 => "KeyN",
        50 => "KeyM",
        51 => "Comma",
        52 => "Period",
        53 => "Slash",
        54 => "ShiftRight",
        56 => "AltLeft",
        57 => "Space",
        97 => "ControlRight",
        100 => "AltRight",
        125 => "MetaLeft",
        126 => "MetaRight",
        _ => "Unidentified",
    }
}

extern "C" fn window_on_key_down(
    widget: *mut GtkWidget,
    event: &GdkEventKey,
    data: *mut c_void,
) -> bool {
    let window = unsafe { &mut *data.cast::<WindowData>() };
    if !event_targets_widget(window, widget, event.window) {
        return false;
    }
    let repeat = !window.pressed_keys.insert(event.hardware_keycode);
    send_event(crate::Event::Window(WindowEvent::KeyDown(key_event(
        event, repeat,
    ))));
    false
}

extern "C" fn window_on_key_up(
    widget: *mut GtkWidget,
    event: &GdkEventKey,
    data: *mut c_void,
) -> bool {
    let window = unsafe { &mut *data.cast::<WindowData>() };
    if !event_targets_widget(window, widget, event.window) {
        return false;
    }
    window.pressed_keys.remove(&event.hardware_keycode);
    send_event(crate::Event::Window(WindowEvent::KeyUp(key_event(
        event, false,
    ))));
    false
}

extern "C" fn window_on_theme_change(_widget: *mut GtkWidget, data: *mut c_void) {
    let window = unsafe { &mut *data.cast::<WindowData>() };
    if !window.follows_system_theme {
        return;
    }
    let theme = system_theme();
    if theme != window.theme {
        window.theme = theme;
        send_theme_change(theme);
    }
}

extern "C" fn window_on_move(
    _window: *mut GtkWindow,
    _allocation: *mut c_void,
    _self: &mut WindowData,
) -> bool {
    let mut x = 0;
    let mut y = 0;
    unsafe { gtk_window_get_position(_self.window, &mut x, &mut y) };
    send_event(crate::Event::Window(WindowEvent::Move(LogicalPoint::new(
        x as f32, y as f32,
    ))));
    false
}

extern "C" fn window_on_resize(
    _window: *mut GtkWindow,
    _allocation: *mut c_void,
    _self: &mut WindowData,
) {
    let mut width = 0;
    let mut height = 0;
    unsafe { gtk_window_get_size(_self.window, &mut width, &mut height) };
    send_event(crate::Event::Window(WindowEvent::Resize(LogicalSize::new(
        width as f32,
        height as f32,
    ))));
}

extern "C" fn window_on_close(
    _window: *mut GtkWindow,
    _event: *mut c_void,
    _self: &mut WindowData,
) -> bool {
    let request = CloseRequest::new();
    send_event(crate::Event::Window(WindowEvent::CloseRequested(
        request.clone(),
    )));
    if request.is_prevented() {
        return true;
    }
    #[cfg(feature = "remember_window_state")]
    if _self.remember_window_state {
        save_window_state(_self.window);
    }
    false
}
