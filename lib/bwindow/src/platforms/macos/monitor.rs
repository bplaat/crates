/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use objc2::runtime::AnyObject;
use objc2::{class, msg_send};

use super::cocoa::{NSRect, NSString};
use crate::dpi::{LogicalPoint, LogicalSize};

pub(crate) struct PlatformMonitor {
    pub(crate) screen: *mut AnyObject,
}

impl PlatformMonitor {
    pub(crate) fn new(screen: *mut AnyObject) -> Self {
        Self { screen }
    }

    pub(crate) fn name(&self) -> String {
        let name: NSString = unsafe { msg_send![self.screen, localizedName] };
        name.to_string()
    }

    pub(crate) fn is_primary(&self) -> bool {
        let main_screen: *mut AnyObject = unsafe { msg_send![class!(NSScreen), mainScreen] };
        unsafe { msg_send![self.screen, isEqualTo:main_screen] }
    }

    pub(crate) fn position(&self) -> LogicalPoint {
        let frame: NSRect = unsafe { msg_send![self.screen, frame] };
        LogicalPoint::new(frame.origin.x as f32, frame.origin.y as f32)
    }

    pub(crate) fn size(&self) -> LogicalSize {
        let frame: NSRect = unsafe { msg_send![self.screen, frame] };
        LogicalSize::new(frame.size.width as f32, frame.size.height as f32)
    }

    pub(crate) fn scale_factor(&self) -> f32 {
        let backing_scale_factor: f64 = unsafe { msg_send![self.screen, backingScaleFactor] };
        backing_scale_factor as f32
    }
}
