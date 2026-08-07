/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::Cell;
use std::ffi::c_void;
use std::path::PathBuf;
use std::ptr::null;

use objc2::rc::autoreleasepool;
use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};

use super::cocoa::*;
use super::webkit::*;
use crate::{
    Accelerator, CloseRequest, Event, EventLoopBuilder, KeyCode, LogicalPoint, LogicalSize,
    MenuBuilderEntry, Modifiers, Theme, WindowEvent,
};

// MARK: AppDelegate
struct AppDelegateIvars {
    event_loop: Cell<*mut PlatformEventLoop>,
    allow_termination: Cell<bool>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = AppDelegateIvars]
    struct AppDelegate;

    impl AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn _did_finish_launching(&self, notification: *mut Object) { self.did_finish_launching(notification); }

        #[unsafe(method(applicationShouldTerminateAfterLastWindowClosed:))]
        const fn _should_terminate(&self, _: *mut Object) -> Bool { Bool::YES }

        #[unsafe(method(applicationShouldTerminate:))]
        fn _application_should_terminate(&self, _: *mut Object) -> u64 { self.application_should_terminate() }

        #[unsafe(method(application:openURLs:))]
        fn _open_urls(&self, _: *mut Object, urls: *mut Object) { self.open_urls(urls); }

        #[unsafe(method(sendEvent:))]
        fn _send_event(&self, value: *mut Object) { self.send_event(value); }

        #[unsafe(method(openAboutDialog:))]
        fn _open_about_dialog(&self, _: *mut Object) { self.open_about_dialog(); }

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
            let _: () = msg_send![application, activateIgnoringOtherApps:true];

            let windows: *mut Object = msg_send![application, windows];
            let windows_count: usize = msg_send![windows, count];
            for i in 0..windows_count {
                let window: *mut Object = msg_send![windows, objectAtIndex:i];
                let _: () = msg_send![window, makeKeyAndOrderFront:null::<Object>()];
                send_event(Event::Window(WindowEvent::Create));
            }
        }
    }

    fn send_event(&self, value: *mut Object) {
        let ptr: *mut c_void = unsafe { msg_send![value, pointerValue] };
        let event = unsafe { Box::from_raw(ptr as *mut Event) };
        send_event(*event);
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

    fn menu_item_selected(&self, sender: *mut Object) {
        let action: NSString = unsafe { msg_send![sender, representedObject] };
        send_event(Event::MacosMenuItem(action.to_string()));
    }
}

// MARK: Menu
impl Modifiers {
    /// NSEventModifierFlags mask for these modifiers
    const fn ns_modifier_mask(self) -> u64 {
        let mut mask = 0;
        if self.contains(Modifiers::COMMAND) {
            mask |= NS_EVENT_MODIFIER_FLAG_COMMAND;
        }
        if self.contains(Modifiers::CONTROL) {
            mask |= NS_EVENT_MODIFIER_FLAG_CONTROL;
        }
        if self.contains(Modifiers::OPTION) {
            mask |= NS_EVENT_MODIFIER_FLAG_OPTION;
        }
        if self.contains(Modifiers::SHIFT) {
            mask |= NS_EVENT_MODIFIER_FLAG_SHIFT;
        }
        mask
    }
}

