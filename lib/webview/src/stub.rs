/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use super::*;

/// Webview
pub struct Webview;

impl Webview {
    pub(crate) fn new(_builder: WebviewBuilder) -> Self {
        todo!()
    }

    /// Start event loop
    pub fn run(&mut self, _event_handler: fn(&mut Webview, Event)) {
        todo!()
    }

    /// Set title
    pub fn set_title(&mut self, _title: impl AsRef<str>) {
        todo!()
    }

    /// Set position
    pub fn set_position(&mut self, _x: i32, _y: i32) {
        todo!()
    }

    /// Set size
    pub fn set_size(&mut self, _width: i32, _height: i32) {
        todo!()
    }

    /// Open URL
    pub fn open_url(&mut self, _url: impl AsRef<str>) {
        todo!()
    }

    /// Open HTML
    pub fn open_html(&mut self, _html: impl AsRef<str>) {
        todo!()
    }

    /// Eval JavaScript
    pub fn eval(&mut self, _js: String) {
        todo!()
    }
}
