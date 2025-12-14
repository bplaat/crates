/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]

use std::ffi::c_void;

use crate::windows::*;

#[cfg_attr(not(target_env = "msvc"), link(name = "WebView2Loader"))]
unsafe extern "system" {
    pub(crate) fn CreateCoreWebView2EnvironmentWithOptions(
        browserExecutableFolder: *const u16,
        userDataFolder: *const u16,
        environmentOptions: *const c_void,
        creationCompletedHandler: *mut ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
    ) -> HRESULT;
}

#[cfg_attr(target_env = "msvc", link(name = "advapi32"))]
unsafe extern "C" {}

// ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler
#[repr(C)]
pub(crate) struct ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler {
    pub(crate) lpVtbl: *const ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandlerVtbl,
}

#[repr(C)]
pub(crate) struct ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandlerVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const c_void,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Invoke: unsafe extern "system" fn(
        This: *mut ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
        hr: HRESULT,
        env: *mut ICoreWebView2Environment,
    ) -> HRESULT,
}

// ICoreWebView2Environment
#[repr(C)]
pub(crate) struct ICoreWebView2Environment {
    pub(crate) lpVtbl: *const ICoreWebView2EnvironmentVtbl,
}

impl ICoreWebView2Environment {
    pub(crate) unsafe fn CreateCoreWebView2Controller(
        &self,
        parentWindow: HWND,
        controllerCreatedHandler: *mut ICoreWebView2CreateCoreWebView2ControllerCompletedHandler,
    ) -> HRESULT {
        unsafe {
            ((*self.lpVtbl).CreateCoreWebView2Controller)(
                self as *const _ as *mut _,
                parentWindow,
                controllerCreatedHandler,
            )
        }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2EnvironmentVtbl {
    QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const c_void,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    CreateCoreWebView2Controller: unsafe extern "system" fn(
        This: *mut ICoreWebView2Environment,
        parentWindow: HWND,
        controllerCreatedHandler: *mut ICoreWebView2CreateCoreWebView2ControllerCompletedHandler,
    ) -> HRESULT,
}

// ICoreWebView2CreateCoreWebView2ControllerCompletedHandler
#[repr(C)]
pub(crate) struct ICoreWebView2CreateCoreWebView2ControllerCompletedHandler {
    pub(crate) lpVtbl: *const ICoreWebView2CreateCoreWebView2ControllerCompletedHandlerVtbl,
}

#[repr(C)]
pub(crate) struct ICoreWebView2CreateCoreWebView2ControllerCompletedHandlerVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const c_void,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Invoke: unsafe extern "system" fn(
        This: *mut ICoreWebView2CreateCoreWebView2ControllerCompletedHandler,
        hr: HRESULT,
        controller: *mut ICoreWebView2Controller,
    ) -> HRESULT,
}

// ICoreWebView2Controller
#[repr(C)]
pub(crate) struct ICoreWebView2Controller {
    pub(crate) lpVtbl: *const ICoreWebView2ControllerVtbl,
}

impl ICoreWebView2Controller {
    pub(crate) unsafe fn AddRef(&self) -> HRESULT {
        unsafe { ((*self.lpVtbl).AddRef)(self as *const _ as *mut _) }
    }

    pub(crate) unsafe fn put_Bounds(&self, bounds: RECT) -> HRESULT {
        unsafe { ((*self.lpVtbl).put_Bounds)(self as *const _ as *mut _, bounds) }
    }

    pub(crate) unsafe fn get_CoreWebView2(&self, webview: *mut *mut ICoreWebView2) -> HRESULT {
        unsafe { ((*self.lpVtbl).get_CoreWebView2)(self as *const _ as *mut _, webview) }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2ControllerVtbl {
    QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const c_void,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    padding: [usize; 3],
    put_Bounds:
        unsafe extern "system" fn(This: *mut ICoreWebView2Controller, bounds: RECT) -> HRESULT,
    padding2: [usize; 18],
    get_CoreWebView2: unsafe extern "system" fn(
        This: *mut ICoreWebView2Controller,
        webview: *mut *mut ICoreWebView2,
    ) -> HRESULT,
}

// ICoreWebView2
#[repr(C)]
pub(crate) struct ICoreWebView2 {
    pub(crate) lpVtbl: *const ICoreWebView2Vtbl,
}

impl ICoreWebView2 {
    pub(crate) unsafe fn Navigate(&self, uri: *const u16) -> HRESULT {
        unsafe { ((*self.lpVtbl).Navigate)(self as *const _ as *mut _, uri) }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2Vtbl {
    QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const c_void,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    padding: [usize; 2],
    Navigate: unsafe extern "system" fn(This: *mut ICoreWebView2, uri: *const u16) -> HRESULT,
}