impl KeyCode {
    /// NSMenuItem key equivalent string for this key
    const fn ns_key_equivalent(self) -> &'static str {
        match self {
            KeyCode::KeyA => "a",
            KeyCode::KeyB => "b",
            KeyCode::KeyC => "c",
            KeyCode::KeyD => "d",
            KeyCode::KeyE => "e",
            KeyCode::KeyF => "f",
            KeyCode::KeyG => "g",
            KeyCode::KeyH => "h",
            KeyCode::KeyI => "i",
            KeyCode::KeyJ => "j",
            KeyCode::KeyK => "k",
            KeyCode::KeyL => "l",
            KeyCode::KeyM => "m",
            KeyCode::KeyN => "n",
            KeyCode::KeyO => "o",
            KeyCode::KeyP => "p",
            KeyCode::KeyQ => "q",
            KeyCode::KeyR => "r",
            KeyCode::KeyS => "s",
            KeyCode::KeyT => "t",
            KeyCode::KeyU => "u",
            KeyCode::KeyV => "v",
            KeyCode::KeyW => "w",
            KeyCode::KeyX => "x",
            KeyCode::KeyY => "y",
            KeyCode::KeyZ => "z",
            KeyCode::Digit0 => "0",
            KeyCode::Digit1 => "1",
            KeyCode::Digit2 => "2",
            KeyCode::Digit3 => "3",
            KeyCode::Digit4 => "4",
            KeyCode::Digit5 => "5",
            KeyCode::Digit6 => "6",
            KeyCode::Digit7 => "7",
            KeyCode::Digit8 => "8",
            KeyCode::Digit9 => "9",
            KeyCode::F1 => "\u{f704}",
            KeyCode::F2 => "\u{f705}",
            KeyCode::F3 => "\u{f706}",
            KeyCode::F4 => "\u{f707}",
            KeyCode::F5 => "\u{f708}",
            KeyCode::F6 => "\u{f709}",
            KeyCode::F7 => "\u{f70a}",
            KeyCode::F8 => "\u{f70b}",
            KeyCode::F9 => "\u{f70c}",
            KeyCode::F10 => "\u{f70d}",
            KeyCode::F11 => "\u{f70e}",
            KeyCode::F12 => "\u{f70f}",
            KeyCode::F13 => "\u{f710}",
            KeyCode::F14 => "\u{f711}",
            KeyCode::F15 => "\u{f712}",
            KeyCode::F16 => "\u{f713}",
            KeyCode::F17 => "\u{f714}",
            KeyCode::F18 => "\u{f715}",
            KeyCode::F19 => "\u{f716}",
            KeyCode::F20 => "\u{f717}",
            KeyCode::F21 => "\u{f718}",
            KeyCode::F22 => "\u{f719}",
            KeyCode::F23 => "\u{f71a}",
            KeyCode::F24 => "\u{f71b}",
            KeyCode::Backquote => "`",
            KeyCode::Backslash => "\\",
            KeyCode::BracketLeft => "[",
            KeyCode::BracketRight => "]",
            KeyCode::Comma => ",",
            KeyCode::Equal => "=",
            KeyCode::Minus => "-",
            KeyCode::Period => ".",
            KeyCode::Quote => "'",
            KeyCode::Semicolon => ";",
            KeyCode::Slash => "/",
            KeyCode::Space => " ",
            KeyCode::Tab => "\t",
            // The backspace key reports NSDeleteCharacter, NSBackspaceCharacter is never matched
            KeyCode::Backspace => "\u{7f}",
            KeyCode::Delete => "\u{f728}",
            KeyCode::Enter => "\r",
            KeyCode::Escape => "\u{1b}",
            KeyCode::ArrowUp => "\u{f700}",
            KeyCode::ArrowDown => "\u{f701}",
            KeyCode::ArrowLeft => "\u{f702}",
            KeyCode::ArrowRight => "\u{f703}",
            KeyCode::Home => "\u{f729}",
            KeyCode::End => "\u{f72b}",
            KeyCode::PageUp => "\u{f72c}",
            KeyCode::PageDown => "\u{f72d}",
        }
    }
}

#[derive(Clone, Copy)]
enum MenuRole {
    Application,
    Normal,
    Window,
    Help,
}

enum MenuItemAction {
    About,
    Hide,
    HideOthers,
    ShowAll,
    Terminate,
    Close,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
    Minimize,
    Zoom,
    Custom(String),
}

struct NativeMenuItem {
    title: String,
    action: MenuItemAction,
    accelerator: Option<Accelerator>,
}

