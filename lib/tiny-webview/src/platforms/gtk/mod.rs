/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CStr, CString, c_char, c_void};
use std::path::Path;
use std::process::exit;
use std::ptr::{null, null_mut};
use std::{env, fs, iter};

use self::headers::*;
use crate::{Event, LogicalPoint, LogicalSize, WebviewBuilder};

mod headers;

// MARK: EventLoop
pub(crate) struct PlatformEventLoop;

static mut EVENT_HANDLER: Option<Box<dyn FnMut(Event) + 'static>> = None;

impl PlatformEventLoop {
    pub(crate) fn new() -> Self {
        // Init GTK
        unsafe {
            let args = env::args()
                .map(|arg| CString::new(arg.as_str()).expect("Can't convert to CString"))
                .collect::<Vec<CString>>();
            let mut argc = args.len() as i32;
            let mut argv: Vec<*mut c_char> = args
                .iter()
                .map(|arg| arg.as_ptr() as *mut c_char)
                .chain(iter::once(null_mut()))
                .collect();
            let mut argv_ptr = argv.as_mut_ptr();
            gtk_init(&mut argc, &mut argv_ptr);
        }

        Self
    }
}

impl crate::EventLoopInterface for PlatformEventLoop {
    fn run(self, event_handler: impl FnMut(Event) + 'static) -> ! {
        unsafe { EVENT_HANDLER = Some(Box::new(event_handler)) };

        // Start event loop
        unsafe { gtk_main() };
        exit(0);
    }

    fn create_proxy(&self) -> PlatformEventLoopProxy {
        PlatformEventLoopProxy::new()
    }
}

fn send_event(event: Event) {
    unsafe {
        #[allow(static_mut_refs)]
        if let Some(handler) = &mut EVENT_HANDLER {
            handler(event);
        }
    }
}

// MARK: PlatformEventLoopProxy
pub(crate) struct PlatformEventLoopProxy;

