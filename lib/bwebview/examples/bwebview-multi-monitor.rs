/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple bwebview multi-monitor example

use bwebview::{EventLoop, WebviewBuilder, WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();

    // Print monitors information
    let mut monitors = event_loop.available_monitors();
    monitors.sort_by_key(|m| !m.is_primary());
    for monitor in &monitors {
        println!(
            "{} {}x{}x{}x{}@{} {}",
            monitor.name(),
            monitor.position().x,
            monitor.position().y,
            monitor.size().width,
            monitor.size().height,
            monitor.scale_factor(),
            if monitor.is_primary() {
                "(primary)"
            } else {
                ""
            }
        );
    }

    let window_a = WindowBuilder::new()
        .title("Window A")
        .monitor(&monitors[0])
        .center()
        .build();
    let mut _webview_a = WebviewBuilder::new(&window_a)
        .load_url("https://example.com")
        .build();

    let window_b = WindowBuilder::new()
        .title("Window B")
        .monitor(monitors.get(1).unwrap_or(&monitors[0]))
        .center()
        .build();
    let mut _webview_b = WebviewBuilder::new(&window_b)
        .load_url("https://example.com")
        .build();

    event_loop.run(|_| {});
}
