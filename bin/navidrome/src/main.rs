/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A [navidrome.plaatsoft.nl](https://navidrome.plaatsoft.nl/) webview wrapper

use tiny_webview::{EventLoopBuilder, LogicalSize, WebviewBuilder};

fn main() {
    let event_loop = EventLoopBuilder::build();

    let _webview = WebviewBuilder::new()
        .title("Navidrome")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state(true)
        .force_dark_mode(true)
        .load_url("https://navidrome.plaatsoft.nl/")
        .build();

    event_loop.run(move |_event| {});
}
