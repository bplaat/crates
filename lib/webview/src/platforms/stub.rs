/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::{Event, LogicalPoint, LogicalSize, WebviewBuilder};

/// Webview
pub(crate) struct Webview;

impl Webview {
    pub(crate) fn new(_builder: WebviewBuilder) -> Self {
        todo!()
    }
}

impl crate::Webview for Webview {
    fn run(&mut self, _event_handler: fn(&mut Webview, Event)) -> ! {
        todo!()
    }

    fn set_title(&mut self, _title: impl AsRef<str>) {
        todo!()
    }

    fn position(&self) -> LogicalPoint {
        todo!()
    }

    fn size(&self) -> LogicalSize {
        todo!()
    }

    fn set_position(&mut self, _point: LogicalPoint) {
        todo!()
    }

    fn set_size(&mut self, _size: LogicalSize) {
        todo!()
    }

    fn set_min_size(&mut self, _min_size: LogicalSize) {
        todo!()
    }

    fn set_resizable(&mut self, _resizable: bool) {
        todo!()
    }

    fn load_url(&mut self, _url: impl AsRef<str>) {
        todo!()
    }

    fn load_html(&mut self, _html: impl AsRef<str>) {
        todo!()
    }

    fn evaluate_script(&mut self, _script: impl AsRef<str>) {
        todo!()
    }

    #[cfg(feature = "ipc")]
    fn send_ipc_message(&mut self, _message: impl AsRef<str>) {
        todo!()
    }
}
