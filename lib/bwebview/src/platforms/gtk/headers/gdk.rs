/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unused)]

use std::ffi::{c_char, c_void};

// MARK: GDK
#[repr(C)]
pub(crate) struct GdkDisplay([u8; 0]);
#[repr(C)]
pub(crate) struct GdkCursor([u8; 0]);
#[repr(C)]
pub(crate) struct GdkWindow([u8; 0]);
#[repr(C)]
pub(crate) struct GdkDragContext([u8; 0]);
#[repr(C)]
pub(crate) struct GdkEventButton {
    pub r#type: i32,
    pub window: *mut GdkWindow,
    pub send_event: i8,
    pub time: u32,
    pub x: f64,
    pub y: f64,
    pub axes: *mut f64,
    pub state: u32,
    pub button: u32,
    pub device: *mut c_void,
    pub x_root: f64,
    pub y_root: f64,
}
#[repr(C)]
pub(crate) struct GdkEventMotion {
    pub r#type: i32,
    pub window: *mut GdkWindow,
    pub send_event: i8,
    pub time: u32,
    pub x: f64,
    pub y: f64,
    pub axes: *mut f64,
    pub state: u32,
    pub is_hint: i16,
    pub device: *mut c_void,
    pub x_root: f64,
    pub y_root: f64,
}
#[repr(C)]
pub(crate) struct GdkEventCrossing {
    pub r#type: i32,
    pub window: *mut GdkWindow,
    pub send_event: i8,
    pub subwindow: *mut GdkWindow,
    pub time: u32,
    pub x: f64,
    pub y: f64,
    pub x_root: f64,
    pub y_root: f64,
    pub mode: i32,
    pub detail: i32,
    pub focus: i32,
    pub state: u32,
}
#[repr(C)]
pub(crate) struct GdkEventScroll {
    pub r#type: i32,
    pub window: *mut GdkWindow,
    pub send_event: i8,
    pub time: u32,
    pub x: f64,
    pub y: f64,
    pub state: u32,
    pub direction: i32,
    pub device: *mut c_void,
    pub x_root: f64,
    pub y_root: f64,
    pub delta_x: f64,
    pub delta_y: f64,
    pub is_stop: u32,
}
#[repr(C)]
pub(crate) struct GdkEventKey {
    pub r#type: i32,
    pub window: *mut GdkWindow,
    pub send_event: i8,
    pub time: u32,
    pub state: u32,
    pub keyval: u32,
    pub length: i32,
    pub string: *mut c_char,
    pub hardware_keycode: u16,
    pub group: u8,
    pub is_modifier: u32,
}
#[repr(C)]
pub(crate) struct GdkRectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
#[repr(C)]
pub(crate) struct GdkRGBA {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64,
}
// GTK < 3.22
#[repr(C)]
pub(crate) struct GdkScreen([u8; 0]);

// GTK 3.22+
#[repr(C)]
pub(crate) struct GdkMonitor([u8; 0]);

unsafe extern "C" {
    pub(crate) fn gdk_display_get_default() -> *mut GdkDisplay;
    pub(crate) fn gdk_display_get_name(display: *mut GdkDisplay) -> *const c_char;
    pub(crate) fn gdk_cursor_new_from_name(
        display: *mut GdkDisplay,
        name: *const c_char,
    ) -> *mut GdkCursor;
    pub(crate) fn gdk_window_set_cursor(window: *mut GdkWindow, cursor: *mut GdkCursor);
    pub(crate) fn gdk_keyval_to_unicode(keyval: u32) -> u32;
    pub(crate) fn gdk_keyval_name(keyval: u32) -> *const c_char;

    // GTK < 3.22
    pub(crate) fn gdk_screen_get_default() -> *mut GdkScreen;
    pub(crate) fn gdk_screen_get_n_monitors(screen: *mut GdkScreen) -> i32;
    pub(crate) fn gdk_screen_get_primary_monitor(screen: *mut GdkScreen) -> i32;
    pub(crate) fn gdk_screen_get_monitor_geometry(
        screen: *mut GdkScreen,
        monitor_num: i32,
        dest: *mut GdkRectangle,
    );
    pub(crate) fn gdk_screen_get_monitor_scale_factor(
        screen: *mut GdkScreen,
        monitor_num: i32,
    ) -> i32;
    pub(crate) fn gdk_screen_get_monitor_plug_name(
        screen: *mut GdkScreen,
        monitor_num: i32,
    ) -> *mut c_char;

    // GTK 3.22+
    pub(crate) fn gdk_display_get_n_monitors(display: *mut GdkDisplay) -> i32;
    pub(crate) fn gdk_display_get_monitor(
        display: *mut GdkDisplay,
        monitor_num: i32,
    ) -> *mut GdkMonitor;
    pub(crate) fn gdk_display_get_primary_monitor(display: *mut GdkDisplay) -> *mut GdkMonitor;
    pub(crate) fn gdk_monitor_get_model(monitor: *mut GdkMonitor) -> *const c_char;
    pub(crate) fn gdk_monitor_get_geometry(monitor: *mut GdkMonitor, geometry: *mut GdkRectangle);
    pub(crate) fn gdk_monitor_get_scale_factor(monitor: *mut GdkMonitor) -> i32;
    pub(crate) fn gdk_monitor_is_primary(monitor: *mut GdkMonitor) -> bool;
}