impl PlatformEventLoopProxy {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl crate::EventLoopProxyInterface for PlatformEventLoopProxy {
    fn send_user_event(&self, data: String) {
        send_event(Event::UserEvent(data));
    }
}

// MARK: Webview
struct WebviewData {
    window: *mut GtkWindow,
    webview: *mut WebKitWebView,
    #[cfg(feature = "remember_window_state")]
    remember_window_state: bool,
}

pub(crate) struct PlatformWebview(Box<WebviewData>);

impl PlatformWebview {
    pub(crate) fn new(builder: WebviewBuilder) -> Self {
        // Force dark mode if enabled
        if builder.should_force_dark_mode {
            unsafe {
                let settings = gtk_settings_get_default();
                g_object_set(
                    settings as *mut c_void,
                    c"gtk-application-prefer-dark-theme".as_ptr(),
                    1 as *const c_void,
                    null::<c_void>(),
                );
            }
        }

        let mut webview_data = Box::new(WebviewData {
            window: null_mut(),
            webview: null_mut(),
            #[cfg(feature = "remember_window_state")]
            remember_window_state: builder.remember_window_state,
        });

        // Create window
        let window = unsafe {
            let window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
            let title = CString::new(builder.title).expect("Can't convert to CString");
            gtk_window_set_title(window, title.as_ptr());
            gtk_window_set_default_size(
                window,
                builder.size.width as i32,
                builder.size.height as i32,
            );
            if let Some(min_size) = builder.min_size {
                gtk_widget_set_size_request(
                    window as *mut GtkWidget,
                    min_size.width as i32,
                    min_size.height as i32,
                );
            }
            gtk_window_set_resizable(window, builder.resizable);
            if builder.should_center {
                gtk_window_set_position(window, GTK_WIN_POS_CENTER);
            }
            #[cfg(feature = "remember_window_state")]
            if builder.remember_window_state {
                Self::load_window_state(window);
            }

            g_signal_connect_data(
                window as *mut c_void,
                c"destroy".as_ptr(),
                gtk_main_quit as *const c_void,
                null(),
                null(),
                G_CONNECT_DEFAULT,
            );
            let display = gdk_display_get_default();
            let display_name = CStr::from_ptr(gdk_display_get_name(display)).to_string_lossy();
            if !display_name.contains("wayland") {
                g_signal_connect_data(
                    window as *mut c_void,
                    c"configure-event".as_ptr(),
                    window_on_move as *const c_void,
                    webview_data.as_mut() as *mut _ as *const c_void,
                    null(),
                    G_CONNECT_DEFAULT,
                );
            }
            g_signal_connect_data(
                window as *mut c_void,
                c"size-allocate".as_ptr(),
                window_on_resize as *const c_void,
                webview_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window as *mut c_void,
                c"delete-event".as_ptr(),
                window_on_close as *const c_void,
                webview_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            window
        };

        // Create webview
        let webview = unsafe {
            let webview = if cfg!(feature = "ipc") {
                let user_content_controller = webkit_user_content_manager_new();
                let user_script = webkit_user_script_new(
                    c"window.ipc = new EventTarget();\
                            window.ipc.postMessage = message => window.webkit.messageHandlers.ipc.postMessage(typeof message !== 'string' ? JSON.stringify(message) : message);\
                            console.log = message => window.webkit.messageHandlers.console.postMessage(typeof message !== 'string' ? JSON.stringify(message) : message);".as_ptr(),
                    WEBKIT_USER_CONTENT_INJECT_TOP_FRAME,
                    WEBKIT_USER_SCRIPT_INJECT_AT_DOCUMENT_START,
                    null(),
                    null(),
                );
                webkit_user_content_manager_add_script(user_content_controller, user_script);
                g_signal_connect_data(
                    user_content_controller as *mut c_void,
                    c"script-message-received::ipc".as_ptr(),
                    webview_on_message_ipc as *const c_void,
                    webview_data.as_mut() as *mut _ as *const c_void,
                    null(),
                    G_CONNECT_DEFAULT,
                );
                g_signal_connect_data(
                    user_content_controller as *mut c_void,
                    c"script-message-received::console".as_ptr(),
                    webview_on_message_console as *const c_void,
                    webview_data.as_mut() as *mut _ as *const c_void,
                    null(),
                    G_CONNECT_DEFAULT,
                );
                webkit_user_content_manager_register_script_message_handler(
                    user_content_controller,
                    c"ipc".as_ptr(),
                );
                webkit_user_content_manager_register_script_message_handler(
                    user_content_controller,
                    c"console".as_ptr(),
                );

                webkit_web_view_new_with_user_content_manager(user_content_controller)
            } else {
                webkit_web_view_new()
            };
            gtk_container_add(window as *mut GtkWidget, webview as *mut GtkWidget);
            if cfg!(debug_assertions) {
                let webview_settings = webkit_web_view_get_settings(webview);
                webkit_settings_set_enable_developer_extras(webview_settings, true);
            }
            if let Some(should_load_url) = builder.should_load_url {
                let url = CString::new(should_load_url).expect("Can't convert to CString");
                webkit_web_view_load_uri(webview, url.as_ptr());
            }
            if let Some(should_load_html) = builder.should_load_html {
                let html = CString::new(should_load_html).expect("Can't convert to CString");
                webkit_web_view_load_html(webview, html.as_ptr(), null());
            }
            g_signal_connect_data(
                webview as *mut c_void,
                c"load-changed".as_ptr(),
                webview_on_load_changed as *const c_void,
                webview_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                webview as *mut c_void,
                c"decide-policy".as_ptr(),
                webview_on_navigation_policy_decision as *const c_void,
                webview_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            webview
        };

        // Show window
        unsafe { gtk_widget_show_all(window) };

        // Send window created event
        send_event(Event::WindowCreated);

        // Fill webview data and return
        webview_data.window = window;
        webview_data.webview = webview;
        Self(webview_data)
    }

    #[cfg(feature = "remember_window_state")]
    fn settings_path() -> String {
        let config_dir = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            format!(
                "{}/.config",
                env::var("HOME").expect("Can't read $HOME env variable")
            )
        });
        format!(
            "{}/{}/settings.ini",
            config_dir,
            env::current_exe()
                .expect("Can't get current process name")
                .file_name()
                .expect("Can't get current process name")
                .to_string_lossy()
        )
    }

    #[cfg(feature = "remember_window_state")]
    fn load_window_state(window: *mut GtkWindow) {
        unsafe {
            let settings = g_key_file_new();
            let file = CString::new(Self::settings_path()).expect("Can't convert to CString");
            let mut error = null_mut();
            g_key_file_load_from_file(settings, file.as_ptr(), 0, &mut error);
            if error.is_null() {
                let group = c"window".as_ptr();
                let x = g_key_file_get_integer(settings, group, c"x".as_ptr(), null_mut());
                let y = g_key_file_get_integer(settings, group, c"y".as_ptr(), null_mut());
                gtk_window_move(window, x, y);

                let width = g_key_file_get_integer(settings, group, c"width".as_ptr(), null_mut());
                let height =
                    g_key_file_get_integer(settings, group, c"height".as_ptr(), null_mut());
                gtk_window_set_default_size(window, width, height);

                let maximized =
                    g_key_file_get_boolean(settings, group, c"maximized".as_ptr(), null_mut());
                if maximized {
                    gtk_window_maximize(window);
                }
            } else {
                g_error_free(error);
            }
            g_key_file_free(settings);
        }
    }

