/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::upper_case_acronyms)]

use std::ffi::{c_char, c_void};

// MARK: Base types
pub(crate) type BOOL = i32;
pub(crate) const TRUE: BOOL = 1;
pub(crate) const FALSE: BOOL = 0;
pub(crate) type w_char = u16;

// MARK: kernel32.dll
pub(crate) type HMODULE = *const c_void;

#[link(name = "kernel32")]
unsafe extern "system" {
    pub(crate) fn GetModuleHandleA(lpModuleName: *const u8) -> HMODULE;
}

// MARK: gdi32.dll
pub(crate) type HBRUSH = *mut c_void;
pub(crate) type HDC = *mut c_void;

#[link(name = "gdi32")]
unsafe extern "system" {
    pub(crate) fn FillRect(hdc: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    pub(crate) fn CreateSolidBrush(color: u32) -> HBRUSH;
}

// MARK: user32.dll
pub(crate) type HWND = *mut c_void;
pub(crate) type HCURSOR = *mut c_void;
pub(crate) type HICON = *mut c_void;
pub(crate) type HMENU = *mut c_void;
pub(crate) type HMONITOR = *mut c_void;
pub(crate) type ATOM = u16;
pub(crate) type WPARAM = usize;
pub(crate) type LPARAM = isize;
pub(crate) type LRESULT = isize;

#[repr(C)]
#[derive(Default)]
pub(crate) struct WNDCLASSEXA {
    pub(crate) cbSize: u32,
    pub(crate) style: u32,
    pub(crate) lpfnWndProc: Option<
        unsafe extern "system" fn(
            window: HWND,
            message: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT,
    >,
    pub(crate) cbClsExtra: i32,
    pub(crate) cbWndExtra: i32,
    pub(crate) hInstance: HMODULE,
    pub(crate) hIcon: HICON,
    pub(crate) hCursor: HCURSOR,
    pub(crate) hbrBackground: usize,
    pub(crate) lpszMenuName: *const c_char,
    pub(crate) lpszClassName: *const c_char,
    pub(crate) hIconSm: HICON,
}

#[repr(C)]
pub(crate) struct MSG {
    pub(crate) hwnd: HWND,
    pub(crate) message: u32,
    pub(crate) wParam: WPARAM,
    pub(crate) lParam: LPARAM,
    pub(crate) time: u32,
    pub(crate) pt_x: i32,
    pub(crate) pt_y: i32,
}

#[repr(C)]
pub(crate) struct POINT {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[repr(C)]
#[derive(Default, Clone)]
pub(crate) struct RECT {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

#[repr(C)]
pub(crate) struct MINMAXINFO {
    pub(crate) ptReserved: POINT,
    pub(crate) ptMaxSize: POINT,
    pub(crate) ptMaxPosition: POINT,
    pub(crate) ptMinTrackSize: POINT,
    pub(crate) ptMaxTrackSize: POINT,
}

#[repr(C)]
#[derive(Default)]
pub(crate) struct MONITORINFOEXA {
    pub(crate) cbSize: u32,
    pub(crate) rcMonitor: RECT,
    pub(crate) rcWork: RECT,
    pub(crate) dwFlags: u32,
    pub(crate) szDevice: [c_char; 32],
}

#[repr(C)]
pub(crate) struct WINDOWPLACEMENT {
    pub(crate) length: u32,
    pub(crate) flags: u32,
    pub(crate) showCmd: u32,
    pub(crate) ptMinPosition: POINT,
    pub(crate) ptMaxPosition: POINT,
    pub(crate) rcNormalPosition: RECT,
}

pub(crate) const WS_POPUP: u32 = 0x80000000;
pub(crate) const WS_THICKFRAME: u32 = 0x00040000;
pub(crate) const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;

pub(crate) const CW_USEDEFAULT: i32 = 0x80000000u32 as i32;

pub(crate) const WM_CREATE: u32 = 0x0001;
pub(crate) const WM_DESTROY: u32 = 0x0002;
pub(crate) const WM_MOVE: u32 = 0x0003;
pub(crate) const WM_SIZE: u32 = 0x0005;
pub(crate) const WM_CLOSE: u32 = 0x0010;
pub(crate) const WM_ERASEBKGND: u32 = 0x0014;
pub(crate) const WM_GETMINMAXINFO: u32 = 0x0024;
pub(crate) const WM_DPICHANGED: u32 = 0x02E0;
pub(crate) const WM_USER: u32 = 0x0400;

pub(crate) const SW_SHOWDEFAULT: i32 = 10;
pub(crate) const SW_SHOWNORMAL: i32 = 1;

pub(crate) const MB_OK: u32 = 0x00000000;

pub(crate) const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: isize = -4isize;

pub(crate) const MONITOR_DEFAULTTOPRIMARY: u32 = 0x00000001;

pub(crate) const USER_DEFAULT_SCREEN_DPI: u32 = 96;
pub(crate) const MDT_EFFECTIVE_DPI: i32 = 0;

pub(crate) const SM_CXSCREEN: i32 = 0;
pub(crate) const SM_CYSCREEN: i32 = 1;

pub(crate) const GWL_STYLE: i32 = -16;
pub(crate) const GWL_USERDATA: i32 = -21;

pub(crate) const SWP_NOSIZE: u32 = 0x0001;
pub(crate) const SWP_NOZORDER: u32 = 0x0004;
pub(crate) const SWP_NOACTIVATE: u32 = 0x0010;
pub(crate) const SWP_NOREPOSITION: u32 = 0x0200;

#[link(name = "user32")]
unsafe extern "system" {
    pub(crate) fn ExtractIconExA(
        lpszFile: *const c_char,
        nIconIndex: i32,
        phiconLarge: *mut HICON,
        phiconSmall: *mut HICON,
        nIcons: u32,
    ) -> u32;
    pub(crate) fn GetClassInfoExA(
        hInstance: HMODULE,
        lpClassName: *const c_char,
        lpWndClass: *mut WNDCLASSEXA,
    ) -> BOOL;
    pub(crate) fn RegisterClassExA(lpWndClass: *const WNDCLASSEXA) -> ATOM;
    pub(crate) fn CreateWindowExA(
        dwExStyle: u32,
        lpClassName: *const c_char,
        lpWindowName: *const c_char,
        dwStyle: u32,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HMODULE,
        lpParam: LPARAM,
    ) -> HWND;
    pub(crate) fn SetWindowTextA(hWnd: HWND, lpString: *const c_char) -> i32;
    pub(crate) fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> i32;
    pub(crate) fn UpdateWindow(hWnd: HWND) -> i32;
    pub(crate) fn GetWindowRect(hWnd: HWND, rect: *mut RECT) -> i32;
    pub(crate) fn GetClientRect(hWnd: HWND, rect: *mut RECT) -> i32;
    pub(crate) fn SetWindowPos(
        hWnd: HWND,
        hWndInsertAfter: HWND,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> BOOL;
    pub(crate) fn GetMessageA(
        lpMsg: *mut MSG,
        hWnd: HWND,
        wMsgFilterMin: u32,
        wMsgFilterMax: u32,
    ) -> i32;
    pub(crate) fn TranslateMessage(lpMsg: *const MSG) -> i32;
    pub(crate) fn DispatchMessageA(lpMsg: *const MSG) -> isize;
    pub(crate) fn DefWindowProcA(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> isize;
    pub(crate) fn PostQuitMessage(nExitCode: i32);
    pub(crate) fn SetProcessDpiAwarenessContext(value: isize) -> i32;
    pub(crate) fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL;
    pub(crate) fn MessageBoxA(
        hWnd: HWND,
        lpText: *const c_char,
        lpCaption: *const c_char,
        uType: u32,
    ) -> i32;
    #[allow(dead_code)]
    pub(crate) fn GetWindowLongA(hwnd: HWND, index: i32) -> i32;
    #[allow(dead_code)]
    pub(crate) fn GetWindowLongPtrA(hwnd: HWND, index: i32) -> isize;
    #[allow(dead_code)]
    pub(crate) fn SetWindowLongA(hwnd: HWND, index: i32, value: i32) -> i32;
    #[allow(dead_code)]
    pub(crate) fn SetWindowLongPtrA(hwnd: HWND, index: i32, value: isize) -> isize;
    pub(crate) fn MonitorFromPoint(pt: POINT, dwFlags: u32) -> *mut c_void;
    pub(crate) fn EnumDisplayMonitors(
        hdc: HDC,
        lprcClip: *const RECT,
        lpfnEnum: unsafe extern "system" fn(
            hMonitor: HMONITOR,
            hdcMonitor: HDC,
            lprcMonitor: *const RECT,
            dwData: LPARAM,
        ) -> BOOL,
        dwData: LPARAM,
    ) -> BOOL;
    pub(crate) fn PostMessageA(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    pub(crate) fn GetMonitorInfoA(hMonitor: HMONITOR, lpmi: *mut MONITORINFOEXA) -> BOOL;
    pub(crate) fn GetDpiForSystem() -> u32;
    pub(crate) fn GetSystemMetrics(nIndex: i32) -> i32;
    pub(crate) fn AdjustWindowRectExForDpi(
        lpRect: *mut RECT,
        dwStyle: u32,
        bMenu: BOOL,
        dwExStyle: u32,
        dpi: u32,
    ) -> BOOL;
    pub(crate) fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub(crate) fn GetWindowPlacement(hWnd: HWND, lpwndpl: *mut WINDOWPLACEMENT) -> BOOL;
    pub(crate) fn SetWindowPlacement(hWnd: HWND, lpwndpl: *const WINDOWPLACEMENT) -> BOOL;
}

#[cfg(target_pointer_width = "32")]
pub(crate) unsafe fn GetWindowLong(hwnd: HWND, index: i32) -> isize {
    (unsafe { GetWindowLongA(hwnd, index) }) as isize
}
#[cfg(target_pointer_width = "64")]
pub(crate) unsafe fn GetWindowLong(hwnd: HWND, index: i32) -> isize {
    unsafe { GetWindowLongPtrA(hwnd, index) }
}

#[cfg(target_pointer_width = "32")]
pub(crate) unsafe fn SetWindowLong(hwnd: HWND, index: i32, value: isize) -> isize {
    (unsafe { SetWindowLongA(hwnd, index, value as i32) }) as isize
}
#[cfg(target_pointer_width = "64")]
pub(crate) unsafe fn SetWindowLong(hwnd: HWND, index: i32, value: isize) -> isize {
    unsafe { SetWindowLongPtrA(hwnd, index, value) }
}

// MARK: shell32.dll
#[link(name = "shell32")]
unsafe extern "system" {
    pub(crate) fn ShellExecuteA(
        hwnd: HWND,
        lpOperation: *const c_char,
        lpFile: *const c_char,
        lpParameters: *const c_char,
        lpDirectory: *const c_char,
        nShowCmd: i32,
    ) -> isize;
}

// MARK: shcore.dll
#[link(name = "shcore")]
unsafe extern "system" {
    pub(crate) fn GetDpiForMonitor(
        hmonitor: HMONITOR,
        dpiType: i32,
        dpiX: *mut u32,
        dpiY: *mut u32,
    ) -> HRESULT;
}

// MARK: dwmapi.dll
pub(crate) const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;

#[link(name = "dwmapi")]
unsafe extern "system" {
    pub(crate) fn DwmSetWindowAttribute(
        hwnd: HWND,
        dwAttribute: u32,
        pvAttribute: *const c_void,
        cbAttribute: u32,
    ) -> i32;
}

// MARK: COM
pub(crate) type HRESULT = i32;

#[repr(C)]
pub(crate) struct GUID {
    pub(crate) data1: u32,
    pub(crate) data2: u16,
    pub(crate) data3: u16,
    pub(crate) data4: [u8; 8],
}

pub(crate) const COINIT_APARTMENTTHREADED: u32 = 0x2;
pub(crate) const COINIT_DISABLE_OLE1DDE: u32 = 0x4;

pub(crate) const S_OK: HRESULT = 0;
pub(crate) const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT;
pub(crate) const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT;

#[link(name = "ole32")]
unsafe extern "system" {
    pub(crate) fn CoInitializeEx(pvReserved: *mut c_void, dwCoInit: u32) -> HRESULT;
    pub(crate) fn CoUninitialize();
}

// Utils
pub(crate) fn str_to_wchar(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    v
}

pub(crate) fn wchar_to_string(ptr: *const u16) -> String {
    unsafe {
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len))
    }
}
