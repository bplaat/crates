/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]

use std::ffi::{c_char, c_void};

// MARK: kernel32.dll
pub(crate) type HMODULE = *const c_void;

#[link(name = "kernel32")]
unsafe extern "system" {
    pub(crate) fn GetModuleHandleA(lpModuleName: *const u8) -> HMODULE;
}

// MARK:  user32.dll
pub(crate) type HWND = *mut c_void;
pub(crate) type HCURSOR = *mut c_void;
pub(crate) type HICON = *mut c_void;
pub(crate) type HMENU = *mut c_void;
pub(crate) type WPARAM = usize;
pub(crate) type LPARAM = isize;
pub(crate) type LRESULT = isize;

#[repr(C)]
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
pub(crate) struct RECT {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

pub(crate) const CS_HREDRAW: u32 = 0x0002;
pub(crate) const CS_VREDRAW: u32 = 0x0001;

pub(crate) const IDC_ARROW: *const u16 = 32512 as *const u16;

pub(crate) const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;

pub(crate) const CW_USEDEFAULT: i32 = 0x80000000u32 as i32;

pub(crate) const WM_DESTROY: u32 = 0x0002;
pub(crate) const WM_SIZE: u32 = 0x0005;

pub(crate) const SW_SHOWDEFAULT: i32 = 10;

pub(crate) const MB_OK: u32 = 0x00000000;

pub(crate) const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: isize = -4isize;

#[link(name = "user32")]
unsafe extern "system" {
    pub(crate) fn LoadCursorA(hInstance: HMODULE, lpCursorName: *const u16) -> HCURSOR;
    pub(crate) fn RegisterClassExA(lpWndClass: *const WNDCLASSEXA) -> u16;
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
    pub(crate) fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> i32;
    pub(crate) fn UpdateWindow(hWnd: HWND) -> i32;
    pub(crate) fn GetClientRect(hWnd: HWND, rect: *mut RECT) -> i32;
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
    pub(crate) fn MessageBoxA(
        hWnd: HWND,
        lpText: *const c_char,
        lpCaption: *const c_char,
        uType: u32,
    ) -> i32;
}

// MARK: COM
pub(crate) type HRESULT = i32;

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
