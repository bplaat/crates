/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CStr, CString, c_char, c_void};
use std::fs::File;
use std::mem::MaybeUninit;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::process::exit;
use std::ptr::{null, null_mut};
use std::{env, fs, iter};

use self::headers::*;
use crate::{Event, EventLoopBuilder, LogicalPoint, LogicalSize, WebviewBuilder};

mod headers;

// MARK: EventLoop
pub(crate) struct PlatformEventLoop;

static mut EVENT_HANDLER: Option<Box<dyn FnMut(Event) + 'static>> = None;

impl PlatformEventLoop {
    pub(crate) fn new(builder: EventLoopBuilder) -> Self {
        // Ensure single instance
        if let Some(app_id) = builder.app_id {
            let lock_file = env::temp_dir().join(app_id).join(".lock");
            if let Some(parent) = lock_file.parent() {
                fs::create_dir_all(parent).expect("Failed to create lock file directory");
            }
            let file = File::create(&lock_file).expect("Failed to open lock file");
            if unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) } != 0 {
                exit(0);
            }
            std::mem::forget(file);
        }

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
    fn primary_monitor(&self) -> PlatformMonitor {
        unsafe {
            let mut primary_monitor = gdk_display_get_primary_monitor(gdk_display_get_default());
            if primary_monitor.is_null() {
                primary_monitor = gdk_display_get_monitor(gdk_display_get_default(), 0);
            }
            PlatformMonitor::new(primary_monitor)
        }
    }

    fn available_monitors(&self) -> Vec<PlatformMonitor> {
        unsafe {
            let display = gdk_display_get_default();
            let mut monitors = Vec::new();
            for i in 0..gdk_display_get_n_monitors(display) {
                monitors.push(PlatformMonitor::new(gdk_display_get_monitor(display, i)));
            }
            monitors
        }
    }

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
        let ptr = Box::leak(Box::new(Event::UserEvent(data))) as *mut Event as *mut c_void;
        unsafe { g_idle_add(send_event_callback, ptr) };
    }
}

extern "C" fn send_event_callback(ptr: *mut c_void) -> i32 {
    let event = unsafe { Box::from_raw(ptr as *mut Event) };
    send_event(*event);
    0
}

// MARK: PlatformMonitor
pub(crate) struct PlatformMonitor {
    monitor: *mut GdkMonitor,
}

impl PlatformMonitor {
    pub(crate) fn new(monitor: *mut GdkMonitor) -> Self {
        Self { monitor }
    }
}

impl crate::MonitorInterface for PlatformMonitor {
    fn name(&self) -> String {
        unsafe {
            let name_ptr = gdk_monitor_get_model(self.monitor);
            CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
        }
    }

    fn position(&self) -> LogicalPoint {
        let mut rect = MaybeUninit::<GdkRectangle>::uninit();
        unsafe {
            gdk_monitor_get_geometry(self.monitor, rect.as_mut_ptr());
        }
        let rect = unsafe { rect.assume_init() };

        // The GTK monitors are not offset by primary monitor position,
        // so we need to calculate the position relative to the primary monitor.
        let primary_monitor_rect = unsafe {
            let mut primary_monitor = gdk_display_get_primary_monitor(gdk_display_get_default());
            if primary_monitor.is_null() {
                primary_monitor = gdk_display_get_monitor(gdk_display_get_default(), 0);
            }
            let mut primary_monitor_rect = MaybeUninit::<GdkRectangle>::uninit();
            gdk_monitor_get_geometry(primary_monitor, primary_monitor_rect.as_mut_ptr());
            primary_monitor_rect.assume_init()
        };
        LogicalPoint::new(
            (rect.x - primary_monitor_rect.x) as f32,
            (rect.y - primary_monitor_rect.y) as f32,
        )
    }

    fn size(&self) -> LogicalSize {
        let mut rect = MaybeUninit::<GdkRectangle>::uninit();
        unsafe {
            gdk_monitor_get_geometry(self.monitor, rect.as_mut_ptr());
        }
        let rect = unsafe { rect.assume_init() };
        LogicalSize::new(rect.width as f32, rect.height as f32)
    }

    fn scale_factor(&self) -> f32 {
        unsafe { gdk_monitor_get_scale_factor(self.monitor) as f32 }
    }

