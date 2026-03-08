/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::env;
use std::ffi::{CStr, CString, c_void};
use std::fs;
use std::mem::MaybeUninit;
use std::ptr::{null, null_mut};

use super::event_loop::APP_ID;
use super::headers::*;
use crate::{Key, LogicalPoint, LogicalSize, Modifiers, MouseButton, Theme, WindowBuilder, WindowHandler, WindowId};

pub(super) struct WindowData {
    pub(super) window_id: WindowId,
    pub(super) window: *mut GtkWindow,
    #[cfg(feature = "webview")]
    pub(super) webview: *mut WebKitWebView,
    pub(super) background_color: Option<u32>,
    pub(super) remember_window_state: bool,
    pub(super) window_handler: Option<*mut dyn WindowHandler>,
    #[cfg(feature = "webview")]
    pub(super) webview_handler: Option<*mut dyn crate::WebviewHandler>,
}

pub(crate) struct PlatformWindow(pub(super) Box<WindowData>);

impl PlatformWindow {
    pub(crate) fn new(window_id: WindowId, builder: &WindowBuilder) -> Self {
        let is_wayland = unsafe {
            CStr::from_ptr(gdk_display_get_name(gdk_display_get_default()))
                .to_string_lossy()
                .contains("wayland")
        };

        // Force dark mode if enabled
        if let Some(theme) = builder.theme {
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

        let mut window_data = Box::new(WindowData {
            window_id,
            window: null_mut(),
            #[cfg(feature = "webview")]
            webview: null_mut(),
            background_color: builder.background_color,
            remember_window_state: builder.remember_window_state,
            window_handler: builder.window_handler,
            #[cfg(feature = "webview")]
            webview_handler: None,
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
                let mut rect = MaybeUninit::<GdkRectangle>::uninit();
                gdk_monitor_get_geometry(monitor.monitor, rect.as_mut_ptr());
                rect.assume_init()
            } else {
                let mut primary_monitor =
                    gdk_display_get_primary_monitor(gdk_display_get_default());
                if primary_monitor.is_null() {
                    primary_monitor = gdk_display_get_monitor(gdk_display_get_default(), 0);
                }
                let mut rect = MaybeUninit::<GdkRectangle>::uninit();
                gdk_monitor_get_geometry(primary_monitor, rect.as_mut_ptr());
                rect.assume_init()
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
            if builder.remember_window_state {
                Self::load_window_state(window);
            }

            g_signal_connect_data(
                window as *mut GObject,
                c"focus-in-event".as_ptr(),
                window_on_focus as *const c_void,
                window_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window as *mut GObject,
                c"focus-out-event".as_ptr(),
                window_on_blur as *const c_void,
                window_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window as *mut GObject,
                c"key-press-event".as_ptr(),
                window_on_key_press as *const c_void,
                window_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window as *mut GObject,
                c"key-release-event".as_ptr(),
                window_on_key_release as *const c_void,
                window_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window as *mut GObject,
                c"motion-notify-event".as_ptr(),
                window_on_mouse_move as *const c_void,
                window_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window as *mut GObject,
                c"button-press-event".as_ptr(),
                window_on_button_press as *const c_void,
                window_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window as *mut GObject,
                c"button-release-event".as_ptr(),
                window_on_button_release as *const c_void,
                window_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window as *mut GObject,
                c"scroll-event".as_ptr(),
                window_on_scroll as *const c_void,
                window_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            gtk_widget_add_events(
                window as *mut GtkWidget,
                GDK_BUTTON_PRESS_MASK | GDK_BUTTON_RELEASE_MASK | GDK_POINTER_MOTION_MASK | GDK_SCROLL_MASK,
            );
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

        window_data.window = window;
        PlatformWindow(window_data)
    }

    fn load_window_state(window: *mut GtkWindow) {
        unsafe {
            let settings = g_key_file_new();
            let file = CString::new(config_dir().join("settings.ini").display().to_string())
                .expect("Can't convert to CString");
            let mut err = null_mut();
            g_key_file_load_from_file(settings, file.as_ptr(), 0, &mut err);
            if err.is_null() {
                let group = c"window".as_ptr();
                let x = g_key_file_get_integer(settings, group, c"x".as_ptr(), null_mut());
                let y = g_key_file_get_integer(settings, group, c"y".as_ptr(), null_mut());
                gtk_window_move(window, x, y);

                let width = g_key_file_get_integer(settings, group, c"width".as_ptr(), null_mut());
                let height =
                    g_key_file_get_integer(settings, group, c"height".as_ptr(), null_mut());
                gtk_window_set_default_size(window, width, height);

                let maximized =
                    g_key_file_get_boolean(settings, group, c"maximized".as_ptr(), null_mut());
                if maximized {
                    gtk_window_maximize(window);
                }
            } else {
                g_error_free(err);
            }
            g_key_file_free(settings);
        }
    }

    fn save_window_state(window: *mut GtkWindow) {
        fs::create_dir_all(config_dir()).expect("Can't create settings directory");
        let settings_path = config_dir().join("settings.ini");
        unsafe {
            let settings = g_key_file_new();
            let group = c"window".as_ptr();

            let mut x = 0;
            let mut y = 0;
            gtk_window_get_position(window, &mut x, &mut y);
            g_key_file_set_integer(settings, group, c"x".as_ptr(), x);
            g_key_file_set_integer(settings, group, c"y".as_ptr(), y);

            let mut width = 0;
            let mut height = 0;
            gtk_window_get_size(window, &mut width, &mut height);
            g_key_file_set_integer(settings, group, c"width".as_ptr(), width);
            g_key_file_set_integer(settings, group, c"height".as_ptr(), height);

            let maximized = gtk_window_is_maximized(window);
            g_key_file_set_boolean(settings, group, c"maximized".as_ptr(), maximized);

            let file = CString::new(settings_path.display().to_string())
                .expect("Can't convert to CString");
            g_key_file_save_to_file(settings, file.as_ptr(), null_mut());
            g_key_file_free(settings);
        }
    }
}

impl crate::WindowInterface for PlatformWindow {
    fn set_title(&mut self, title: impl AsRef<str>) {
        let title = CString::new(title.as_ref()).expect("Can't convert to CString");
        unsafe { gtk_window_set_title(self.0.window, title.as_ptr()) }
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
        let primary_monitor_rect = unsafe {
            let mut primary_monitor = gdk_display_get_primary_monitor(gdk_display_get_default());
            if primary_monitor.is_null() {
                primary_monitor = gdk_display_get_monitor(gdk_display_get_default(), 0);
            }
            let mut primary_monitor_rect = MaybeUninit::<GdkRectangle>::uninit();
            gdk_monitor_get_geometry(primary_monitor, primary_monitor_rect.as_mut_ptr());
            primary_monitor_rect.assume_init()
        };
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
            #[cfg(feature = "webview")]
            if !self.0.webview.is_null() {
                webkit_web_view_set_background_color(self.0.webview, &rgba);
            }
        }
    }
}

extern "C" fn window_on_move(
    _window: *mut GtkWindow,
    _allocation: *mut c_void,
    _self: &mut WindowData,
) -> bool {
    if let Some(h_ptr) = _self.window_handler {
        let mut x = 0;
        let mut y = 0;
        unsafe { gtk_window_get_position(_self.window, &mut x, &mut y) };
        unsafe {
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(_self);
            handler.on_move(&mut window, x, y);
            std::mem::forget(window.platform.0);
        }
    }
    false
}

extern "C" fn window_on_resize(
    _window: *mut GtkWindow,
    _allocation: *mut c_void,
    _self: &mut WindowData,
) {
    if let Some(h_ptr) = _self.window_handler {
        let mut width = 0;
        let mut height = 0;
        unsafe { gtk_window_get_size(_self.window, &mut width, &mut height) };
        unsafe {
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(_self);
            handler.on_resize(&mut window, width as u32, height as u32);
            std::mem::forget(window.platform.0);
        }
    }
}

extern "C" fn window_on_close(
    _window: *mut GtkWindow,
    _event: *mut c_void,
    _self: &mut WindowData,
) -> bool {
    // Save window state
    if _self.remember_window_state {
        PlatformWindow::save_window_state(_self.window);
    }

    let allow = if let Some(h_ptr) = _self.window_handler {
        unsafe {
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(_self);
            let result = handler.on_close(&mut window);
            std::mem::forget(window.platform.0);
            result
        }
    } else {
        true
    };
    // Return TRUE to prevent close, FALSE to allow
    !allow
}

extern "C" fn window_on_focus(
    _window: *mut GtkWindow,
    _event: *mut c_void,
    _self: &mut WindowData,
) -> bool {
    if let Some(h_ptr) = _self.window_handler {
        unsafe {
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(_self);
            handler.on_focus(&mut window);
            std::mem::forget(window.platform.0);
        }
    }
    false
}

extern "C" fn window_on_blur(
    _window: *mut GtkWindow,
    _event: *mut c_void,
    _self: &mut WindowData,
) -> bool {
    if let Some(h_ptr) = _self.window_handler {
        unsafe {
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(_self);
            handler.on_blur(&mut window);
            std::mem::forget(window.platform.0);
        }
    }
    false
}

extern "C" fn window_on_key_press(
    _window: *mut GtkWindow,
    event: *mut GdkEventKey,
    _self: &mut WindowData,
) -> bool {
    if let Some(h_ptr) = _self.window_handler {
        unsafe {
            let key = gdk_keyval_to_key((*event).keyval);
            let mods = gdk_state_to_modifiers((*event).state);
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(_self);
            handler.on_key_down(&mut window, key, mods);
            std::mem::forget(window.platform.0);
        }
    }
    false
}

extern "C" fn window_on_key_release(
    _window: *mut GtkWindow,
    event: *mut GdkEventKey,
    _self: &mut WindowData,
) -> bool {
    if let Some(h_ptr) = _self.window_handler {
        unsafe {
            let key = gdk_keyval_to_key((*event).keyval);
            let mods = gdk_state_to_modifiers((*event).state);
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(_self);
            handler.on_key_up(&mut window, key, mods);
            std::mem::forget(window.platform.0);
        }
    }
    false
}

extern "C" fn window_on_mouse_move(
    _window: *mut GtkWindow,
    event: *mut GdkEventMotion,
    _self: &mut WindowData,
) -> bool {
    if let Some(h_ptr) = _self.window_handler {
        unsafe {
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(_self);
            handler.on_mouse_move(&mut window, (*event).x, (*event).y);
            std::mem::forget(window.platform.0);
        }
    }
    false
}

extern "C" fn window_on_button_press(
    _window: *mut GtkWindow,
    event: *mut GdkEventButton,
    _self: &mut WindowData,
) -> bool {
    if let Some(h_ptr) = _self.window_handler {
        unsafe {
            let button = gdk_button_to_mouse_button((*event).button);
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(_self);
            handler.on_mouse_down(&mut window, button, (*event).x, (*event).y);
            std::mem::forget(window.platform.0);
        }
    }
    false
}

extern "C" fn window_on_button_release(
    _window: *mut GtkWindow,
    event: *mut GdkEventButton,
    _self: &mut WindowData,
) -> bool {
    if let Some(h_ptr) = _self.window_handler {
        unsafe {
            let button = gdk_button_to_mouse_button((*event).button);
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(_self);
            handler.on_mouse_up(&mut window, button, (*event).x, (*event).y);
            std::mem::forget(window.platform.0);
        }
    }
    false
}

extern "C" fn window_on_scroll(
    _window: *mut GtkWindow,
    event: *mut GdkEventScroll,
    _self: &mut WindowData,
) -> bool {
    if let Some(h_ptr) = _self.window_handler {
        unsafe {
            let (dx, dy) = match (*event).direction {
                GDK_SCROLL_UP => (0.0, -3.0),
                GDK_SCROLL_DOWN => (0.0, 3.0),
                GDK_SCROLL_LEFT => (-3.0, 0.0),
                GDK_SCROLL_RIGHT => (3.0, 0.0),
                GDK_SCROLL_SMOOTH => ((*event).delta_x, (*event).delta_y),
                _ => (0.0, 0.0),
            };
            let handler = &mut *h_ptr;
            let mut window = make_temp_window(_self);
            handler.on_wheel(&mut window, dx, dy);
            std::mem::forget(window.platform.0);
        }
    }
    false
}

// --- GDK keysym to Key mapping ---
fn gdk_keyval_to_key(keyval: u32) -> Key {
    match keyval {
        0x061 => Key::A, 0x062 => Key::B, 0x063 => Key::C, 0x064 => Key::D,
        0x065 => Key::E, 0x066 => Key::F, 0x067 => Key::G, 0x068 => Key::H,
        0x069 => Key::I, 0x06A => Key::J, 0x06B => Key::K, 0x06C => Key::L,
        0x06D => Key::M, 0x06E => Key::N, 0x06F => Key::O, 0x070 => Key::P,
        0x071 => Key::Q, 0x072 => Key::R, 0x073 => Key::S, 0x074 => Key::T,
        0x075 => Key::U, 0x076 => Key::V, 0x077 => Key::W, 0x078 => Key::X,
        0x079 => Key::Y, 0x07A => Key::Z,
        0x041 => Key::A, 0x042 => Key::B, 0x043 => Key::C, 0x044 => Key::D,
        0x045 => Key::E, 0x046 => Key::F, 0x047 => Key::G, 0x048 => Key::H,
        0x049 => Key::I, 0x04A => Key::J, 0x04B => Key::K, 0x04C => Key::L,
        0x04D => Key::M, 0x04E => Key::N, 0x04F => Key::O, 0x050 => Key::P,
        0x051 => Key::Q, 0x052 => Key::R, 0x053 => Key::S, 0x054 => Key::T,
        0x055 => Key::U, 0x056 => Key::V, 0x057 => Key::W, 0x058 => Key::X,
        0x059 => Key::Y, 0x05A => Key::Z,
        0x030 => Key::Digit0, 0x031 => Key::Digit1, 0x032 => Key::Digit2,
        0x033 => Key::Digit3, 0x034 => Key::Digit4, 0x035 => Key::Digit5,
        0x036 => Key::Digit6, 0x037 => Key::Digit7, 0x038 => Key::Digit8, 0x039 => Key::Digit9,
        0xFFBE => Key::F1, 0xFFBF => Key::F2, 0xFFC0 => Key::F3, 0xFFC1 => Key::F4,
        0xFFC2 => Key::F5, 0xFFC3 => Key::F6, 0xFFC4 => Key::F7, 0xFFC5 => Key::F8,
        0xFFC6 => Key::F9, 0xFFC7 => Key::F10, 0xFFC8 => Key::F11, 0xFFC9 => Key::F12,
        0xFF1B => Key::Escape, 0xFF0D => Key::Enter, 0xFF08 => Key::Backspace,
        0xFF09 => Key::Tab, 0x020 => Key::Space, 0xFFFF => Key::Delete, 0xFF63 => Key::Insert,
        0xFF52 => Key::ArrowUp, 0xFF54 => Key::ArrowDown,
        0xFF51 => Key::ArrowLeft, 0xFF53 => Key::ArrowRight,
        0xFF50 => Key::Home, 0xFF57 => Key::End,
        0xFF55 => Key::PageUp, 0xFF56 => Key::PageDown,
        0xFFE1 | 0xFFE2 => Key::Shift,
        0xFFE3 | 0xFFE4 => Key::Control,
        0xFFE9 | 0xFFEA => Key::Alt,
        0xFFEB | 0xFFEC => Key::Meta,
        0xFFE5 => Key::CapsLock,
        0xFFB0 => Key::Numpad0, 0xFFB1 => Key::Numpad1, 0xFFB2 => Key::Numpad2,
        0xFFB3 => Key::Numpad3, 0xFFB4 => Key::Numpad4, 0xFFB5 => Key::Numpad5,
        0xFFB6 => Key::Numpad6, 0xFFB7 => Key::Numpad7, 0xFFB8 => Key::Numpad8, 0xFFB9 => Key::Numpad9,
        0xFFAB => Key::NumpadAdd, 0xFFAD => Key::NumpadSubtract,
        0xFFAA => Key::NumpadMultiply, 0xFFAF => Key::NumpadDivide,
        0xFF8D => Key::NumpadEnter, 0xFFAE => Key::NumpadDecimal,
        0x02D => Key::Minus, 0x03D => Key::Equal, 0x05B => Key::BracketLeft,
        0x05D => Key::BracketRight, 0x05C => Key::Backslash, 0x03B => Key::Semicolon,
        0x027 => Key::Quote, 0x02C => Key::Comma, 0x02E => Key::Period, 0x02F => Key::Slash,
        0x060 => Key::Backtick,
        _ => Key::Unknown,
    }
}

// GDK modifier masks
const GDK_SHIFT_MASK: u32 = 1 << 0;
const GDK_CONTROL_MASK: u32 = 1 << 2;
const GDK_MOD1_MASK: u32 = 1 << 3;  // Alt
const GDK_SUPER_MASK: u32 = 1 << 26; // Meta/Super

fn gdk_state_to_modifiers(state: u32) -> Modifiers {
    let mut mods = Modifiers::empty();
    if state & GDK_SHIFT_MASK != 0 { mods = mods | Modifiers::SHIFT; }
    if state & GDK_CONTROL_MASK != 0 { mods = mods | Modifiers::CTRL; }
    if state & GDK_MOD1_MASK != 0 { mods = mods | Modifiers::ALT; }
    if state & GDK_SUPER_MASK != 0 { mods = mods | Modifiers::META; }
    mods
}

fn gdk_button_to_mouse_button(button: u32) -> MouseButton {
    match button {
        1 => MouseButton::Left,
        2 => MouseButton::Middle,
        3 => MouseButton::Right,
        8 => MouseButton::Back,
        9 => MouseButton::Forward,
        n => MouseButton::Other(n as u8),
    }
}

unsafe fn make_temp_window(data: &mut WindowData) -> std::mem::ManuallyDrop<crate::Window> {
    std::mem::ManuallyDrop::new(crate::Window {
        id: data.window_id,
        platform: PlatformWindow(Box::from_raw(data as *mut WindowData)),
        window_handler: data.window_handler,
    })
}

pub(super) fn config_dir() -> std::path::PathBuf {
    let project_dirs = unsafe {
        if let Some(ref app_id) = APP_ID {
            directories::ProjectDirs::from(
                &app_id.qualifier,
                &app_id.organization,
                &app_id.application,
            )
        } else {
            directories::ProjectDirs::from_path(std::path::PathBuf::from(
                env::current_exe()
                    .expect("Can't get current process name")
                    .file_name()
                    .expect("Can't get current process name")
                    .to_string_lossy()
                    .into_owned(),
            ))
        }
    }
    .expect("Can't get dirs");
    project_dirs.config_dir()
}
