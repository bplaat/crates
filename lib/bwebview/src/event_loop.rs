/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::platforms::{PlatformEventLoop, PlatformEventLoopProxy, PlatformMonitor};
use crate::{Event, LogicalPoint, LogicalSize, Theme};

// MARK: ProgressBarState
/// Application progress shown by the platform shell.
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    windows
))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ProgressBarState {
    /// Hide application progress.
    #[default]
    None,
    /// Show progress without a known completion percentage.
    Indeterminate,
    /// Show normal progress between `0.0` and `1.0`.
    Normal(f64),
    /// Show paused progress between `0.0` and `1.0`.
    Paused(f64),
    /// Show failed progress between `0.0` and `1.0`.
    Error(f64),
}

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    windows
))]
impl ProgressBarState {
    pub(crate) const fn progress(self) -> Option<f64> {
        match self {
            Self::Normal(progress) | Self::Paused(progress) | Self::Error(progress) => {
                Some(progress.clamp(0.0, 1.0))
            }
            Self::None | Self::Indeterminate => None,
        }
    }
}

#[cfg(all(
    test,
    any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        windows
    )
))]
mod progress_bar_state_tests {
    use super::ProgressBarState;

    #[test]
    fn progress_values_are_clamped() {
        assert_eq!(ProgressBarState::Normal(-1.0).progress(), Some(0.0));
        assert_eq!(ProgressBarState::Paused(0.5).progress(), Some(0.5));
        assert_eq!(ProgressBarState::Error(2.0).progress(), Some(1.0));
        assert_eq!(ProgressBarState::Indeterminate.progress(), None);
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
    fn theme(&self) -> Theme;
    fn primary_monitor(&self) -> PlatformMonitor;
    fn available_monitors(&self) -> Vec<PlatformMonitor>;
    fn create_proxy(&self) -> PlatformEventLoopProxy;
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        windows
    ))]
    fn set_progress_bar(&self, state: ProgressBarState);
    fn run(self, event_handler: impl FnMut(Event) + 'static) -> !;
}

/// Event loop
pub struct EventLoop(pub(crate) PlatformEventLoop);

impl EventLoop {
    pub(crate) const fn from_platform(event_loop: PlatformEventLoop) -> Self {
        Self(event_loop)
    }

    /// Create new event loop
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        EventLoopBuilder::new().build()
    }

    /// Get the current system theme
    pub fn theme(&self) -> Theme {
        self.0.theme()
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

    /// Set application progress in the platform shell.
    ///
    /// Use an [`EventLoopProxy`] to update progress from a worker thread.
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        windows
    ))]
    pub fn set_progress_bar(&self, state: ProgressBarState) {
        self.0.set_progress_bar(state);
    }

    /// Run the event loop
    pub fn run(self, event_handler: impl FnMut(Event) + 'static) -> ! {
        self.0.run(event_handler)
    }
}

// MARK: EventLoopProxy
pub(crate) trait EventLoopProxyInterface {
    fn send_user_event(&self, data: String);
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        windows
    ))]
    fn set_progress_bar(&self, state: ProgressBarState);
}

/// Event loop proxy
pub struct EventLoopProxy(pub(crate) PlatformEventLoopProxy);

impl EventLoopProxy {
    pub(crate) const fn new(proxy: PlatformEventLoopProxy) -> Self {
        Self(proxy)
    }

    /// Send user event to the event loop
    pub fn send_user_event(&self, data: String) {
        self.0.send_user_event(data);
    }

    /// Set application progress from any thread.
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
        windows
    ))]
    pub fn set_progress_bar(&self, state: ProgressBarState) {
        self.0.set_progress_bar(state);
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
    pub(crate) const fn new(monitor: PlatformMonitor) -> Self {
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
