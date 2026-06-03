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
// GTK 3.22+ uses GdkMonitor; older GTK uses GdkScreen.
#[cfg(gtk3_22)]
#[repr(C)]
pub(crate) struct GdkMonitor([u8; 0]);
#[cfg(not(gtk3_22))]
#[repr(C)]
pub(crate) struct GdkScreen([u8; 0]);
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
#[link(name = "gdk-3")]
unsafe extern "C" {
    pub(crate) fn gdk_display_get_default() -> *mut GdkDisplay;
    pub(crate) fn gdk_display_get_name(display: *mut GdkDisplay) -> *const c_char;

    // GTK 3.22+ GdkMonitor API
    #[cfg(gtk3_22)]
    pub(crate) fn gdk_display_get_n_monitors(display: *mut GdkDisplay) -> i32;
    #[cfg(gtk3_22)]
    pub(crate) fn gdk_display_get_monitor(
        display: *mut GdkDisplay,
        monitor_num: i32,
    ) -> *mut GdkMonitor;
    #[cfg(gtk3_22)]
    pub(crate) fn gdk_display_get_primary_monitor(display: *mut GdkDisplay) -> *mut GdkMonitor;
    #[cfg(gtk3_22)]
    pub(crate) fn gdk_monitor_get_model(monitor: *mut GdkMonitor) -> *const c_char;
    #[cfg(gtk3_22)]
    pub(crate) fn gdk_monitor_get_geometry(monitor: *mut GdkMonitor, geometry: *mut GdkRectangle);
    #[cfg(gtk3_22)]
    pub(crate) fn gdk_monitor_get_scale_factor(monitor: *mut GdkMonitor) -> i32;
    #[cfg(gtk3_22)]
    pub(crate) fn gdk_monitor_is_primary(monitor: *mut GdkMonitor) -> bool;

    // GTK < 3.22 GdkScreen API (available since GTK 2.x)
    #[cfg(not(gtk3_22))]
    pub(crate) fn gdk_screen_get_default() -> *mut GdkScreen;
    #[cfg(not(gtk3_22))]
    pub(crate) fn gdk_screen_get_n_monitors(screen: *mut GdkScreen) -> i32;
    #[cfg(not(gtk3_22))]
    pub(crate) fn gdk_screen_get_primary_monitor(screen: *mut GdkScreen) -> i32;
    #[cfg(not(gtk3_22))]
    pub(crate) fn gdk_screen_get_monitor_geometry(
        screen: *mut GdkScreen,
        monitor_num: i32,
        dest: *mut GdkRectangle,
    );
    #[cfg(not(gtk3_22))]
    pub(crate) fn gdk_screen_get_monitor_scale_factor(
        screen: *mut GdkScreen,
        monitor_num: i32,
    ) -> i32;
    #[cfg(not(gtk3_22))]
    pub(crate) fn gdk_screen_get_monitor_plug_name(
        screen: *mut GdkScreen,
        monitor_num: i32,
    ) -> *mut c_char;
}
