/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::{CStr, CString, c_void};
use std::ptr::{null, null_mut};

use super::event_loop::{primary_monitor_rect, send_event};
use super::headers::*;
use super::input::gtk_input_event;
#[cfg(feature = "progress_bar")]
use super::progress_bar::update_progress_bar;
#[cfg(feature = "remember_window_state")]
use super::window_state::{load_window_state, save_window_state};
use crate::{
    CloseRequest, LogicalPoint, LogicalSize, PointerLockError, Theme, WindowBuilder, WindowEvent,
};

pub(super) struct WindowData {
    host: std::rc::Rc<crate::content::ContentHost>,
    pub(crate) window_id: crate::WindowId,
    pub(super) window: *mut GtkWindow,
    pub(super) background_color: Option<u32>,
    keys: RefCell<HashSet<u16>>,
    #[cfg(feature = "remember_window_state")]
    pub(super) remember_window_state: bool,
    #[cfg(feature = "file_drop")]
    pub(super) allow_file_drop: bool,
}

thread_local! {
    static WINDOW_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

extern "C" fn window_on_destroy(_: *mut GtkWidget, data: &WindowData) {
    data.host.close();
    WINDOW_COUNT.with(|count| {
        count.set(count.get() - 1);
        if count.get() == 0 {
            super::event_loop::stop();
        }
    });
}

extern "C" fn window_on_style_updated(_: *mut GtkWidget, data: &WindowData) {
    data.host.theme_changed(super::event_loop::system_theme());
}

pub(crate) struct PlatformWindow(pub(super) Box<WindowData>);

impl PlatformWindow {
    pub(crate) const fn id(&self) -> crate::WindowId {
        self.0.window_id
    }

    pub(crate) fn new(
        builder: &WindowBuilder,
        host: std::rc::Rc<crate::content::ContentHost>,
    ) -> Self {
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
            host: host.clone(),
            window_id: builder.window_id,
            window: null_mut(),
            background_color: builder.background_color,
            keys: RefCell::new(HashSet::new()),
            #[cfg(feature = "remember_window_state")]
            remember_window_state: builder.remember_window_state,
            #[cfg(feature = "file_drop")]
            allow_file_drop: builder.allow_file_drop,
        });

        // Create window
        let window = unsafe {
            let window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
            gtk_widget_set_can_focus(window.cast(), 1);
            // Motion, buttons, enter/leave, keys, focus, discrete and smooth scroll.
            gtk_widget_add_events(
                window.cast(),
                (1 << 2)
                    | (1 << 8)
                    | (1 << 9)
                    | (1 << 10)
                    | (1 << 11)
                    | (1 << 12)
                    | (1 << 13)
                    | (1 << 14)
                    | (1 << 21)
                    | (1 << 23),
            );
            #[cfg(feature = "file_drop")]
            if builder.allow_file_drop {
                super::file_drop::gtk_connect_file_drop(window.cast(), host.event_sender());
            }
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
                window_on_destroy as *const c_void,
                window_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window as *mut GObject,
                c"style-updated".as_ptr(),
                window_on_style_updated as *const c_void,
                window_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            for (name, callback) in [
                (c"event", window_on_input as *const c_void),
                (c"focus-in-event", window_on_focus_in as *const c_void),
                (c"focus-out-event", window_on_focus_out as *const c_void),
            ] {
                g_signal_connect_data(
                    window.cast(),
                    name.as_ptr(),
                    callback,
                    window_data.as_mut() as *mut _ as *const c_void,
                    null(),
                    G_CONNECT_DEFAULT,
                );
            }
            if !is_wayland {
                g_signal_connect_data(
                    window as *mut GObject,
                    c"configure-event".as_ptr(),
                    window_on_move as *const c_void,
                    window_data.as_mut() as *mut _ as *const c_void,
                    null(),
                    G_CONNECT_DEFAULT,
                );
            }
            g_signal_connect_data(
                window as *mut GObject,
                c"size-allocate".as_ptr(),
                window_on_resize as *const c_void,
                window_data.as_mut() as *mut _ as *const c_void,
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
            window
        };

        WINDOW_COUNT.with(|count| count.set(count.get() + 1));
        window_data.window = window;
        host.set_handle(crate::NativeWindowHandle::Gtk(window.cast()));
        unsafe { gtk_widget_show_all(window.cast()) };
        send_event(crate::Event::Window(builder.window_id, WindowEvent::Create));
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

    fn set_min_size(&mut self, min_size: LogicalSize) {
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

    fn set_theme(&mut self, theme: Theme) {
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

    fn request_pointer_lock(&mut self) -> Result<(), PointerLockError> {
        const GDK_SEAT_CAPABILITY_POINTER: u32 = 1;
        const GDK_BLANK_CURSOR: i32 = -2;

        unsafe {
            let display = gdk_display_get_default();
            if display.is_null() {
                return Err(PointerLockError(
                    "Could not access the native pointer device".into(),
                ));
            }
            let seat = gdk_display_get_default_seat(display);
            let window = gtk_widget_get_window(self.0.window.cast());
            if seat.is_null() || window.is_null() {
                return Err(PointerLockError(
                    "Could not access the native pointer device".into(),
                ));
            }

            let cursor = gdk_cursor_new_for_display(display, GDK_BLANK_CURSOR);
            if cursor.is_null() {
                return Err(PointerLockError(
                    "Could not create a hidden pointer cursor".into(),
                ));
            }
            let status = gdk_seat_grab(
                seat,
                window,
                GDK_SEAT_CAPABILITY_POINTER,
                1,
                cursor,
                null(),
                null(),
                null_mut(),
            );
            g_object_unref(cursor.cast());
            if status != 0 {
                return Err(PointerLockError(format!(
                    "GDK pointer lock failed with status {status}"
                )));
            }
        }
        Ok(())
    }

    fn exit_pointer_lock(&mut self) {
        release_pointer_lock();
    }

    #[cfg(feature = "progress_bar")]
    fn gtk_set_progress_bar(&mut self, progress: Option<f32>) {
        update_progress_bar(progress);
    }
}

pub(crate) fn release_pointer_lock() {
    unsafe {
        let display = gdk_display_get_default();
        if display.is_null() {
            return;
        }
        let seat = gdk_display_get_default_seat(display);
        if !seat.is_null() {
            gdk_seat_ungrab(seat);
        }
    }
}

extern "C" fn window_on_input(
    window: *mut GtkWidget,
    event: *const GdkEvent,
    data: &WindowData,
) -> i32 {
    let kind = unsafe { gdk_event_get_event_type(event) };
    let repeat = if kind == 8 || kind == 9 {
        let mut code = 0;
        unsafe { gdk_event_get_keycode(event, &mut code) };
        if kind == 8 {
            !data.keys.borrow_mut().insert(code)
        } else {
            data.keys.borrow_mut().remove(&code);
            false
        }
    } else {
        false
    };
    if kind == 4 {
        unsafe { gtk_widget_grab_focus(window) };
    }
    if let Some(event) = unsafe { gtk_input_event(event, repeat) } {
        data.host.event_sender().send(event);
        return 1;
    }
    0
}

extern "C" fn window_on_focus_in(_: *mut GtkWidget, _: *const GdkEvent, data: &WindowData) -> i32 {
    data.host.event_sender().send(WindowEvent::Focus);
    0
}

extern "C" fn window_on_focus_out(_: *mut GtkWidget, _: *const GdkEvent, data: &WindowData) -> i32 {
    data.keys.borrow_mut().clear();
    data.host.event_sender().send(WindowEvent::Blur);
    0
}

extern "C" fn window_on_move(
    _window: *mut GtkWindow,
    _allocation: *mut c_void,
    _self: &mut WindowData,
) -> bool {
    let mut x = 0;
    let mut y = 0;
    unsafe { gtk_window_get_position(_self.window, &mut x, &mut y) };
    send_event(crate::Event::Window(
        _self.window_id,
        WindowEvent::Move(LogicalPoint::new(x as f32, y as f32)),
    ));
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
    let scale = unsafe { gtk_widget_get_scale_factor(_self.window.cast()) };
    let host = _self.host.clone();
    let id = _self.window_id;
    host.resize((width * scale) as u32, (height * scale) as u32);
    if host.is_closed() {
        return;
    }
    send_event(crate::Event::Window(
        id,
        WindowEvent::Resize(LogicalSize::new(width as f32, height as f32)),
    ));
}

extern "C" fn window_on_close(
    _window: *mut GtkWindow,
    _event: *mut c_void,
    _self: &mut WindowData,
) -> bool {
    let request = CloseRequest::new();
    let window = _self.window;
    let host = _self.host.clone();
    #[cfg(feature = "remember_window_state")]
    let remember = _self.remember_window_state;
    // Keep the GObject alive if this request is queued by a nested native callback.
    unsafe { g_object_ref(window as *mut GObject) };
    crate::dispatch::send_then(
        crate::Event::Window(
            _self.window_id,
            WindowEvent::CloseRequested(request.clone()),
        ),
        move || {
            if !request.default_prevented() && !host.is_closed() {
                host.close();
                #[cfg(feature = "remember_window_state")]
                if remember {
                    save_window_state(window);
                }
                unsafe { gtk_widget_destroy(window as *mut GtkWidget) };
            }
            unsafe { g_object_unref(window as *mut GObject) };
        },
    );
    true
}
