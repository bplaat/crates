/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::thread;
use std::time::Duration;

use crate::{Event, LogicalPoint, LogicalSize, WebviewBuilder};

/// Webview
pub(crate) struct Webview {}

impl Webview {
    pub(crate) fn new(_builder: WebviewBuilder) -> Self {
        Self {}
    }
}

impl crate::Webview for Webview {
    fn run(&mut self, _event_handler: fn(&mut Webview, Event)) -> ! {
        loop {
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn set_title(&mut self, _title: impl AsRef<str>) {}

    fn position(&self) -> LogicalPoint {
        LogicalPoint::new(0.0, 0.0)
    }

    fn size(&self) -> LogicalSize {
        LogicalSize::new(0.0, 0.0)
    }

    fn set_position(&mut self, _point: LogicalPoint) {}

    fn set_size(&mut self, _size: LogicalSize) {}

    fn set_min_size(&mut self, _min_size: LogicalSize) {}

    fn set_resizable(&mut self, _resizable: bool) {}

    fn load_url(&mut self, _url: impl AsRef<str>) {}

    fn load_html(&mut self, _html: impl AsRef<str>) {}

    fn evaluate_script(&mut self, _script: impl AsRef<str>) {}

    #[cfg(feature = "ipc")]
    fn send_ipc_message(&mut self, _message: impl AsRef<str>) {}
}
