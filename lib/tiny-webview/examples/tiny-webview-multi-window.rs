/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple tiny webview multi-window example

use tiny_webview::{EventLoopBuilder, LogicalPoint, LogicalSize, WebviewBuilder};

fn main() {
    let event_loop = EventLoopBuilder::build();

    let mut _webview_a = WebviewBuilder::new()
        .title("Window A")
        .position(LogicalPoint::new(100.0, 100.0))
        .size(LogicalSize::new(1024.0, 768.0))
        .load_url("https://example.com")
        .build();

    let mut _webview_b = WebviewBuilder::new()
        .title("Window B")
        .position(LogicalPoint::new(100.0 + 1024.0, 100.0))
        .size(LogicalSize::new(1024.0, 768.0))
        .load_url("https://example.com")
        .build();

    event_loop.run(|_| {});
}