    fn is_primary(&self) -> bool {
        unsafe { gdk_monitor_is_primary(self.monitor) }
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
        let is_wayland = unsafe {
            CStr::from_ptr(gdk_display_get_name(gdk_display_get_default()))
                .to_string_lossy()
                .contains("wayland")
        };

        // Force dark mode if enabled
        if let Some(theme) = builder.theme {
            unsafe {
                let settings = gtk_settings_get_default();
                g_object_set(
                    settings as *mut GObject,
                    c"gtk-application-prefer-dark-theme".as_ptr(),
                    if theme == crate::Theme::Dark { 1 } else { 0 } as *const c_void,
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
            gtk_window_set_resizable(window, builder.resizable);
            if let Some(min_size) = builder.min_size {
                gtk_widget_set_size_request(
                    window as *mut GtkWidget,
                    min_size.width as i32,
                    min_size.height as i32,
                );
            }
            if let Some(color) = builder.background_color {
                let rgba = GdkRGBA {
                    red: ((color >> 16) & 0xFF) as f64 / 255.0,
                    green: ((color >> 8) & 0xFF) as f64 / 255.0,
                    blue: (color & 0xFF) as f64 / 255.0,
                    alpha: 1.0,
                };
                gtk_widget_override_background_color(
                    window as *mut GtkWidget,
                    GTK_STATE_FLAG_NORMAL,
                    &rgba,
                );
            }

            let monitor_rect = if let Some(monitor) = builder.monitor {
                let mut rect = MaybeUninit::<GdkRectangle>::uninit();
                gdk_monitor_get_geometry(monitor.monitor, rect.as_mut_ptr());
                rect.assume_init()
            } else {
                let mut primary_monitor =
                    gdk_display_get_primary_monitor(gdk_display_get_default());
                if primary_monitor.is_null() {
                    primary_monitor = gdk_display_get_monitor(gdk_display_get_default(), 0);
                }
                let mut rect = MaybeUninit::<GdkRectangle>::uninit();
                gdk_monitor_get_geometry(primary_monitor, rect.as_mut_ptr());
                rect.assume_init()
            };
            if let Some(position) = builder.position {
                gtk_window_move(
                    window,
                    position.x as i32 + monitor_rect.x,
                    position.y as i32 + monitor_rect.y,
                );
            }
            if builder.should_fullscreen {
                gtk_window_move(window, monitor_rect.x, monitor_rect.y);
                gtk_window_fullscreen(window);
            }
            if builder.should_center {
                if !is_wayland {
                    gtk_window_move(
                        window,
                        monitor_rect.x + (monitor_rect.width - builder.size.width as i32) / 2,
                        monitor_rect.y + (monitor_rect.height - builder.size.height as i32) / 2,
                    );
                } else {
                    gtk_window_set_position(window, GTK_WIN_POS_CENTER);
                }
            }
            #[cfg(feature = "remember_window_state")]
            if builder.remember_window_state {
                Self::load_window_state(window);
            }

            g_signal_connect_data(
                window as *mut GObject,
                c"destroy".as_ptr(),
                gtk_main_quit as *const c_void,
                null(),
                null(),
                G_CONNECT_DEFAULT,
            );
            if !is_wayland {
                g_signal_connect_data(
                    window as *mut GObject,
                    c"configure-event".as_ptr(),
                    window_on_move as *const c_void,
                    webview_data.as_mut() as *mut _ as *const c_void,
                    null(),
                    G_CONNECT_DEFAULT,
                );
            }
            g_signal_connect_data(
                window as *mut GObject,
                c"size-allocate".as_ptr(),
                window_on_resize as *const c_void,
                webview_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window as *mut GObject,
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
                user_content_controller as *mut GObject,
                c"script-message-received::ipc".as_ptr(),
                webview_on_message_ipc as *const c_void,
                webview_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                user_content_controller as *mut GObject,
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
            let webview = webkit_web_view_new_with_user_content_manager(user_content_controller);
            gtk_container_add(window as *mut GtkWidget, webview as *mut GtkWidget);
            if builder.background_color.is_some() {
                let rgba = GdkRGBA {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 0.0,
                };
                webkit_web_view_set_background_color(webview, &rgba);
            }

            let useragent = CString::new(format!(
                "Mozilla/5.0 ({}; {} {}) bwebview/{}",
                if is_wayland { "Wayland" } else { "X11" },
                if env::consts::OS == "linux" {
                    "Linux"
                } else {
                    env::consts::OS
                },
                env::consts::ARCH,
                env!("CARGO_PKG_VERSION"),
            ))
            .expect("Can't convert to CString");
            let webview_settings = webkit_web_view_get_settings(webview);
            webkit_settings_set_user_agent(webview_settings, useragent.as_ptr());
            if cfg!(debug_assertions) {
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
                webview as *mut GObject,
                c"load-changed".as_ptr(),
                webview_on_load_changed as *const c_void,
                webview_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                webview as *mut GObject,
                c"notify::title".as_ptr(),
                webview_on_title_changed as *const c_void,
                webview_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                webview as *mut GObject,
                c"decide-policy".as_ptr(),
                webview_on_navigation_policy_decision as *const c_void,
                webview_data.as_mut() as *mut _ as *const c_void,
                null(),
                G_CONNECT_DEFAULT,
            );
            webview
        };

        // Fill webview data
        webview_data.window = window;
        webview_data.webview = webview;

        // Show window
        unsafe { gtk_widget_show_all(webview_data.window) };

        // Send window created event
        send_event(Event::WindowCreated);

        Self(webview_data)
    }

    #[cfg(feature = "remember_window_state")]
    fn settings_path() -> String {
        let config_dir = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            format!(
                "{}/.config",
                env::home_dir().expect("Can't find home dir").display()
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
            let mut err = null_mut();
            g_key_file_load_from_file(settings, file.as_ptr(), 0, &mut err);
            if err.is_null() {
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
                g_error_free(err);
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
        let primary_monitor_rect = unsafe {
            let mut primary_monitor = gdk_display_get_primary_monitor(gdk_display_get_default());
            if primary_monitor.is_null() {
                primary_monitor = gdk_display_get_monitor(gdk_display_get_default(), 0);
            }
            let mut primary_monitor_rect = MaybeUninit::<GdkRectangle>::uninit();
            gdk_monitor_get_geometry(primary_monitor, primary_monitor_rect.as_mut_ptr());
            primary_monitor_rect.assume_init()
        };
        unsafe {
            gtk_window_move(
                self.0.window,
                point.x as i32 + primary_monitor_rect.x,
                point.y as i32 + primary_monitor_rect.y,
            )
        }
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

    fn set_theme(&mut self, theme: crate::Theme) {
        unsafe {
            let settings = gtk_settings_get_default();
            g_object_set(
                settings as *mut GObject,
                c"gtk-application-prefer-dark-theme".as_ptr(),
                if theme == crate::Theme::Dark { 1 } else { 0 } as *const c_void,
                null::<c_void>(),
            );
        }
    }

    fn set_background_color(&mut self, color: u32) {
        unsafe {
            let rgba = GdkRGBA {
                red: ((color >> 16) & 0xFF) as f64 / 255.0,
                green: ((color >> 8) & 0xFF) as f64 / 255.0,
                blue: (color & 0xFF) as f64 / 255.0,
                alpha: 1.0,
            };
            gtk_widget_override_background_color(
                self.0.window as *mut GtkWidget,
                GTK_STATE_FLAG_NORMAL,
                &rgba,
            );
        }
    }

    fn url(&self) -> Option<String> {
        unsafe {
            let url = webkit_web_view_get_uri(self.0.webview);
            if !url.is_null() {
                Some(CStr::from_ptr(url).to_string_lossy().into_owned())
            } else {
                None
            }
        }
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

extern "C" fn webview_on_title_changed(
    webview: *mut WebKitWebView,
    _pspec: *const c_void,
    _self: &mut WebviewData,
) {
    let title = unsafe { webkit_web_view_get_title(webview) };
    let title = unsafe { CStr::from_ptr(title) }.to_string_lossy();
    send_event(Event::PageTitleChanged(title.to_string()));
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

extern "C" fn webview_on_message_console(
    _manager: *mut WebKitUserContentManager,
    _message: *mut WebKitJavascriptResult,
    _self: &mut WebviewData,
) {
    let message = unsafe { webkit_javascript_result_get_js_value(_message) };
    let message = unsafe { jsc_value_to_string(message) };
    let message = unsafe { CStr::from_ptr(message) }.to_string_lossy();
    println!("{message}");
}
