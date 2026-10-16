/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple bwebview multi-window example

use bwebview::{WebviewBuilder, WebviewEvent};
use bwindow::{Event, EventLoopBuilder, LogicalPoint, LogicalSize, WindowBuilder};

enum AppEvent {
    Webview(bwindow::WindowId, WebviewEvent),
}

fn main() {
    let event_loop = EventLoopBuilder::new()
        .with_user_event::<AppEvent>()
        .build();

    let mut window_a = WindowBuilder::new()
        .title("Window A")
        .position(LogicalPoint::new(100.0, 100.0))
        .size(LogicalSize::new(1024.0, 768.0))
        .build();
    let mut _webview_a = WebviewBuilder::new(&window_a)
        .on_event(event_loop.create_proxy(), AppEvent::Webview)
        .load_url("https://example.com")
        .build();

    let mut window_b = WindowBuilder::new()
        .title("Window B")
        .position(LogicalPoint::new(100.0 + 1024.0, 100.0))
        .size(LogicalSize::new(1024.0, 768.0))
        .build();
    let mut _webview_b = WebviewBuilder::new(&window_b)
        .on_event(event_loop.create_proxy(), AppEvent::Webview)
        .load_url("https://example.com")
        .build();

    event_loop.run(move |event| {
        if let Event::UserEvent(AppEvent::Webview(id, WebviewEvent::PageTitleChange(title))) = event
        {
            if id == window_a.id() {
                window_a.set_title(format!("Window A: {title}"));
            } else if id == window_b.id() {
                window_b.set_title(format!("Window B: {title}"));
            }
        }
    });
}