impl NativeMenuItem {
    fn new(title: impl Into<String>, action: MenuItemAction) -> Self {
        Self {
            title: title.into(),
            action,
            accelerator: None,
        }
    }

    const fn accelerator(mut self, accelerator: Accelerator) -> Self {
        self.accelerator = Some(accelerator);
        self
    }

    /// Shorthand for the Command + key accelerators that most default items use
    const fn command(self, key: KeyCode) -> Self {
        self.accelerator(Accelerator::new(Modifiers::COMMAND, key))
    }

    /// Creates the NSMenuItem carrying this item's title, accelerator and target action
    unsafe fn create_native(self, app_delegate: *mut Object) -> *mut Object {
        let native_item: *mut Object = unsafe { msg_send![class!(NSMenuItem), new] };
        let _: () = unsafe { msg_send![native_item, setTitle:NSString::from_str(self.title)] };
        if let Some(accelerator) = self.accelerator {
            let _: () = unsafe {
                msg_send![native_item, setKeyEquivalent:NSString::from_str(accelerator.key.ns_key_equivalent())]
            };
            let _: () = unsafe {
                msg_send![native_item, setKeyEquivalentModifierMask:accelerator.modifiers.ns_modifier_mask()]
            };
        }
        // Items left without a target travel the responder chain to whatever view has focus
        match self.action {
            MenuItemAction::About => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(openAboutDialog:)] };
                let _: () = unsafe { msg_send![native_item, setTarget:app_delegate] };
            }
            MenuItemAction::Hide => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(hide:)] };
            }
            MenuItemAction::HideOthers => {
                let _: () =
                    unsafe { msg_send![native_item, setAction:sel!(hideOtherApplications:)] };
            }
            MenuItemAction::ShowAll => {
                let _: () =
                    unsafe { msg_send![native_item, setAction:sel!(unhideAllApplications:)] };
            }
            MenuItemAction::Terminate => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(terminate:)] };
            }
            MenuItemAction::Close => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(performClose:)] };
            }
            MenuItemAction::Undo => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(undo:)] };
            }
            MenuItemAction::Redo => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(redo:)] };
            }
            MenuItemAction::Cut => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(cut:)] };
            }
            MenuItemAction::Copy => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(copy:)] };
            }
            MenuItemAction::Paste => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(paste:)] };
            }
            MenuItemAction::Delete => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(delete:)] };
            }
            MenuItemAction::SelectAll => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(selectAll:)] };
            }
            MenuItemAction::Minimize => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(performMiniaturize:)] };
            }
            MenuItemAction::Zoom => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(performZoom:)] };
            }
            MenuItemAction::Custom(action) => {
                let _: () = unsafe { msg_send![native_item, setAction:sel!(menuItemSelected:)] };
                let _: () = unsafe { msg_send![native_item, setTarget:app_delegate] };
                let _: () = unsafe {
                    msg_send![native_item, setRepresentedObject:NSString::from_str(action)]
                };
            }
        }
        native_item
    }
}

enum MenuEntry {
    Item(NativeMenuItem),
    Separator,
    /// Submenu that macOS populates with the available Services
    ServicesMenu(String),
}

impl MenuEntry {
    /// Appends this entry to a native NSMenu
    unsafe fn add_to(
        self,
        native_menu: *mut Object,
        application: *mut Object,
        app_delegate: *mut Object,
    ) {
        let native_item: *mut Object = match self {
            MenuEntry::Separator => unsafe { msg_send![class!(NSMenuItem), separatorItem] },
            MenuEntry::ServicesMenu(title) => unsafe {
                let native_item: *mut Object = msg_send![class!(NSMenuItem), new];
                let _: () = msg_send![native_item, setTitle:NSString::from_str(title)];
                let services_menu: *mut Object = msg_send![class!(NSMenu), new];
                let _: () = msg_send![native_item, setSubmenu:services_menu];
                let _: () = msg_send![application, setServicesMenu:services_menu];
                native_item
            },
            MenuEntry::Item(item) => unsafe { item.create_native(app_delegate) },
        };
        let _: () = unsafe { msg_send![native_menu, addItem:native_item] };
    }
}

