/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Send typed worker results to the main event loop.

use bwebview::WebviewBuilder;
use bwindow::{Event, EventLoopBuilder, WindowBuilder};

enum AppEvent {
    WorkerFinished(u32),
}

fn main() {
    let event_loop = EventLoopBuilder::new()
        .with_user_event::<AppEvent>()
        .build();
    let mut window = WindowBuilder::new().title("Working...").build();
    let _webview = WebviewBuilder::new(&window)
        .load_html("Worker example")
        .build();
    let proxy = event_loop.create_proxy();
    std::thread::spawn(move || proxy.send_user_event(AppEvent::WorkerFinished(42)));

    event_loop.run(move |event| {
        if let Event::UserEvent(AppEvent::WorkerFinished(answer)) = event {
            window.set_title(format!("Worker result: {answer}"));
        }
    });
}
