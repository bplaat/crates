/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::ptr::null;

use objc2::rc::{Allocated, Retained, autoreleasepool};
use objc2::runtime::{AnyClass, AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};

use super::cocoa::*;
use super::menu::create_menu_bar;
use super::webkit::*;
use crate::{CloseRequest, Event, EventLoopBuilder, LogicalPoint, LogicalSize, Theme, WindowEvent};

thread_local! {
    static PENDING_EVENTS: RefCell<Vec<Event>> = const { RefCell::new(Vec::new()) };
}

type EventHandler = Box<dyn FnMut(Event) + 'static>;

struct QueuedEventIvars {
    event: RefCell<Option<Event>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = QueuedEventIvars]
    struct QueuedEvent;
);

// MARK: AppDelegate
struct AppDelegateIvars {
    event_handler: RefCell<Option<EventHandler>>,
    allow_termination: Cell<bool>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = AppDelegateIvars]
    struct AppDelegate;

    impl AppDelegate {
        #[unsafe(method_id(init))]
        fn _init(this: Allocated<Self>) -> Option<Retained<Self>> {
            unsafe {
                msg_send![
                    super(this.set_ivars(AppDelegateIvars {
                        event_handler: RefCell::new(None),
                        allow_termination: Cell::new(false),
                    })),
                    init
                ]
            }
        }

        #[unsafe(method(applicationDidFinishLaunching:))]
        fn _did_finish_launching(&self, notification: *mut Object) { self.did_finish_launching(notification); }

        #[unsafe(method(applicationShouldTerminateAfterLastWindowClosed:))]
        const fn _should_terminate(&self, _: *mut Object) -> Bool { Bool::YES }

        #[unsafe(method(applicationShouldTerminate:))]
        fn _application_should_terminate(&self, _: *mut Object) -> u64 { self.application_should_terminate() }

        #[unsafe(method(application:openURLs:))]
        fn _open_urls(&self, _: *mut Object, urls: *mut Object) { self.open_urls(urls); }

        #[unsafe(method(sendEvent:))]
        fn _send_event(&self, value: &QueuedEvent) { self.send_event(value); }

        #[unsafe(method(openAboutDialog:))]
        fn _open_about_dialog(&self, _: *mut Object) { self.open_about_dialog(); }

        #[cfg(feature = "menu")]
        #[unsafe(method(menuItemSelected:))]
        fn _menu_item_selected(&self, sender: *mut Object) { self.menu_item_selected(sender); }
    }
);

impl AppDelegate {
    fn application_should_terminate(&self) -> u64 {
        if self.ivars().allow_termination.get() {
            1
        } else {
            let request = CloseRequest::new();
            send_event(Event::Window(WindowEvent::CloseRequested(request.clone())));
            u64::from(!request.is_prevented())
        }
    }

    fn did_finish_launching(&self, notification: *mut Object) {
        unsafe {
            let application: *mut Object = msg_send![notification, object];
            let _: Bool = msg_send![application, setActivationPolicy:NS_APPLICATION_ACTIVATION_POLICY_REGULAR];
            let _: () = msg_send![application, activateIgnoringOtherApps:Bool::YES];

            let windows: *mut Object = msg_send![application, windows];
            let windows_count: usize = msg_send![windows, count];
            for i in 0..windows_count {
                let window: *mut Object = msg_send![windows, objectAtIndex:i];
                let _: () = msg_send![window, makeKeyAndOrderFront:null::<Object>()];
                send_event(Event::Window(WindowEvent::Create));
            }
        }
    }

    fn send_event(&self, value: &QueuedEvent) {
        let Some(event) = value.ivars().event.borrow_mut().take() else {
            return;
        };
        send_event(event);
    }

    fn open_urls(&self, urls: *mut Object) {
        let mut paths = Vec::new();
        unsafe {
            let count: usize = msg_send![urls, count];
            for index in 0..count {
                let url: *mut Object = msg_send![urls, objectAtIndex:index];
                let is_file_url: Bool = msg_send![url, isFileURL];
                if is_file_url == Bool::YES {
                    let path: NSString = msg_send![url, path];
                    paths.push(PathBuf::from(path.to_string()));
                }
            }
        }
        if !paths.is_empty() {
            send_event(Event::MacosOpenFiles(paths));
        }
    }

    fn open_about_dialog(&self) {
        let _: () = unsafe { msg_send![NSApp, orderFrontStandardAboutPanel:null::<Object>()] };
    }

