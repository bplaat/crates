/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple bwindow example

use bwindow::dpi::{LogicalPoint, LogicalSize};
use bwindow::event_loop::EventLoopBuilder;
use bwindow::window::WindowBuilder;

fn main() {
    let event_loop = EventLoopBuilder::build();

    let mut _webview_a = WindowBuilder::new()
        .title("Window A")
        .position(LogicalPoint::new(100.0, 100.0))
        .size(LogicalSize::new(1024.0, 768.0))
        .build();

    let mut _webview_b = WindowBuilder::new()
        .title("Window B")
        .position(LogicalPoint::new(100.0 + 1024.0, 100.0))
        .size(LogicalSize::new(1024.0, 768.0))
        .build();

    event_loop.run(|_| {});
}
