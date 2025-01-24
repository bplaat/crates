/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{c_char, c_void, CString};
use std::path::Path;
use std::process::exit;
use std::ptr::{null, null_mut};
use std::{env, fs, iter};

use crate::{Event, LogicalPoint, LogicalSize, WebviewBuilder};

/// Webview
pub(crate) struct Webview {
    builder: Option<WebviewBuilder>,
    app: *mut GtkApplication,
    window: *mut GtkWindow,
    webview: *mut WebKitWebView,
    event_handler: Option<fn(&mut Webview, Event)>,
    #[cfg(feature = "remember_window_state")]
    remember_window_state: bool,
}

impl Webview {
    pub(crate) fn new(builder: WebviewBuilder) -> Self {
        #[cfg(feature = "remember_window_state")]
        let remember_window_state = builder.remember_window_state;
        Self {
            builder: Some(builder),
            app: unsafe { gtk_application_new(null_mut(), G_APPLICATION_DEFAULT_FLAGS) },
            window: null_mut(),
            webview: null_mut(),
            event_handler: None,
            #[cfg(feature = "remember_window_state")]
            remember_window_state,
        }
    }

    #[cfg(feature = "remember_window_state")]
    fn settings_path(&self) -> String {
        format!(
            "{}/.config/{}/settings.ini",
            env::var("HOME").expect("Can't read HOME env var"),
            env::current_exe()
                .expect("Can't get current process name")
                .file_name()
                .expect("Can't get current process name")
                .to_string_lossy()
        )
    }

    #[cfg(feature = "remember_window_state")]
    fn load_window_state(&mut self) {
        unsafe {
            let settings = g_key_file_new();
            let file = CString::new(self.settings_path()).expect("Can't convert to CString");
            let mut error = null_mut();
            g_key_file_load_from_file(settings, file.as_ptr(), 0, &mut error);
            if error.is_null() {
                let group = c"window".as_ptr();
                let x = g_key_file_get_integer(settings, group, c"x".as_ptr(), null_mut());
                let y = g_key_file_get_integer(settings, group, c"y".as_ptr(), null_mut());
                gtk_window_move(self.window, x, y);

                let width = g_key_file_get_integer(settings, group, c"width".as_ptr(), null_mut());
                let height =
                    g_key_file_get_integer(settings, group, c"height".as_ptr(), null_mut());
                gtk_window_set_default_size(self.window, width, height);

                let maximized =
                    g_key_file_get_boolean(settings, group, c"maximized".as_ptr(), null_mut());
                if maximized {
                    gtk_window_maximize(self.window);
                }
            } else {
                g_error_free(error);
            }
            g_key_file_free(settings);
        }
    }

    #[cfg(feature = "remember_window_state")]
    fn save_window_state(&mut self) {
        let settings_path = self.settings_path();
        fs::create_dir_all(
            Path::new(&settings_path)
                .parent()
                .expect("Can't create settings directory"),
        )
        .expect("Can't create settings directory");

        unsafe {
            let settings = g_key_file_new();
            let group = c"window".as_ptr();

            let mut x = 0;
            let mut y = 0;
            gtk_window_get_position(self.window, &mut x, &mut y);
            g_key_file_set_integer(settings, group, c"x".as_ptr(), x);
            g_key_file_set_integer(settings, group, c"y".as_ptr(), y);

            let mut width = 0;
            let mut height = 0;
            gtk_window_get_size(self.window, &mut width, &mut height);
            g_key_file_set_integer(settings, group, c"width".as_ptr(), width);
            g_key_file_set_integer(settings, group, c"height".as_ptr(), height);

            let maximized = gtk_window_is_maximized(self.window);
            g_key_file_set_boolean(settings, group, c"maximized".as_ptr(), maximized);

            let file = CString::new(settings_path).expect("Can't convert to CString");
            g_key_file_save_to_file(settings, file.as_ptr(), null_mut());
            g_key_file_free(settings);
        }
    }

    fn send_event(&mut self, event: Event) {
        self.event_handler.expect("Should be some")(self, event);
    }
}