struct Menu {
    title: String,
    role: MenuRole,
    entries: Vec<MenuEntry>,
}

impl Menu {
    fn new(title: impl Into<String>, role: MenuRole) -> Self {
        Self {
            title: title.into(),
            role,
            entries: Vec::new(),
        }
    }

    fn item(mut self, item: NativeMenuItem) -> Self {
        self.entries.push(MenuEntry::Item(item));
        self
    }

    fn separator(mut self) -> Self {
        self.entries.push(MenuEntry::Separator);
        self
    }

    fn services_menu(mut self, title: impl Into<String>) -> Self {
        self.entries.push(MenuEntry::ServicesMenu(title.into()));
        self
    }

    /// Creates the menu bar entry that owns this menu's populated NSMenu
    unsafe fn create_native(
        self,
        application: *mut Object,
        app_delegate: *mut Object,
    ) -> *mut Object {
        let menu_item: *mut Object = unsafe { msg_send![class!(NSMenuItem), new] };
        // The application menu takes its title from the process name, so never set one
        if !matches!(self.role, MenuRole::Application) {
            let _: () = unsafe { msg_send![menu_item, setTitle:NSString::from_str(self.title)] };
        }

        let native_menu: *mut Object = unsafe { msg_send![class!(NSMenu), new] };
        let _: () = unsafe { msg_send![menu_item, setSubmenu:native_menu] };
        match self.role {
            MenuRole::Window => {
                let _: () = unsafe { msg_send![application, setWindowsMenu:native_menu] };
            }
            MenuRole::Help => {
                let _: () = unsafe { msg_send![application, setHelpMenu:native_menu] };
            }
            MenuRole::Application | MenuRole::Normal => {}
        }

        for entry in self.entries {
            unsafe { entry.add_to(native_menu, application, app_delegate) };
        }
        menu_item
    }
}

/// The whole macOS menu bar: bwebview's defaults with the app's own menus folded in
struct MenuBar(Vec<Menu>);

impl MenuBar {
    /// The complete menu bar every bwebview app gets before its own menus are merged in
    fn with_defaults(app_name: &str) -> Self {
        Self(vec![
            Menu::new(app_name, MenuRole::Application)
                .item(NativeMenuItem::new(
                    format!("About {app_name}"),
                    MenuItemAction::About,
                ))
                .separator()
                .services_menu("Services")
                .separator()
                .item(
                    NativeMenuItem::new(format!("Hide {app_name}"), MenuItemAction::Hide)
                        .command(KeyCode::KeyH),
                )
                .item(
                    NativeMenuItem::new("Hide Others", MenuItemAction::HideOthers).accelerator(
                        Accelerator::new(Modifiers::COMMAND | Modifiers::OPTION, KeyCode::KeyH),
                    ),
                )
                .item(NativeMenuItem::new("Show All", MenuItemAction::ShowAll))
                .separator()
                .item(
                    NativeMenuItem::new(format!("Quit {app_name}"), MenuItemAction::Terminate)
                        .command(KeyCode::KeyQ),
                ),
            Menu::new("File", MenuRole::Normal).item(
                NativeMenuItem::new("Close Window", MenuItemAction::Close).command(KeyCode::KeyW),
            ),
            Menu::new("Edit", MenuRole::Normal)
                .item(NativeMenuItem::new("Undo", MenuItemAction::Undo).command(KeyCode::KeyZ))
                .item(
                    NativeMenuItem::new("Redo", MenuItemAction::Redo).accelerator(
                        Accelerator::new(Modifiers::COMMAND | Modifiers::SHIFT, KeyCode::KeyZ),
                    ),
                )
                .separator()
                .item(NativeMenuItem::new("Cut", MenuItemAction::Cut).command(KeyCode::KeyX))
                .item(NativeMenuItem::new("Copy", MenuItemAction::Copy).command(KeyCode::KeyC))
                .item(NativeMenuItem::new("Paste", MenuItemAction::Paste).command(KeyCode::KeyV))
                .item(NativeMenuItem::new("Delete", MenuItemAction::Delete))
                .item(
                    NativeMenuItem::new("Select All", MenuItemAction::SelectAll)
                        .command(KeyCode::KeyA),
                ),
            Menu::new("Window", MenuRole::Window)
                .item(
                    NativeMenuItem::new("Minimize", MenuItemAction::Minimize)
                        .command(KeyCode::KeyM),
                )
                .item(NativeMenuItem::new("Zoom", MenuItemAction::Zoom)),
            Menu::new("Help", MenuRole::Help),
        ])
    }

