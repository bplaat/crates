/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! An empty native window without a browser runtime.

use bwindow::{EventLoop, LogicalSize, WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new()
        .title("Native window")
        .size(LogicalSize::new(800.0, 600.0))
        .center()
        .build();
    event_loop.run(|_| {});
}