impl crate::Webview for Webview {
    fn run(&mut self, event_handler: fn(&mut Webview, Event)) -> ! {
        self.event_handler = Some(event_handler);
        unsafe {
            g_signal_connect_data(
                self.app as *mut Self as *mut c_void,
                c"activate".as_ptr(),
                app_on_activate as *const c_void,
                self as *mut Self as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
        }

        // Start event loop
        let args = env::args()
            .map(|arg| CString::new(arg.as_str()).expect("Can't convert to CString"))
            .collect::<Vec<CString>>();
        let argv = args
            .iter()
            .map(|arg| arg.as_ptr())
            .chain(iter::once(null()))
            .collect::<Vec<*const c_char>>();
        exit(unsafe {
            g_application_run(
                self.app as *mut GApplication,
                argv.len() as i32,
                argv.as_ptr(),
            )
        });
    }

    fn set_title(&mut self, title: impl AsRef<str>) {
        let title = CString::new(title.as_ref()).expect("Can't convert to CString");
        unsafe { gtk_window_set_title(self.window, title.as_ptr()) }
    }

    fn position(&self) -> LogicalPoint {
        let mut x = 0;
        let mut y = 0;
        unsafe { gtk_window_get_position(self.window, &mut x, &mut y) };
        LogicalPoint::new(x as f32, y as f32)
    }

    fn size(&self) -> LogicalSize {
        let mut width = 0;
        let mut height = 0;
        unsafe { gtk_window_get_size(self.window, &mut width, &mut height) };
        LogicalSize::new(width as f32, height as f32)
    }

    fn set_position(&mut self, point: LogicalPoint) {
        unsafe { gtk_window_move(self.window, point.x as i32, point.y as i32) }
    }

    fn set_size(&mut self, size: LogicalSize) {
        unsafe { gtk_window_set_default_size(self.window, size.width as i32, size.height as i32) }
    }

    fn set_min_size(&mut self, min_size: LogicalSize) {
        unsafe {
            gtk_widget_set_size_request(
                self.window as *mut GtkWidget,
                min_size.width as i32,
                min_size.height as i32,
            )
        }
    }

    fn set_resizable(&mut self, resizable: bool) {
        unsafe { gtk_window_set_resizable(self.window, resizable) }
    }

    fn load_url(&mut self, url: impl AsRef<str>) {
        let url = CString::new(url.as_ref()).expect("Can't convert to CString");
        unsafe { webkit_web_view_load_uri(self.webview, url.as_ptr()) }
    }

    fn load_html(&mut self, html: impl AsRef<str>) {
        let html = CString::new(html.as_ref()).expect("Can't convert to CString");
        unsafe { webkit_web_view_load_html(self.webview, html.as_ptr(), null()) }
    }

    fn evaluate_script(&mut self, script: impl AsRef<str>) {
        let script = script.as_ref();
        unsafe {
            webkit_web_view_evaluate_javascript(
                self.webview,
                script.as_ptr() as *const c_char,
                script.len(),
                null(),
                null(),
                null(),
                null(),
                null(),
            )
        }
    }

    #[cfg(feature = "ipc")]
    fn send_ipc_message(&mut self, message: impl AsRef<str>) {
        self.evaluate_script(format!(
            "window.ipc.dispatchEvent(new MessageEvent('message',{{data:`{}`}}));",
            message.as_ref()
        ));
    }
}

extern "C" fn app_on_activate(app: *mut GApplication, _self: &mut Webview) {
    let builder = _self.builder.take().expect("Should be some");

    // Create window
    unsafe {
        _self.window = gtk_application_window_new(app as *mut GtkApplication);
        let title = CString::new(builder.title).expect("Can't convert to CString");
        gtk_window_set_title(_self.window, title.as_ptr());
        gtk_window_set_default_size(
            _self.window,
            builder.size.width as i32,
            builder.size.height as i32,
        );
        if let Some(min_size) = builder.min_size {
            gtk_widget_set_size_request(
                _self.window as *mut GtkWidget,
                min_size.width as i32,
                min_size.height as i32,
            );
        }
        gtk_window_set_resizable(_self.window, builder.resizable);
        if builder.should_center {
            gtk_window_set_position(_self.window, GTK_WIN_POS_CENTER);
        }
        #[cfg(feature = "remember_window_state")]
        if builder.remember_window_state {
            _self.load_window_state();
        }

        g_signal_connect_data(
            _self.window as *mut c_void,
            c"configure-event".as_ptr(),
            window_on_move as *const c_void,
            _self as *mut Webview as *const c_void,
            null(),
            G_CONNECT_DEFAULT,
        );
        g_signal_connect_data(
            _self.window as *mut c_void,
            c"size-allocate".as_ptr(),
            window_on_resize as *const c_void,
            _self as *mut Webview as *const c_void,
            null(),
            G_CONNECT_DEFAULT,
        );
        g_signal_connect_data(
            _self.window as *mut c_void,
            c"delete-event".as_ptr(),
            window_on_close as *const c_void,
            _self as *mut Webview as *const c_void,
            null(),
            G_CONNECT_DEFAULT,
        );
    }

    // Create webview
    unsafe {
        #[cfg(feature = "ipc")]
        {
            let user_content_controller = webkit_user_content_manager_new();
            let user_script = webkit_user_script_new(
                c"window.ipc=new EventTarget();window.ipc.postMessage=message=>window.webkit.messageHandlers.ipc.postMessage(message);".as_ptr(),
                WEBKIT_USER_CONTENT_INJECT_TOP_FRAME,
                WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START,
                null(),
                null(),
            );
            webkit_user_content_manager_add_script(user_content_controller, user_script);
            g_signal_connect_data(
                user_content_controller as *mut c_void,
                c"script-message-received".as_ptr(),
                webview_on_message as *const c_void,
                _self as *mut Webview as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            webkit_user_content_manager_register_script_message_handler(
                user_content_controller,
                c"ipc".as_ptr(),
            );
            _self.webview = webkit_web_view_new_with_user_content_manager(user_content_controller);
        }
        #[cfg(not(feature = "ipc"))]
        {
            _self.webview = webkit_web_view_new();
        }
        gtk_container_add(
            _self.window as *mut GtkWidget,
            _self.webview as *mut GtkWidget,
        );
        if let Some(should_load_url) = builder.should_load_url {
            let url = CString::new(should_load_url).expect("Can't convert to CString");
            webkit_web_view_load_uri(_self.webview, url.as_ptr());
        }
        if let Some(should_load_html) = builder.should_load_html {
            let html = CString::new(should_load_html).expect("Can't convert to CString");
            webkit_web_view_load_html(_self.webview, html.as_ptr(), null());
        }
        g_signal_connect_data(
            _self.webview as *mut c_void,
            c"load-changed".as_ptr(),
            webview_on_load_changed as *const c_void,
            _self as *mut Webview as *const c_void,
            null(),
            G_CONNECT_DEFAULT,
        );
    }

    // Show window
    unsafe { gtk_widget_show_all(_self.window) };

    // Send window created event
    _self.send_event(Event::WindowCreated);
}

extern "C" fn window_on_move(
    _window: *mut GtkWindow,
    _allocation: *mut c_void,
    _self: &mut Webview,
) -> bool {
    let mut x = 0;
    let mut y = 0;
    unsafe { gtk_window_get_position(_self.window, &mut x, &mut y) };
    _self.send_event(Event::WindowMoved(LogicalPoint::new(x as f32, y as f32)));
    false
}

extern "C" fn window_on_resize(
    _window: *mut GtkWindow,
    _allocation: *mut c_void,
    _self: &mut Webview,
) {
    let mut width = 0;
    let mut height = 0;
    unsafe { gtk_window_get_size(_self.window, &mut width, &mut height) };
    _self.send_event(Event::WindowResized(LogicalSize::new(
        width as f32,
        height as f32,
    )));
}

extern "C" fn window_on_close(
    _window: *mut GtkWindow,
    _event: *mut c_void,
    _self: &mut Webview,
) -> bool {
    // Save window state
    #[cfg(feature = "remember_window_state")]
    if _self.remember_window_state {
        _self.save_window_state();
    }

    // Send window closed event
    _self.send_event(Event::WindowClosed);
    false
}

extern "C" fn webview_on_load_changed(
    _webview: *mut WebKitWebView,
    event: i32,
    _self: &mut Webview,
) {
    if event == WEBKIT_LOAD_STARTED {
        _self.send_event(Event::PageLoadStarted)
    }
    if event == WEBKIT_LOAD_FINISHED {
        _self.send_event(Event::PageLoadFinished)
    }
}

#[cfg(feature = "ipc")]
extern "C" fn webview_on_message(
    _manager: *mut WebKitUserContentManager,
    _message: *mut WebKitJavascriptResult,
    _self: &mut Webview,
) {
    let message = unsafe { webkit_javascript_result_get_js_value(_message) };
    let message = unsafe { jsc_value_to_string(message) };
    let message = unsafe { std::ffi::CStr::from_ptr(message) }.to_string_lossy();
    _self.send_event(Event::PageMessageReceived(message.to_string()));
}

// MARK: GTK headers
// GObject
const G_CONNECT_DEFAULT: i32 = 0;
#[link(name = "gobject-2.0")]
extern "C" {
    fn g_signal_connect_data(
        instance: *mut c_void,
        detailed_signal: *const c_char,
        c_handler: *const c_void,
        data: *const c_void,
        destroy_data: *const c_void,
        connect_flags: i32,
    );
}

// Glib
#[repr(C)]
struct GError(u8);
#[repr(C)]
struct GKeyFile(u8);
#[link(name = "glib-2.0")]
extern "C" {
    fn g_error_free(error: *mut GError);
    fn g_key_file_new() -> *mut GKeyFile;
    fn g_key_file_load_from_file(
        key_file: *mut GKeyFile,
        file: *const c_char,
        flags: i32,
        error: *mut *mut GError,
    ) -> bool;
    fn g_key_file_get_integer(
        key_file: *mut GKeyFile,
        group_name: *const c_char,
        key: *const c_char,
        error: *mut *mut c_void,
    ) -> i32;
    fn g_key_file_get_boolean(
        key_file: *mut GKeyFile,
        group_name: *const c_char,
        key: *const c_char,
        error: *mut *mut c_void,
    ) -> bool;
    fn g_key_file_set_integer(
        key_file: *mut GKeyFile,
        group_name: *const c_char,
        key: *const c_char,
        value: i32,
    );
    fn g_key_file_set_boolean(
        key_file: *mut GKeyFile,
        group_name: *const c_char,
        key: *const c_char,
        value: bool,
    );
    fn g_key_file_save_to_file(
        key_file: *mut GKeyFile,
        file: *const c_char,
        error: *mut *mut c_void,
    ) -> bool;
    fn g_key_file_free(key_file: *mut GKeyFile);
}

// GIO
#[repr(C)]
struct GApplication(u8);
const G_APPLICATION_DEFAULT_FLAGS: i32 = 0;
#[link(name = "gio-2.0")]
extern "C" {
    fn g_application_run(
        application: *mut GApplication,
        argc: i32,
        argv: *const *const c_char,
    ) -> i32;
}

// GTK
#[repr(C)]
struct GtkApplication(u8);
#[repr(C)]
struct GtkWidget(u8);
#[repr(C)]
struct GtkWindow(u8);
const GTK_WIN_POS_CENTER: i32 = 1;
#[link(name = "gtk-3")]
extern "C" {
    fn gtk_application_new(application_id: *const c_char, flags: i32) -> *mut GtkApplication;
    fn gtk_application_window_new(app: *mut GtkApplication) -> *mut GtkWindow;
    fn gtk_widget_set_size_request(widget: *mut GtkWidget, width: i32, height: i32);
    fn gtk_container_add(container: *mut GtkWidget, widget: *mut GtkWidget);
    fn gtk_window_get_position(window: *mut GtkWindow, x: *mut i32, y: *mut i32);
    fn gtk_window_set_title(window: *mut GtkWindow, title: *const c_char);
    fn gtk_window_set_position(window: *mut GtkWindow, position: i32);
    fn gtk_window_move(window: *mut GtkWindow, x: i32, y: i32);
    fn gtk_window_get_size(window: *mut GtkWindow, width: *mut i32, height: *mut i32);
    fn gtk_window_set_default_size(window: *mut GtkWindow, width: i32, height: i32);
    fn gtk_window_set_resizable(window: *mut GtkWindow, resizable: bool);
    fn gtk_window_is_maximized(window: *mut GtkWindow) -> bool;
    fn gtk_window_maximize(window: *mut GtkWindow);
    fn gtk_widget_show_all(window: *mut GtkWindow);
}

// WebKitGtk
#[repr(C)]
struct WebKitWebView(u8);
const WEBKIT_LOAD_STARTED: i32 = 1;
const WEBKIT_LOAD_FINISHED: i32 = 3;
#[link(name = "webkit2gtk-4.1")]
extern "C" {
    #[cfg(not(feature = "ipc"))]
    fn webkit_web_view_new() -> *mut GtkWidget;
    fn webkit_web_view_load_uri(web_view: *mut WebKitWebView, uri: *const c_char);
    fn webkit_web_view_load_html(
        web_view: *mut WebKitWebView,
        content: *const c_char,
        base_uri: *const c_char,
    );
    fn webkit_web_view_evaluate_javascript(
        web_view: *mut WebKitWebView,
        script: *const c_char,
        length: usize,
        world_name: *const c_char,
        source_uri: *const c_char,
        cancellable: *const c_void,
        callback: *const c_void,
        user_data: *const c_void,
    );
}

#[repr(C)]
struct WebKitUserContentManager(u8);
#[repr(C)]
struct WebKitUserScript(u8);
#[repr(C)]
struct WebKitJavascriptResult(u8);
const WEBKIT_USER_CONTENT_INJECT_TOP_FRAME: i32 = 1;
const WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START: i32 = 0;
extern "C" {
    fn webkit_user_content_manager_new() -> *mut WebKitUserContentManager;
    fn webkit_user_script_new(
        source: *const c_char,
        injected_frames: i32,
        injection_time: i32,
        whitelist: *const *const c_char,
        blacklist: *const *const c_char,
    ) -> *mut WebKitUserScript;
    fn webkit_user_content_manager_add_script(
        manager: *mut WebKitUserContentManager,
        script: *mut WebKitUserScript,
    );
    fn webkit_user_content_manager_register_script_message_handler(
        manager: *mut WebKitUserContentManager,
        name: *const c_char,
    );
    fn webkit_web_view_new_with_user_content_manager(
        manager: *mut WebKitUserContentManager,
    ) -> *mut WebKitWebView;
    fn webkit_javascript_result_get_js_value(result: *mut WebKitJavascriptResult) -> *mut c_void;
}

#[link(name = "javascriptcoregtk-4.1")]
extern "C" {
    fn jsc_value_to_string(value: *mut c_void) -> *const c_char;
}
