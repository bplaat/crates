/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::dpi::{LogicalPoint, LogicalSize};
use crate::platforms::PlatformMonitor;

/// Monitor
pub struct Monitor(PlatformMonitor);

impl Monitor {
    pub(crate) fn new(monitor: PlatformMonitor) -> Self {
        Self(monitor)
    }

    pub(crate) fn inner(&self) -> &PlatformMonitor {
        &self.0
    }

    /// Get monitor name
    pub fn name(&self) -> String {
        self.0.name()
    }

    /// Get if monitor is primary
    pub fn is_primary(&self) -> bool {
        self.0.is_primary()
    }

    /// Get monitor position
    ///
    /// Primary monitor has position 0x0, all other monitors are relative to the primary monitor.
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
}
