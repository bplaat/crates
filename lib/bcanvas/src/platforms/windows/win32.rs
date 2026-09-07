/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_camel_case_types, non_snake_case)]
#![allow(clippy::upper_case_acronyms)]

use bwindow::ffi::*;

pub(super) const CS_VREDRAW: u32 = 0x0001;
pub(super) const CS_HREDRAW: u32 = 0x0002;
pub(super) const WS_CHILD: u32 = 0x40000000;
pub(super) const WS_VISIBLE: u32 = 0x10000000;
pub(super) const WS_TABSTOP: u32 = 0x00010000;
pub(super) const WM_SETFOCUS: u32 = 0x0007;
pub(super) const WM_KILLFOCUS: u32 = 0x0008;
pub(super) const WM_PAINT: u32 = 0x000f;
pub(super) const WM_SETCURSOR: u32 = 0x0020;
pub(super) const WM_NCDESTROY: u32 = 0x0082;
pub(super) const WM_KEYDOWN: u32 = 0x0100;
pub(super) const WM_KEYUP: u32 = 0x0101;
pub(super) const WM_SYSKEYDOWN: u32 = 0x0104;
pub(super) const WM_SYSKEYUP: u32 = 0x0105;
pub(super) const WM_TIMER: u32 = 0x0113;
pub(super) const WM_MOUSEMOVE: u32 = 0x0200;
pub(super) const WM_LBUTTONDOWN: u32 = 0x0201;
pub(super) const WM_LBUTTONUP: u32 = 0x0202;
pub(super) const WM_RBUTTONDOWN: u32 = 0x0204;
pub(super) const WM_RBUTTONUP: u32 = 0x0205;
pub(super) const WM_MBUTTONDOWN: u32 = 0x0207;
pub(super) const WM_MBUTTONUP: u32 = 0x0208;
pub(super) const WM_MOUSEWHEEL: u32 = 0x020a;
pub(super) const WM_XBUTTONDOWN: u32 = 0x020b;
pub(super) const WM_XBUTTONUP: u32 = 0x020c;
pub(super) const WM_MOUSEHWHEEL: u32 = 0x020e;
pub(super) const WM_MOUSELEAVE: u32 = 0x02a3;
pub(super) const HTCLIENT: u16 = 1;
pub(super) const TME_LEAVE: u32 = 2;
pub(super) const IDC_ARROW: *const u16 = 32512usize as *const u16;
pub(super) const IDC_HAND: *const u16 = 32649usize as *const u16;
pub(super) const IDC_APPSTARTING: *const u16 = 32650usize as *const u16;

#[repr(C)]
#[derive(Default)]
pub(super) struct PAINTSTRUCT {
    pub(super) hdc: HDC,
    pub(super) fErase: BOOL,
    pub(super) rcPaint: RECT,
    pub(super) fRestore: BOOL,
    pub(super) fIncUpdate: BOOL,
    pub(super) rgbReserved: [u8; 32],
}

#[repr(C)]
#[derive(Default)]
pub(super) struct TRACKMOUSEEVENT {
    pub(super) cbSize: u32,
    pub(super) dwFlags: u32,
    pub(super) hwndTrack: HWND,
    pub(super) dwHoverTime: u32,
}

const _: () = {
    let wide = size_of::<HWND>() == 8;
    assert!(size_of::<PAINTSTRUCT>() == if wide { 72 } else { 64 });
    assert!(size_of::<TRACKMOUSEEVENT>() == if wide { 24 } else { 16 });
};

#[link(name = "user32")]
unsafe extern "system" {
    pub(super) fn BeginPaint(hwnd: HWND, paint: *mut PAINTSTRUCT) -> HDC;
    pub(super) fn EndPaint(hwnd: HWND, paint: *const PAINTSTRUCT) -> BOOL;
    pub(super) fn SetTimer(
        hwnd: HWND,
        id: usize,
        interval: u32,
        callback: Option<unsafe extern "system" fn(HWND, u32, usize, u32)>,
    ) -> usize;
    pub(super) fn KillTimer(hwnd: HWND, id: usize) -> BOOL;
    pub(super) fn GetCursorPos(point: *mut POINT) -> BOOL;
    pub(super) fn WindowFromPoint(point: POINT) -> HWND;
    pub(super) fn SetCursor(cursor: HCURSOR) -> HCURSOR;
    pub(super) fn LoadCursorW(instance: HMODULE, name: *const u16) -> HCURSOR;
    pub(super) fn SetFocus(hwnd: HWND) -> HWND;
    pub(super) fn SetCapture(hwnd: HWND) -> HWND;
    pub(super) fn ReleaseCapture() -> BOOL;
    pub(super) fn TrackMouseEvent(event: *mut TRACKMOUSEEVENT) -> BOOL;
}
