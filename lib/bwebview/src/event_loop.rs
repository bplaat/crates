/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::platforms::{PlatformEventLoop, PlatformEventLoopProxy, PlatformMonitor};
use crate::{LogicalPoint, LogicalSize};

// MARK: EventLoopHandler
/// Event loop handler trait
pub trait EventLoopHandler {
    /// Called once when the application has finished launching, before the event loop starts.
    /// Create your windows and webviews here.
    fn on_init(&mut self) {}
    /// Called when a user event is received (sent via `EventLoopProxy::send_user_event`)
    fn on_user_event(&mut self, data: String) {
        let _ = data;
    }
}

// MARK: AppId
pub(crate) struct AppId {
    pub qualifier: String,
    pub organization: String,
    pub application: String,
}

// MARK: EventLoopBuilder
/// EventLoop builder
#[derive(Default)]
pub struct EventLoopBuilder {
    pub(crate) app_id: Option<AppId>,
    pub(crate) event_loop_handler: Option<*mut dyn EventLoopHandler>,
}

impl EventLoopBuilder {
    /// Create new event loop builder
    pub fn new() -> Self {
        Self::default()
    }

    /// App id used for storing window state and other platform specific features
    pub fn app_id(
        mut self,
        qualifier: impl AsRef<str>,
        organization: impl AsRef<str>,
        application: impl AsRef<str>,
    ) -> Self {
        self.app_id = Some(AppId {
            qualifier: qualifier.as_ref().to_string(),
            organization: organization.as_ref().to_string(),
            application: application.as_ref().to_string(),
        });
        self
    }

    /// Set event loop handler
    pub fn handler<H: EventLoopHandler + 'static>(mut self, handler: &mut H) -> Self {
        self.event_loop_handler = Some(handler as *mut dyn EventLoopHandler);
        self
    }

    /// Create new event loop
    pub fn build(self) -> EventLoop {
        EventLoop::from_platform(PlatformEventLoop::new(self))
    }
}

// MARK: EventLoop
pub(crate) trait EventLoopInterface {
    fn primary_monitor(&self) -> PlatformMonitor;
    fn available_monitors(&self) -> Vec<PlatformMonitor>;
    fn create_proxy(&self) -> PlatformEventLoopProxy;
    fn run(self) -> !;
    fn quit();
}

/// Event loop
pub struct EventLoop(pub(crate) PlatformEventLoop);

impl EventLoop {
    pub(crate) fn from_platform(event_loop: PlatformEventLoop) -> Self {
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

    /// Quit the event loop and exit the application
    pub fn quit() {
        PlatformEventLoop::quit();
    }

    /// Run the event loop
    pub fn run(self) -> ! {
        self.0.run()
    }
}

// MARK: EventLoopProxy
pub(crate) trait EventLoopProxyInterface {
    fn send_user_event(&self, data: String);
}

/// Event loop proxy
pub struct EventLoopProxy(pub(crate) PlatformEventLoopProxy);

impl EventLoopProxy {
    pub(crate) fn new(proxy: PlatformEventLoopProxy) -> Self {
        Self(proxy)
    }

    /// Send user event to the event loop
    pub fn send_user_event(&self, data: String) {
        self.0.send_user_event(data);
    }
}

// MARK: Monitor
pub(crate) trait MonitorInterface {
    fn name(&self) -> String;
    fn position(&self) -> LogicalPoint;
    fn size(&self) -> LogicalSize;
    fn scale_factor(&self) -> f32;
    fn is_primary(&self) -> bool;
}

/// Monitor
pub struct Monitor(pub(crate) PlatformMonitor);

impl Monitor {
    pub(crate) fn new(monitor: PlatformMonitor) -> Self {
        Self(monitor)
    }

    /// Get monitor name
    pub fn name(&self) -> String {
        self.0.name()
    }

    /// Get monitor position
    ///
    /// Primary monitor is 0x0 position all other monitors are relative to the primary monitor.
    pub fn position(&self) -> LogicalPoint {
        self.0.position()
    }

    /// Get monitor size
    pub fn size(&self) -> LogicalSize {
        self.0.size()
    }

    /// Get monitor scale factor
    pub fn scale_factor(&self) -> f32 {
        self.0.scale_factor()
    }

    /// Get if monitor is primary
    pub fn is_primary(&self) -> bool {
        self.0.is_primary()
    }
}
