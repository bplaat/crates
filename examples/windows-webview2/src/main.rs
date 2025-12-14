/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal windows webview2 example

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![allow(non_upper_case_globals)]

use std::ffi::c_void;
use std::process::exit;
use std::ptr::{null, null_mut};

use crate::webview2::*;
use crate::windows::*;

mod webview2;
mod windows;

static mut g_hwnd: HWND = null_mut();
static mut g_webview_controller: *mut ICoreWebView2Controller = null_mut();

fn main() {
    unsafe {
        // Initialize COM
        CoInitializeEx(
            null_mut(),
            COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
        );

        // Enable high DPI support
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        // Register window class
        let wc = WNDCLASSEXA {
            hCursor: LoadCursorA(null_mut(), IDC_ARROW),
            hInstance: GetModuleHandleA(null()),
            lpszClassName: c"window".as_ptr(),
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: null_mut(),
            hbrBackground: 0,
            lpszMenuName: null(),
            cbSize: size_of::<WNDCLASSEXA>() as u32,
            hIconSm: null_mut(),
        };
        RegisterClassExA(&wc);

        // Create window
        g_hwnd = CreateWindowExA(
            0,
            wc.lpszClassName,
            c"This is a sample window".as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            null_mut(),
            null_mut(),
            wc.hInstance,
            0,
        );
        ShowWindow(g_hwnd, SW_SHOWDEFAULT);
        UpdateWindow(g_hwnd);

        // Init WebView2
        static vtbl: ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandlerVtbl =
            ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandlerVtbl {
                QueryInterface: unimplemented_query_interface,
                AddRef: unimplemented_add_ref,
                Release: unimplemented_release,
                Invoke: environment_created,
            };
        let creation_completed_handler = Box::into_raw(Box::new(
            ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler { lpVtbl: &vtbl },
        ));
        if CreateCoreWebView2EnvironmentWithOptions(
            null(),
            null(),
            null_mut(),
            creation_completed_handler,
        ) != S_OK
        {
            MessageBoxA(
                null_mut(),
                c"Failed to create WebView2 environment".as_ptr(),
                c"Error".as_ptr(),
                MB_OK,
            );
            exit(1);
        }

        // Message loop
        let mut message = std::mem::zeroed();
        while GetMessageA(&mut message, null_mut(), 0, 0) != 0 {
            TranslateMessage(&message);
            DispatchMessageA(&message);
        }
        CoUninitialize();
        exit(message.wParam as i32)
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_SIZE => {
                if !g_webview_controller.is_null() {
                    let mut rect: RECT = std::mem::zeroed();
                    GetClientRect(window, &mut rect);
                    (*g_webview_controller).put_Bounds(rect);
                }
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}

extern "system" fn unimplemented_query_interface(
    _this: *mut c_void,
    _riid: *const c_void,
    _ppv_object: *mut *mut c_void,
) -> HRESULT {
    E_NOINTERFACE
}
extern "system" fn unimplemented_add_ref(_this: *mut c_void) -> HRESULT {
    E_NOTIMPL
}
extern "system" fn unimplemented_release(_this: *mut c_void) -> HRESULT {
    E_NOTIMPL
}

extern "system" fn environment_created(
    _this: *mut ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
    _result: HRESULT,
    environment: *mut ICoreWebView2Environment,
) -> HRESULT {
    unsafe {
        static vtbl: ICoreWebView2CreateCoreWebView2ControllerCompletedHandlerVtbl =
            ICoreWebView2CreateCoreWebView2ControllerCompletedHandlerVtbl {
                QueryInterface: unimplemented_query_interface,
                AddRef: unimplemented_add_ref,
                Release: unimplemented_release,
                Invoke: controller_created,
            };
        let creation_completed_handler = Box::into_raw(Box::new(
            ICoreWebView2CreateCoreWebView2ControllerCompletedHandler { lpVtbl: &vtbl },
        ));
        (*environment).CreateCoreWebView2Controller(g_hwnd, creation_completed_handler);

        S_OK
    }
}

extern "system" fn controller_created(
    _this: *mut ICoreWebView2CreateCoreWebView2ControllerCompletedHandler,
    _result: HRESULT,
    controller: *mut ICoreWebView2Controller,
) -> HRESULT {
    unsafe {
        (*controller).AddRef();
        g_webview_controller = controller;

        let mut rect: RECT = std::mem::zeroed();
        GetClientRect(g_hwnd, &mut rect);
        (*controller).put_Bounds(rect);

        let mut webview: *mut ICoreWebView2 = null_mut();
        (*controller).get_CoreWebView2(&mut webview);

        let url = "https://www.example.com\0"
            .encode_utf16()
            .collect::<Vec<u16>>();
        (*webview).Navigate(url.as_ptr());

        S_OK
    }
}