    #[cfg(feature = "remember_window_state")]
    fn save_window_state(window: *mut GtkWindow) {
        let settings_path = Self::settings_path();
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
            gtk_window_get_position(window, &mut x, &mut y);
            g_key_file_set_integer(settings, group, c"x".as_ptr(), x);
            g_key_file_set_integer(settings, group, c"y".as_ptr(), y);

            let mut width = 0;
            let mut height = 0;
            gtk_window_get_size(window, &mut width, &mut height);
            g_key_file_set_integer(settings, group, c"width".as_ptr(), width);
            g_key_file_set_integer(settings, group, c"height".as_ptr(), height);

            let maximized = gtk_window_is_maximized(window);
            g_key_file_set_boolean(settings, group, c"maximized".as_ptr(), maximized);

            let file = CString::new(settings_path).expect("Can't convert to CString");
            g_key_file_save_to_file(settings, file.as_ptr(), null_mut());
            g_key_file_free(settings);
        }
    }
}

impl crate::WebviewInterface for PlatformWebview {
    fn set_title(&mut self, title: impl AsRef<str>) {
        let title = CString::new(title.as_ref()).expect("Can't convert to CString");
        unsafe { gtk_window_set_title(self.0.window, title.as_ptr()) }
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
        unsafe { gtk_window_move(self.0.window, point.x as i32, point.y as i32) }
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

    fn load_url(&mut self, url: impl AsRef<str>) {
        let url = CString::new(url.as_ref()).expect("Can't convert to CString");
        unsafe { webkit_web_view_load_uri(self.0.webview, url.as_ptr()) }
    }

    fn load_html(&mut self, html: impl AsRef<str>) {
        let html = CString::new(html.as_ref()).expect("Can't convert to CString");
        unsafe { webkit_web_view_load_html(self.0.webview, html.as_ptr(), null()) }
    }

    fn evaluate_script(&mut self, script: impl AsRef<str>) {
        let script = script.as_ref();
        unsafe {
            webkit_web_view_evaluate_javascript(
                self.0.webview,
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
}

extern "C" fn window_on_move(
    _window: *mut GtkWindow,
    _allocation: *mut c_void,
    _self: &mut WebviewData,
) -> bool {
    let mut x = 0;
    let mut y = 0;
    unsafe { gtk_window_get_position(_self.window, &mut x, &mut y) };
    send_event(Event::WindowMoved(LogicalPoint::new(x as f32, y as f32)));
    false
}

extern "C" fn window_on_resize(
    _window: *mut GtkWindow,
    _allocation: *mut c_void,
    _self: &mut WebviewData,
) {
    let mut width = 0;
    let mut height = 0;
    unsafe { gtk_window_get_size(_self.window, &mut width, &mut height) };
    send_event(Event::WindowResized(LogicalSize::new(
        width as f32,
        height as f32,
    )));
}

extern "C" fn window_on_close(
    _window: *mut GtkWindow,
    _event: *mut c_void,
    _self: &mut WebviewData,
) -> bool {
    // Save window state
    #[cfg(feature = "remember_window_state")]
    if _self.remember_window_state {
        PlatformWebview::save_window_state(_self.window);
    }

    // Send window closed event
    send_event(Event::WindowClosed);
    false
}

extern "C" fn webview_on_load_changed(
    _webview: *mut WebKitWebView,
    event: i32,
    _self: &mut WebviewData,
) {
    if event == WEBKIT_LOAD_STARTED {
        send_event(Event::PageLoadStarted)
    }
    if event == WEBKIT_LOAD_FINISHED {
        send_event(Event::PageLoadFinished)
    }
}

extern "C" fn webview_on_navigation_policy_decision(
    _webview: *mut WebKitWebView,
    decision: *mut WebKitNavigationPolicyDecision,
    decision_type: i32,
    _self: &mut WebviewData,
) -> bool {
    if decision_type == WEBKIT_POLICY_DECISION_TYPE_NEW_WINDOW_ACTION {
        let request = unsafe { webkit_navigation_policy_decision_get_request(decision) };
        let uri = unsafe { webkit_uri_request_get_uri(request) };
        unsafe { gtk_show_uri_on_window(null_mut(), uri, 0, null_mut()) };
        return true;
    }
    false
}

#[cfg(feature = "ipc")]
extern "C" fn webview_on_message_ipc(
    _manager: *mut WebKitUserContentManager,
    _message: *mut WebKitJavascriptResult,
    _self: &mut WebviewData,
) {
    let message = unsafe { webkit_javascript_result_get_js_value(_message) };
    let message = unsafe { jsc_value_to_string(message) };
    let message = unsafe { CStr::from_ptr(message) }.to_string_lossy();
    send_event(Event::PageMessageReceived(message.to_string()));
}

#[cfg(feature = "ipc")]
extern "C" fn webview_on_message_console(
    _manager: *mut WebKitUserContentManager,
    _message: *mut WebKitJavascriptResult,
    _self: &mut WebviewData,
) {
    let message = unsafe { webkit_javascript_result_get_js_value(_message) };
    let message = unsafe { jsc_value_to_string(message) };
    let message = unsafe { CStr::from_ptr(message) }.to_string_lossy();
    println!("{}", message);
}
