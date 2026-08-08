/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CStr, CString, c_void};
use std::ptr::{null, null_mut};

use super::event_loop::{primary_monitor_rect, send_event};
use super::headers::*;
#[cfg(feature = "progress_bar")]
use super::progress_bar::update_progress_bar;
#[cfg(feature = "remember_window_state")]
use super::window_state::{load_window_state, save_window_state};
use crate::{CloseRequest, LogicalPoint, LogicalSize, Theme, WindowBuilder, WindowEvent};

pub(super) struct WindowData {
    pub(super) window: *mut GtkWindow,
    pub(super) background_color: Option<u32>,
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

    #[cfg(feature = "progress_bar")]
    fn gtk_set_progress_bar(&mut self, progress: Option<f32>) {
        update_progress_bar(progress);
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