    #[cfg(feature = "menu")]
    fn menu_item_selected(&self, sender: *mut Object) {
        let action: NSString = unsafe { msg_send![sender, representedObject] };
        send_event(Event::MacosMenuItem(action.to_string()));
    }
}

// MARK: EventLoop
pub(crate) struct PlatformEventLoop {
    application: Retained<Object>,
    delegate: Retained<AppDelegate>,
    theme: Theme,
}

impl PlatformEventLoop {
    pub(crate) fn new(mut builder: EventLoopBuilder) -> Self {
        // Create AppDelegate instance (registers class lazily on first call)
        let app_delegate: Retained<AppDelegate> = unsafe { msg_send![AppDelegate::class(), new] };

        // Get application
        let application = unsafe {
            let application: Retained<Object> = msg_send![class!(NSApplication), sharedApplication];
            let _: () = msg_send![&application, setDelegate:&*app_delegate];
            application
        };

        // Create menu
        unsafe {
            create_menu_bar(
                application.as_ptr(),
                app_delegate.as_ptr().cast::<Object>(),
                &mut builder,
            );
        }

        Self {
            application,
            delegate: app_delegate,
            theme: system_theme(),
        }
    }
}

impl Drop for PlatformEventLoop {
    fn drop(&mut self) {
        // Menu targets and NSApplication.delegate are non-owning. Clear the application-owned
        // references before our retained delegate is dropped.
        let current: *mut Object = unsafe { msg_send![&self.application, delegate] };
        if current == self.delegate.as_ptr().cast::<Object>() {
            unsafe {
                let _: () =
                    msg_send![&self.application, setMainMenu:std::ptr::null_mut::<Object>()];
                let _: () =
                    msg_send![&self.application, setWindowsMenu:std::ptr::null_mut::<Object>()];
                let _: () =
                    msg_send![&self.application, setHelpMenu:std::ptr::null_mut::<Object>()];
                let _: () =
                    msg_send![&self.application, setServicesMenu:std::ptr::null_mut::<Object>()];
                let _: () =
                    msg_send![&self.application, setDelegate:std::ptr::null_mut::<Object>()];
            }
        }
    }
}

