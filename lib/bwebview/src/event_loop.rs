/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::platforms::{PlatformEventLoop, PlatformEventLoopProxy, PlatformMonitor};
use crate::{Event, LogicalPoint, LogicalSize};

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
    fn run(self, event_handler: impl FnMut(Event) + 'static) -> !;
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

    /// Run the event loop
    pub fn run(self, event_handler: impl FnMut(Event) + 'static) -> ! {
        self.0.run(event_handler)
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