    /// Drops an accelerator everywhere it is already used, so app shortcuts beat the defaults
    fn clear_shortcut(&mut self, accelerator: Accelerator) {
        for menu in &mut self.0 {
            for entry in &mut menu.entries {
                if let MenuEntry::Item(item) = entry
                    && item.accelerator == Some(accelerator)
                {
                    item.accelerator = None;
                }
            }
        }
    }

    /// Folds the app's menus into the defaults. Menus and items matched by title are merged and
    /// overridden in place, anything new is inserted ahead of the trailing system menus.
    fn merge(mut self, menu_bar: Option<crate::MenuBarBuilder>) -> Self {
        let Some(menu_bar) = menu_bar else {
            return self;
        };

        for custom_menu in menu_bar.menus {
            let menu_index = if let Some(index) = self
                .0
                .iter()
                .position(|menu| menu.title == custom_menu.title)
            {
                index
            } else {
                let index = self
                    .0
                    .iter()
                    .position(|menu| matches!(menu.role, MenuRole::Window | MenuRole::Help))
                    .unwrap_or(self.0.len());
                self.0
                    .insert(index, Menu::new(custom_menu.title, MenuRole::Normal));
                index
            };

            let mut insertion_index = 0;
            for entry in custom_menu.entries {
                match entry {
                    MenuBuilderEntry::Separator => {
                        self.0[menu_index]
                            .entries
                            .insert(insertion_index, MenuEntry::Separator);
                        insertion_index += 1;
                    }
                    MenuBuilderEntry::Item(custom_item) => {
                        if let Some(accelerator) = custom_item.accelerator {
                            self.clear_shortcut(accelerator);
                        }
                        let item = NativeMenuItem {
                            title: custom_item.title,
                            action: MenuItemAction::Custom(custom_item.action),
                            accelerator: custom_item.accelerator,
                        };
                        let existing = self.0[menu_index].entries.iter().position(|entry| {
                            matches!(entry, MenuEntry::Item(existing) if existing.title == item.title)
                        });
                        if let Some(index) = existing {
                            self.0[menu_index].entries[index] = MenuEntry::Item(item);
                        } else {
                            self.0[menu_index]
                                .entries
                                .insert(insertion_index, MenuEntry::Item(item));
                            insertion_index += 1;
                        }
                    }
                }
            }
        }
        self
    }

    /// Installs this menu bar as the application's main menu
    unsafe fn create_native(self, application: *mut Object, app_delegate: *mut Object) {
        let menubar: *mut Object = unsafe { msg_send![class!(NSMenu), new] };
        let _: () = unsafe { msg_send![application, setMainMenu:menubar] };
        for menu in self.0 {
            let menu_item = unsafe { menu.create_native(application, app_delegate) };
            let _: () = unsafe { msg_send![menubar, addItem:menu_item] };
        }
    }
}

// MARK: EventLoop
pub(crate) struct PlatformEventLoop {
    application: *mut Object,
    theme: Theme,
    event_handler: Option<Box<dyn FnMut(Event) + 'static>>,
}

