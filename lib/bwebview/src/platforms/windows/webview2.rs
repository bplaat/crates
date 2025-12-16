/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::upper_case_acronyms)]
#![allow(unused)]

use std::ffi::c_void;

use super::win32::*;

#[cfg_attr(not(target_env = "msvc"), link(name = "WebView2Loader"))]
unsafe extern "system" {
    pub(crate) fn CreateCoreWebView2EnvironmentWithOptions(
        browserExecutableFolder: *const w_char,
        userDataFolder: *const w_char,
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
    pub(crate) user_data: *mut c_void,
}

#[repr(C)]
pub(crate) struct ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandlerVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
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
    pub(crate) unsafe fn AddRef(&self) -> HRESULT {
        unsafe { ((*self.lpVtbl).AddRef)(self as *const _ as *mut _) }
    }

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

    pub(crate) unsafe fn CreateWebResourceResponse(
        &self,
        content: *mut IStream,
        statusCode: u32,
        reasonPhrase: *const w_char,
        headers: *const w_char,
        response: *mut *mut ICoreWebView2WebResourceResponse,
    ) -> HRESULT {
        unsafe {
            ((*self.lpVtbl).CreateWebResourceResponse)(
                self as *const _ as *mut _,
                content,
                statusCode,
                reasonPhrase,
                headers,
                response,
            )
        }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2EnvironmentVtbl {
    QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    CreateCoreWebView2Controller: unsafe extern "system" fn(
        This: *mut ICoreWebView2Environment,
        parentWindow: HWND,
        controllerCreatedHandler: *mut ICoreWebView2CreateCoreWebView2ControllerCompletedHandler,
    ) -> HRESULT,
    CreateWebResourceResponse: unsafe extern "system" fn(
        This: *mut ICoreWebView2Environment,
        content: *mut IStream,
        statusCode: u32,
        reasonPhrase: *const w_char,
        headers: *const w_char,
        response: *mut *mut ICoreWebView2WebResourceResponse,
    ) -> HRESULT,
}

// ICoreWebView2CreateCoreWebView2ControllerCompletedHandler
#[repr(C)]
pub(crate) struct ICoreWebView2CreateCoreWebView2ControllerCompletedHandler {
    pub(crate) lpVtbl: *const ICoreWebView2CreateCoreWebView2ControllerCompletedHandlerVtbl,
    pub(crate) user_data: *mut c_void,
}

#[repr(C)]
pub(crate) struct ICoreWebView2CreateCoreWebView2ControllerCompletedHandlerVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
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
    pub(crate) unsafe fn QueryInterface(
        &self,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT {
        unsafe { ((*self.lpVtbl).QueryInterface)(self as *const _ as *mut _, riid, ppvObject) }
    }

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
        riid: *const GUID,
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

// ICoreWebView2Controller2
pub(crate) const IID_ICoreWebView2Controller2: GUID = GUID {
    data1: 0xc979903e,
    data2: 0xd4ca,
    data3: 0x4228,
    data4: [0x92, 0xeb, 0x47, 0xee, 0x3f, 0xa9, 0x6e, 0xab],
};

#[repr(C)]
pub(crate) struct ICoreWebView2Controller2 {
    pub(crate) lpVtbl: *const ICoreWebView2Controller2Vtbl,
}

impl ICoreWebView2Controller2 {
    pub(crate) unsafe fn put_DefaultBackgroundColor(&self, color: u32) -> HRESULT {
        unsafe { ((*self.lpVtbl).put_DefaultBackgroundColor)(self as *const _ as *mut _, color) }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2Controller2Vtbl {
    QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    padding: [usize; 24],
    put_DefaultBackgroundColor:
        unsafe extern "system" fn(This: *mut ICoreWebView2Controller2, color: u32) -> HRESULT,
}

// ICoreWebView2
pub(crate) const COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL: u32 = 0;

#[repr(C)]
pub(crate) struct ICoreWebView2 {
    pub(crate) lpVtbl: *const ICoreWebView2Vtbl,
}

impl ICoreWebView2 {
    pub(crate) unsafe fn get_Settings(&self, settings: *mut *mut ICoreWebView2Settings) -> HRESULT {
        unsafe { ((*self.lpVtbl).get_Settings)(self as *const _ as *mut _, settings) }
    }

    pub(crate) unsafe fn get_Source(&self, uri: *mut *mut w_char) -> HRESULT {
        unsafe { ((*self.lpVtbl).get_Source)(self as *const _ as *mut _, uri) }
    }

    pub(crate) unsafe fn Navigate(&self, uri: *const w_char) -> HRESULT {
        unsafe { ((*self.lpVtbl).Navigate)(self as *const _ as *mut _, uri) }
    }

    pub(crate) unsafe fn NavigateToString(&self, html: *const w_char) -> HRESULT {
        unsafe { ((*self.lpVtbl).NavigateToString)(self as *const _ as *mut _, html) }
    }

    pub(crate) unsafe fn add_NavigationStarting(
        &self,
        eventHandler: *mut ICoreWebView2NavigationStartingEventHandler,
        token: *mut u32,
    ) -> HRESULT {
        unsafe {
            ((*self.lpVtbl).add_NavigationStarting)(self as *const _ as *mut _, eventHandler, token)
        }
    }

    pub(crate) unsafe fn add_NavigationCompleted(
        &self,
        eventHandler: *mut ICoreWebView2NavigationCompletedEventHandler,
        token: *mut u32,
    ) -> HRESULT {
        unsafe {
            ((*self.lpVtbl).add_NavigationCompleted)(
                self as *const _ as *mut _,
                eventHandler,
                token,
            )
        }
    }

    pub(crate) unsafe fn AddScriptToExecuteOnDocumentCreated(
        &self,
        script: *const w_char,
        resultHandler: *mut c_void,
    ) -> HRESULT {
        unsafe {
            ((*self.lpVtbl).AddScriptToExecuteOnDocumentCreated)(
                self as *const _ as *mut _,
                script,
                resultHandler,
            )
        }
    }

    pub(crate) unsafe fn ExecuteScript(
        &self,
        script: *const w_char,
        resultHandler: *mut c_void,
    ) -> HRESULT {
        unsafe { ((*self.lpVtbl).ExecuteScript)(self as *const _ as *mut _, script, resultHandler) }
    }

    pub(crate) unsafe fn add_WebMessageReceived(
        &self,
        eventHandler: *mut ICoreWebView2WebMessageReceivedEventHandler,
        token: *mut u32,
    ) -> HRESULT {
        unsafe {
            ((*self.lpVtbl).add_WebMessageReceived)(self as *const _ as *mut _, eventHandler, token)
        }
    }

    pub(crate) unsafe fn add_NewWindowRequested(
        &self,
        eventHandler: *mut ICoreWebView2NewWindowRequestedEventHandler,
        token: *mut u32,
    ) -> HRESULT {
        unsafe {
            ((*self.lpVtbl).add_NewWindowRequested)(self as *const _ as *mut _, eventHandler, token)
        }
    }

    pub(crate) unsafe fn add_DocumentTitleChanged(
        &self,
        eventHandler: *mut ICoreWebView2DocumentTitleChangedEventHandler,
        token: *mut u32,
    ) -> HRESULT {
        unsafe {
            ((*self.lpVtbl).add_DocumentTitleChanged)(
                self as *const _ as *mut _,
                eventHandler,
                token,
            )
        }
    }

    pub(crate) unsafe fn get_DocumentTitle(&self, title: *mut *mut w_char) -> HRESULT {
        unsafe { ((*self.lpVtbl).get_DocumentTitle)(self as *const _ as *mut _, title) }
    }

    pub(crate) unsafe fn add_WebResourceRequested(
        &self,
        eventHandler: *mut ICoreWebView2WebResourceRequestedEventHandler,
        token: *mut u32,
    ) -> HRESULT {
        unsafe {
            ((*self.lpVtbl).add_WebResourceRequested)(
                self as *const _ as *mut _,
                eventHandler,
                token,
            )
        }
    }

    pub(crate) unsafe fn AddWebResourceRequestedFilter(
        &self,
        uri: *const w_char,
        resourceContext: u32,
    ) -> HRESULT {
        unsafe {
            ((*self.lpVtbl).AddWebResourceRequestedFilter)(
                self as *const _ as *mut _,
                uri,
                resourceContext,
            )
        }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2Vtbl {
    QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    get_Settings: unsafe extern "system" fn(
        This: *mut ICoreWebView2,
        settings: *mut *mut ICoreWebView2Settings,
    ) -> HRESULT,
    get_Source:
        unsafe extern "system" fn(This: *mut ICoreWebView2, uri: *mut *mut w_char) -> HRESULT,
    Navigate: unsafe extern "system" fn(This: *mut ICoreWebView2, uri: *const w_char) -> HRESULT,
    NavigateToString:
        unsafe extern "system" fn(This: *mut ICoreWebView2, html: *const w_char) -> HRESULT,
    add_NavigationStarting: unsafe extern "system" fn(
        This: *mut ICoreWebView2,
        eventHandler: *mut ICoreWebView2NavigationStartingEventHandler,
        token: *mut u32,
    ) -> HRESULT,
    padding2: [usize; 7],
    add_NavigationCompleted: unsafe extern "system" fn(
        This: *mut ICoreWebView2,
        eventHandler: *mut ICoreWebView2NavigationCompletedEventHandler,
        token: *mut u32,
    ) -> HRESULT,
    padding3: [usize; 11],
    AddScriptToExecuteOnDocumentCreated: unsafe extern "system" fn(
        This: *mut ICoreWebView2,
        script: *const w_char,
        resultHandler: *mut c_void,
    ) -> HRESULT,
    padding4: [usize; 1],
    ExecuteScript: unsafe extern "system" fn(
        This: *mut ICoreWebView2,
        script: *const w_char,
        resultHandler: *mut c_void,
    ) -> HRESULT,
    padding5: [usize; 4],
    add_WebMessageReceived: unsafe extern "system" fn(
        This: *mut ICoreWebView2,
        eventHandler: *mut ICoreWebView2WebMessageReceivedEventHandler,
        token: *mut u32,
    ) -> HRESULT,
    padding6: [usize; 9],
    add_NewWindowRequested: unsafe extern "system" fn(
        This: *mut ICoreWebView2,
        eventHandler: *mut ICoreWebView2NewWindowRequestedEventHandler,
        token: *mut u32,
    ) -> HRESULT,
    padding7: [usize; 1],
    add_DocumentTitleChanged: unsafe extern "system" fn(
        This: *mut ICoreWebView2,
        eventHandler: *mut ICoreWebView2DocumentTitleChangedEventHandler,
        token: *mut u32,
    ) -> HRESULT,
    padding8: [usize; 1],
    get_DocumentTitle:
        unsafe extern "system" fn(This: *mut ICoreWebView2, title: *mut *mut w_char) -> HRESULT,
    padding9: [usize; 6],
    add_WebResourceRequested: unsafe extern "system" fn(
        This: *mut ICoreWebView2,
        eventHandler: *mut ICoreWebView2WebResourceRequestedEventHandler,
        token: *mut u32,
    ) -> HRESULT,
    padding10: [usize; 1],
    AddWebResourceRequestedFilter: unsafe extern "system" fn(
        This: *mut ICoreWebView2,
        uri: *const w_char,
        resourceContext: u32,
    ) -> HRESULT,
}

// ICoreWebView2WebMessageReceivedEventHandler
#[repr(C)]
pub(crate) struct ICoreWebView2WebMessageReceivedEventHandler {
    pub(crate) lpVtbl: *const ICoreWebView2WebMessageReceivedEventHandlerVtbl,
}

#[repr(C)]
pub(crate) struct ICoreWebView2WebMessageReceivedEventHandlerVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Invoke: unsafe extern "system" fn(
        This: *mut ICoreWebView2WebMessageReceivedEventHandler,
        sender: *mut ICoreWebView2,
        args: *mut ICoreWebView2WebMessageReceivedEventArgs,
    ) -> HRESULT,
}

// ICoreWebView2WebMessageReceivedEventArgs
#[repr(C)]
pub(crate) struct ICoreWebView2WebMessageReceivedEventArgs {
    pub(crate) lpVtbl: *const ICoreWebView2WebMessageReceivedEventArgsVtbl,
}

impl ICoreWebView2WebMessageReceivedEventArgs {
    pub(crate) unsafe fn TryGetWebMessageAsString(&self, message: *mut *mut w_char) -> HRESULT {
        unsafe { ((*self.lpVtbl).TryGetWebMessageAsString)(self as *const _ as *mut _, message) }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2WebMessageReceivedEventArgsVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    padding: [usize; 2],
    pub(crate) TryGetWebMessageAsString: unsafe extern "system" fn(
        This: *mut ICoreWebView2WebMessageReceivedEventArgs,
        message: *mut *mut w_char,
    ) -> HRESULT,
}

// ICoreWebView2Settings
#[repr(C)]
pub(crate) struct ICoreWebView2Settings {
    pub(crate) lpVtbl: *const ICoreWebView2SettingsVtbl,
}

impl ICoreWebView2Settings {
    pub(crate) unsafe fn QueryInterface(
        &self,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT {
        unsafe { ((*self.lpVtbl).QueryInterface)(self as *const _ as *mut _, riid, ppvObject) }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2SettingsVtbl {
    QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
}

// ICoreWebView2Settings2
pub(crate) const IID_ICoreWebView2Settings2: GUID = GUID {
    data1: 0xee9a0f68,
    data2: 0xf46c,
    data3: 0x4e32,
    data4: [0xac, 0x23, 0xef, 0x8c, 0xac, 0x22, 0x4d, 0x2a],
};

#[repr(C)]
pub(crate) struct ICoreWebView2Settings2 {
    pub(crate) lpVtbl: *const ICoreWebView2Settings2Vtbl,
}

impl ICoreWebView2Settings2 {
    pub(crate) unsafe fn put_UserAgent(&self, userAgent: *const w_char) -> HRESULT {
        unsafe { ((*self.lpVtbl).put_UserAgent)(self as *const _ as *mut _, userAgent) }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2Settings2Vtbl {
    QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    padding: [usize; 19],
    put_UserAgent: unsafe extern "system" fn(
        This: *mut ICoreWebView2Settings2,
        userAgent: *const w_char,
    ) -> HRESULT,
}

// ICoreWebView2NavigationStartingEventHandler
#[repr(C)]
pub(crate) struct ICoreWebView2NavigationStartingEventHandler {
    pub(crate) lpVtbl: *const ICoreWebView2NavigationStartingEventHandlerVtbl,
}

#[repr(C)]
pub(crate) struct ICoreWebView2NavigationStartingEventHandlerVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Invoke: unsafe extern "system" fn(
        This: *mut ICoreWebView2NavigationStartingEventHandler,
        sender: *mut ICoreWebView2,
        args: *mut c_void,
    ) -> HRESULT,
}

// ICoreWebView2NavigationCompletedEventHandler
#[repr(C)]
pub(crate) struct ICoreWebView2NavigationCompletedEventHandler {
    pub(crate) lpVtbl: *const ICoreWebView2NavigationCompletedEventHandlerVtbl,
}

#[repr(C)]
pub(crate) struct ICoreWebView2NavigationCompletedEventHandlerVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Invoke: unsafe extern "system" fn(
        This: *mut ICoreWebView2NavigationCompletedEventHandler,
        sender: *mut ICoreWebView2,
        args: *mut c_void,
    ) -> HRESULT,
}

// ICoreWebView2DocumentTitleChangedEventHandler
#[repr(C)]
pub(crate) struct ICoreWebView2DocumentTitleChangedEventHandler {
    pub(crate) lpVtbl: *const ICoreWebView2DocumentTitleChangedEventHandlerVtbl,
}

#[repr(C)]
pub(crate) struct ICoreWebView2DocumentTitleChangedEventHandlerVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Invoke: unsafe extern "system" fn(
        This: *mut ICoreWebView2DocumentTitleChangedEventHandler,
        sender: *mut ICoreWebView2,
        args: *mut c_void,
    ) -> HRESULT,
}

// ICoreWebView2NewWindowRequestedEventHandlerVtbl
#[repr(C)]
pub(crate) struct ICoreWebView2NewWindowRequestedEventHandler {
    pub(crate) lpVtbl: *const ICoreWebView2NewWindowRequestedEventHandlerVtbl,
}

#[repr(C)]
pub(crate) struct ICoreWebView2NewWindowRequestedEventHandlerVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Invoke: unsafe extern "system" fn(
        This: *mut ICoreWebView2NewWindowRequestedEventHandler,
        sender: *mut ICoreWebView2,
        args: *mut ICoreWebView2NewWindowRequestedEventArgs,
    ) -> HRESULT,
}

// ICoreWebView2NewWindowRequestedEventArgs
#[repr(C)]
pub(crate) struct ICoreWebView2NewWindowRequestedEventArgs {
    pub(crate) lpVtbl: *const ICoreWebView2NewWindowRequestedEventArgsVtbl,
}

impl ICoreWebView2NewWindowRequestedEventArgs {
    pub(crate) unsafe fn get_Uri(&self, uri: *mut *mut w_char) -> HRESULT {
        unsafe { ((*self.lpVtbl).get_Uri)(self as *const _ as *mut _, uri) }
    }

    pub(crate) unsafe fn put_Handled(&self, handled: BOOL) -> HRESULT {
        unsafe { ((*self.lpVtbl).put_Handled)(self as *const _ as *mut _, handled) }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2NewWindowRequestedEventArgsVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) get_Uri: unsafe extern "system" fn(
        This: *mut ICoreWebView2NewWindowRequestedEventArgs,
        uri: *mut *mut w_char,
    ) -> HRESULT,
    padding: [usize; 2],
    pub(crate) put_Handled: unsafe extern "system" fn(
        This: *mut ICoreWebView2NewWindowRequestedEventArgs,
        handled: BOOL,
    ) -> HRESULT,
}

// ICoreWebView2WebResourceRequestedEventHandler
#[repr(C)]
pub(crate) struct ICoreWebView2WebResourceRequestedEventHandler {
    pub(crate) lpVtbl: *const ICoreWebView2WebResourceRequestedEventHandlerVtbl,
    pub(crate) user_data: *mut c_void,
}

#[repr(C)]
pub(crate) struct ICoreWebView2WebResourceRequestedEventHandlerVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Invoke: unsafe extern "system" fn(
        This: *mut ICoreWebView2WebResourceRequestedEventHandler,
        sender: *mut ICoreWebView2,
        args: *mut ICoreWebView2WebResourceRequestedEventArgs,
    ) -> HRESULT,
}

// ICoreWebView2WebResourceRequestedEventArgs
#[repr(C)]
pub(crate) struct ICoreWebView2WebResourceRequestedEventArgs {
    pub(crate) lpVtbl: *const ICoreWebView2WebResourceRequestedEventArgsVtbl,
}

impl ICoreWebView2WebResourceRequestedEventArgs {
    pub(crate) unsafe fn get_Request(
        &self,
        request: *mut *mut ICoreWebView2WebResourceRequest,
    ) -> HRESULT {
        unsafe { ((*self.lpVtbl).get_Request)(self as *const _ as *mut _, request) }
    }

    pub(crate) unsafe fn put_Response(
        &self,
        response: *mut ICoreWebView2WebResourceResponse,
    ) -> HRESULT {
        unsafe { ((*self.lpVtbl).put_Response)(self as *const _ as *mut _, response) }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2WebResourceRequestedEventArgsVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) get_Request: unsafe extern "system" fn(
        This: *mut ICoreWebView2WebResourceRequestedEventArgs,
        request: *mut *mut ICoreWebView2WebResourceRequest,
    ) -> HRESULT,
    padding: [usize; 1],
    pub(crate) put_Response: unsafe extern "system" fn(
        This: *mut ICoreWebView2WebResourceRequestedEventArgs,
        response: *mut ICoreWebView2WebResourceResponse,
    ) -> HRESULT,
}

// ICoreWebView2WebResourceRequest
#[repr(C)]
pub(crate) struct ICoreWebView2WebResourceRequest {
    pub(crate) lpVtbl: *const ICoreWebView2WebResourceRequestVtbl,
}

impl ICoreWebView2WebResourceRequest {
    pub(crate) unsafe fn Release(&self) -> HRESULT {
        unsafe { ((*self.lpVtbl).Release)(self as *const _ as *mut _) }
    }

    pub(crate) unsafe fn get_Uri(&self, uri: *mut *mut w_char) -> HRESULT {
        unsafe { ((*self.lpVtbl).get_Uri)(self as *const _ as *mut _, uri) }
    }

    pub(crate) unsafe fn get_Method(&self, method: *mut *mut w_char) -> HRESULT {
        unsafe { ((*self.lpVtbl).get_Method)(self as *const _ as *mut _, method) }
    }

    pub(crate) unsafe fn get_Content(&self, content: *mut *mut IStream) -> HRESULT {
        unsafe { ((*self.lpVtbl).get_Content)(self as *const _ as *mut _, content) }
    }

    pub(crate) unsafe fn get_Headers(
        &self,
        headers: *mut *mut ICoreWebView2HttpRequestHeaders,
    ) -> HRESULT {
        unsafe { ((*self.lpVtbl).get_Headers)(self as *const _ as *mut _, headers) }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2WebResourceRequestVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) get_Uri: unsafe extern "system" fn(
        This: *mut ICoreWebView2WebResourceRequest,
        uri: *mut *mut w_char,
    ) -> HRESULT,
    padding1: [usize; 1],
    pub(crate) get_Method: unsafe extern "system" fn(
        This: *mut ICoreWebView2WebResourceRequest,
        method: *mut *mut w_char,
    ) -> HRESULT,
    padding2: [usize; 1],
    get_Content: unsafe extern "system" fn(
        This: *mut ICoreWebView2WebResourceRequest,
        content: *mut *mut IStream,
    ) -> HRESULT,
    padding3: [usize; 1],
    pub(crate) get_Headers: unsafe extern "system" fn(
        This: *mut ICoreWebView2WebResourceRequest,
        headers: *mut *mut ICoreWebView2HttpRequestHeaders,
    ) -> HRESULT,
}

// ICoreWebView2HttpRequestHeaders
#[repr(C)]
pub(crate) struct ICoreWebView2HttpRequestHeaders {
    pub(crate) lpVtbl: *const ICoreWebView2HttpRequestHeadersVtbl,
}

impl ICoreWebView2HttpRequestHeaders {
    pub(crate) unsafe fn Release(&self) -> HRESULT {
        unsafe { ((*self.lpVtbl).Release)(self as *const _ as *mut _) }
    }

    pub(crate) unsafe fn GetIterator(
        &self,
        iterator: *mut *mut ICoreWebView2HttpRequestHeadersIterator,
    ) -> HRESULT {
        unsafe { ((*self.lpVtbl).GetIterator)(self as *const _ as *mut _, iterator) }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2HttpRequestHeadersVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    padding: [usize; 5],
    pub(crate) GetIterator: unsafe extern "system" fn(
        This: *mut ICoreWebView2HttpRequestHeaders,
        iterator: *mut *mut ICoreWebView2HttpRequestHeadersIterator,
    ) -> HRESULT,
}

// ICoreWebView2HttpRequestHeadersIterator
#[repr(C)]
pub(crate) struct ICoreWebView2HttpRequestHeadersIterator {
    pub(crate) lpVtbl: *const ICoreWebView2HttpRequestHeadersIteratorVtbl,
}

impl ICoreWebView2HttpRequestHeadersIterator {
    pub(crate) unsafe fn Release(&self) -> HRESULT {
        unsafe { ((*self.lpVtbl).Release)(self as *const _ as *mut _) }
    }

    pub(crate) unsafe fn GetCurrentHeader(
        &self,
        name: *mut *mut w_char,
        value: *mut *mut w_char,
    ) -> HRESULT {
        unsafe { ((*self.lpVtbl).GetCurrentHeader)(self as *const _ as *mut _, name, value) }
    }

    pub(crate) unsafe fn get_HasCurrentHeader(&self, hasCurrent: *mut BOOL) -> HRESULT {
        unsafe { ((*self.lpVtbl).get_HasCurrentHeader)(self as *const _ as *mut _, hasCurrent) }
    }

    pub(crate) unsafe fn MoveNext(&self, hasNext: *mut BOOL) -> HRESULT {
        unsafe { ((*self.lpVtbl).MoveNext)(self as *const _ as *mut _, hasNext) }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2HttpRequestHeadersIteratorVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) GetCurrentHeader: unsafe extern "system" fn(
        This: *mut ICoreWebView2HttpRequestHeadersIterator,
        name: *mut *mut w_char,
        value: *mut *mut w_char,
    ) -> HRESULT,
    pub(crate) get_HasCurrentHeader: unsafe extern "system" fn(
        This: *mut ICoreWebView2HttpRequestHeadersIterator,
        hasCurrent: *mut BOOL,
    ) -> HRESULT,
    pub(crate) MoveNext: unsafe extern "system" fn(
        This: *mut ICoreWebView2HttpRequestHeadersIterator,
        hasNext: *mut BOOL,
    ) -> HRESULT,
}

// ICoreWebView2WebResourceResponse
#[repr(C)]
pub(crate) struct ICoreWebView2WebResourceResponse {
    pub(crate) lpVtbl: *const ICoreWebView2WebResourceResponseVtbl,
}

impl ICoreWebView2WebResourceResponse {
    pub(crate) unsafe fn Release(&self) -> HRESULT {
        unsafe { ((*self.lpVtbl).Release)(self as *const _ as *mut _) }
    }
}

#[repr(C)]
pub(crate) struct ICoreWebView2WebResourceResponseVtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut c_void,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut c_void) -> HRESULT,
}
