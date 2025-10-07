/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::monitor::Monitor;
use crate::platforms::{PlatformEventLoop, PlatformEventLoopProxy, PlatformMonitor};

// MARK: EventLoopBuilder
/// EventLoop handler trait
pub trait EventLoopHandler {
    /// Called when the application is activated
    fn activate(&mut self, event_loop: &mut EventLoop);
}

/// EventLoop builder
#[derive(Default)]
pub struct EventLoopBuilder<'a> {
    pub(crate) app_id: Option<String>,
    pub(crate) handler: Option<&'a mut (dyn EventLoopHandler)>,
}

impl<'a> EventLoopBuilder<'a> {
    /// Create new webview builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set application id
    pub fn app_id(mut self, app_id: impl AsRef<str>) -> Self {
        self.app_id = Some(app_id.as_ref().to_string());
        self
    }

    /// Set handler
    pub fn handler(mut self, handler: &'a mut (dyn EventLoopHandler)) -> Self {
        self.handler = Some(handler);
        self
    }

    /// Create new event loop
    pub fn build(self) -> EventLoop<'a> {
        EventLoop::from_platform(PlatformEventLoop::new(self))
    }
}

// MARK: EventLoop
pub(crate) trait EventLoopInterface {
    fn primary_monitor(&self) -> PlatformMonitor;
    fn available_monitors(&self) -> Vec<PlatformMonitor>;
    fn create_proxy(&self) -> PlatformEventLoopProxy;
    fn run(self) -> !;
}

/// Event loop
pub struct EventLoop<'a>(PlatformEventLoop<'a>);

impl<'a> EventLoop<'a> {
    fn from_platform(event_loop: PlatformEventLoop<'a>) -> Self {
        Self(event_loop)
    }

    /// Create new event loop
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        EventLoopBuilder::new().build()
    }

    /// Get primary monitor
    pub fn primary_monitor(&self) -> Monitor {
        Monitor::new(self.0.primary_monitor())
    }

    /// List available monitors
    pub fn available_monitors(&self) -> Vec<Monitor> {
        self.0
            .available_monitors()
            .into_iter()
            .map(Monitor::new)
            .collect()
    }

    /// Create new event loop proxy
    pub fn create_proxy(&self) -> EventLoopProxy {
        EventLoopProxy::new(self.0.create_proxy())
    }

    /// Run the event loop
    pub fn run(self) -> ! {
        self.0.run()
    }
}

// MARK: EventLoopProxy
/// Event loop proxy
pub struct EventLoopProxy(PlatformEventLoopProxy);

pub(crate) trait EventLoopProxyInterface {
    fn send_user_event(&self, data: String);
}

impl EventLoopProxy {
    fn new(proxy: PlatformEventLoopProxy) -> Self {
        Self(proxy)
    }

    /// Send user event to the event loop
    pub fn send_user_event(&self, data: String) {
        self.0.send_user_event(data);
    }
}