impl PlatformEventLoop {
    pub(crate) fn new(builder: EventLoopBuilder) -> Self {
        // Create AppDelegate instance (registers class lazily on first call)
        let app_delegate: *mut Object = unsafe { msg_send![AppDelegate::class(), new] };

        // Get application
        let application = unsafe {
            let application: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let _: () = msg_send![application, setDelegate:app_delegate];
            application
        };

        // Create menu
        unsafe {
            let app_name: NSString = msg_send![application, valueForKey:ns_string!("name")];
            MenuBar::with_defaults(&app_name.to_string())
                .merge(builder.macos_menu)
                .create_native(application, app_delegate);
        }

        Self {
            application,
            theme: system_theme(),
            event_handler: None,
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

    fn run(mut self, event_handler: impl FnMut(Event) + 'static) -> ! {
        self.event_handler = Some(Box::new(event_handler));
        autoreleasepool(|_| unsafe {
            let delegate: *mut Object = msg_send![self.application, delegate];
            let delegate_ref = &*(delegate as *const AppDelegate);
            delegate_ref
                .ivars()
                .event_loop
                .set(&mut self as *mut PlatformEventLoop);
            let _: () = msg_send![self.application, run];
        });
        unreachable!()
    }

    fn create_proxy(&self) -> PlatformEventLoopProxy {
        PlatformEventLoopProxy::new()
    }
}

pub(crate) fn send_event(event: Event) {
    let _self = unsafe {
        let app_delegate: *mut Object = msg_send![NSApp, delegate];
        let delegate_ref = &*(app_delegate as *const AppDelegate);
        &mut *delegate_ref.ivars().event_loop.get()
    };

    if let Some(handler) = _self.event_handler.as_mut() {
        handler(event);
    }
}

pub(super) fn allow_termination_if_last_window(closing_window: *mut Object) {
    let app_delegate: *mut Object = unsafe { msg_send![NSApp, delegate] };
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
pub(crate) struct PlatformEventLoopProxy;

impl PlatformEventLoopProxy {
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl crate::EventLoopProxyInterface for PlatformEventLoopProxy {
    fn send_user_event(&self, data: String) {
        unsafe {
            let ptr = Box::leak(Box::new(Event::UserEvent(data))) as *mut Event as *mut c_void;
            let value: *mut Object = msg_send![class!(NSValue), valueWithPointer:ptr];
            let app_delegate: *mut Object = msg_send![NSApp, delegate];
            let _: () = msg_send![app_delegate, performSelectorOnMainThread:sel!(sendEvent:),
                       withObject:value,
                    waitUntilDone:Bool::NO];
        }
    }
}

// MARK: Monitor
pub(crate) struct PlatformMonitor {
    pub(crate) screen: *mut Object,
}

impl PlatformMonitor {
    pub(crate) const fn new(screen: *mut Object) -> Self {
        Self { screen }
    }
}

impl crate::MonitorInterface for PlatformMonitor {
    fn name(&self) -> String {
        let name: NSString = unsafe { msg_send![self.screen, localizedName] };
        name.to_string()
    }

    fn position(&self) -> LogicalPoint {
        let frame: NSRect = unsafe { msg_send![self.screen, frame] };
        LogicalPoint::new(frame.origin.x as f32, frame.origin.y as f32)
    }

    fn size(&self) -> LogicalSize {
        let frame: NSRect = unsafe { msg_send![self.screen, frame] };
        LogicalSize::new(frame.size.width as f32, frame.size.height as f32)
    }

    fn scale_factor(&self) -> f32 {
        let backing_scale_factor: f64 = unsafe { msg_send![self.screen, backingScaleFactor] };
        backing_scale_factor as f32
    }

    fn is_primary(&self) -> bool {
        let main_screen: *mut Object = unsafe { msg_send![class!(NSScreen), mainScreen] };
        self.screen == main_screen
    }
}
