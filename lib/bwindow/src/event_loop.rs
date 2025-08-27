/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::event::Event;
use crate::monitor::Monitor;
use crate::platforms::{PlatformEventLoop, PlatformEventLoopProxy};

// MARK: EventLoop
/// Event loop builder
pub struct EventLoopBuilder;

impl EventLoopBuilder {
    /// Create new event loop
    pub fn build() -> EventLoop {
        EventLoop::new(PlatformEventLoop::new())
    }
}

/// Event loop
pub struct EventLoop(PlatformEventLoop);

impl EventLoop {
    fn new(event_loop: PlatformEventLoop) -> Self {
        Self(event_loop)
    }

    /// List available monitors
    pub fn available_monitors(&self) -> Vec<Monitor> {
        self.0
            .available_monitors()
            .into_iter()
            .map(Monitor::new)
            .collect()
    }

    /// Get primary monitor
    pub fn primary_monitor(&self) -> Monitor {
        Monitor::new(self.0.primary_monitor())
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
/// Event loop proxy
pub struct EventLoopProxy(PlatformEventLoopProxy);

impl EventLoopProxy {
    fn new(proxy: PlatformEventLoopProxy) -> Self {
        Self(proxy)
    }

    /// Send user event to the event loop
    pub fn send_user_event(&self, data: Vec<u8>) {
        self.0.send_user_event(data);
    }
}