// MARK: Theme
fn system_theme() -> Theme {
    unsafe {
        let application: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let appearance: *mut Object = msg_send![application, effectiveAppearance];
        let name: NSString = msg_send![appearance, name];
        if name.to_string().contains("Dark") {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

impl crate::EventLoopInterface for PlatformEventLoop {
    fn theme(&self) -> Theme {
        self.theme
    }

    fn primary_monitor(&self) -> PlatformMonitor {
        unsafe {
            let screen: *mut Object = msg_send![class!(NSScreen), mainScreen];
            PlatformMonitor::new(screen)
        }
    }

    fn available_monitors(&self) -> Vec<PlatformMonitor> {
        let mut monitors = Vec::new();
        unsafe {
            let screens: *mut Object = msg_send![class!(NSScreen), screens];
            let count: usize = msg_send![screens, count];
            for i in 0..count {
                let screen: *mut Object = msg_send![screens, objectAtIndex:i];
                monitors.push(PlatformMonitor::new(screen));
            }
        }
        monitors
    }

    fn run(self, event_handler: impl FnMut(Event) + 'static) -> ! {
        *self.delegate.ivars().event_handler.borrow_mut() = Some(Box::new(event_handler));
        autoreleasepool(|_| unsafe {
            let pending = PENDING_EVENTS.with(|events| std::mem::take(&mut *events.borrow_mut()));
            for event in pending {
                send_event(event);
            }
            let _: () = msg_send![&self.application, run];
        });
        unreachable!()
    }

    fn create_proxy(&self) -> PlatformEventLoopProxy {
        PlatformEventLoopProxy::new(self.delegate.clone())
    }
}

pub(crate) fn send_event(event: Event) {
    let app_delegate = unsafe {
        let app_delegate: *mut Object = msg_send![NSApp, delegate];
        if app_delegate.is_null() {
            PENDING_EVENTS.with(|events| events.borrow_mut().push(event));
            return;
        }
        let app_delegate_class = AppDelegate::class().cast::<AnyClass>();
        let is_ours: Bool = msg_send![app_delegate, isKindOfClass:app_delegate_class];
        if is_ours == Bool::NO {
            PENDING_EVENTS.with(|events| events.borrow_mut().push(event));
            return;
        }
        &*(app_delegate as *const AppDelegate)
    };
    let Ok(mut event_handler) = app_delegate.ivars().event_handler.try_borrow_mut() else {
        PENDING_EVENTS.with(|events| events.borrow_mut().push(event));
        return;
    };
    if let Some(handler) = event_handler.as_mut() {
        handler(event);
    } else {
        PENDING_EVENTS.with(|events| events.borrow_mut().push(event));
        return;
    }
    drop(event_handler);
    let pending = PENDING_EVENTS.with(|events| std::mem::take(&mut *events.borrow_mut()));
    for event in pending {
        send_event(event);
    }
}

pub(super) fn allow_termination_if_last_window(closing_window: *mut Object) {
    let app_delegate: *mut Object = unsafe { msg_send![NSApp, delegate] };
    if app_delegate.is_null() {
        return;
    }
    let app_delegate_class = AppDelegate::class().cast::<AnyClass>();
    let is_ours: Bool = unsafe { msg_send![app_delegate, isKindOfClass:app_delegate_class] };
    if is_ours == Bool::NO {
        return;
    }
    let app_delegate = unsafe { &*(app_delegate as *const AppDelegate) };
    let windows: *mut Object = unsafe { msg_send![NSApp, windows] };
    let count: usize = unsafe { msg_send![windows, count] };
    let has_other_visible_window = (0..count).any(|index| unsafe {
        let window: *mut Object = msg_send![windows, objectAtIndex:index];
        let visible: Bool = msg_send![window, isVisible];
        window != closing_window && visible == Bool::YES
    });
    if !has_other_visible_window {
        app_delegate.ivars().allow_termination.set(true);
    }
}

// MARK: EventLoopProxy
pub(crate) struct PlatformEventLoopProxy {
    delegate: Retained<AppDelegate>,
}

// SAFETY: The proxy only sends performSelectorOnMainThread: to its retained delegate. Objective-C
// retain/release and that scheduling API are thread-safe; delegate ivars are only read on main.
unsafe impl Send for PlatformEventLoopProxy {}
// SAFETY: Sending through a shared proxy has the same thread-safe behavior as an owned proxy.
unsafe impl Sync for PlatformEventLoopProxy {}

impl PlatformEventLoopProxy {
    const fn new(delegate: Retained<AppDelegate>) -> Self {
        Self { delegate }
    }
}

impl crate::EventLoopProxyInterface for PlatformEventLoopProxy {
    fn send_user_event(&self, data: String) {
        unsafe {
            let value: Allocated<QueuedEvent> = msg_send![QueuedEvent::class(), alloc];
            let value: Retained<QueuedEvent> = msg_send![
                super(value.set_ivars(QueuedEventIvars {
                    event: RefCell::new(Some(Event::UserEvent(data))),
                })),
                init
            ];
            let _: () = msg_send![&*self.delegate, performSelectorOnMainThread:sel!(sendEvent:),
                       withObject:&*value,
                    waitUntilDone:Bool::NO];
        }
    }
}

// MARK: Monitor
pub(crate) struct PlatformMonitor {
    pub(crate) screen: Retained<Object>,
}

impl PlatformMonitor {
    pub(crate) fn new(screen: *mut Object) -> Self {
        Self {
            // SAFETY: NSScreen APIs returned a live object which is retained for this handle.
            screen: unsafe { Retained::retain(screen) }.expect("NSScreen returned null"),
        }
    }
}

impl crate::MonitorInterface for PlatformMonitor {
    fn name(&self) -> String {
        let name: NSString = unsafe { msg_send![&self.screen, localizedName] };
        name.to_string()
    }

    fn position(&self) -> LogicalPoint {
        let frame: NSRect = unsafe { msg_send![&self.screen, frame] };
        LogicalPoint::new(frame.origin.x as f32, frame.origin.y as f32)
    }

    fn size(&self) -> LogicalSize {
        let frame: NSRect = unsafe { msg_send![&self.screen, frame] };
        LogicalSize::new(frame.size.width as f32, frame.size.height as f32)
    }

    fn scale_factor(&self) -> f32 {
        let backing_scale_factor: f64 = unsafe { msg_send![&self.screen, backingScaleFactor] };
        backing_scale_factor as f32
    }

    fn is_primary(&self) -> bool {
        let main_screen: *mut Object = unsafe { msg_send![class!(NSScreen), mainScreen] };
        self.screen.as_ptr() == main_screen
    }
}
