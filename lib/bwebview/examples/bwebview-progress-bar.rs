/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A platform shell progress bar example

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    windows
))]
use std::{thread, time::Duration};

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    windows
))]
use bwebview::{EventLoop, ProgressBarState, WebviewBuilder, WindowBuilder};

#[cfg(target_os = "macos")]
fn main() {}

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    windows
))]
fn main() {
    let event_loop = EventLoop::new();
    let progress = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .title("Progress Bar Example")
        .center()
        .build();
    let mut _webview = WebviewBuilder::new(&window)
        .load_html(
            r#"<body style="font:16px system-ui;height:100vh;margin:0;display:flex;align-items:center;justify-content:center;text-align:center">
                <main><h1>Application progress</h1><p>Watch the taskbar, Dock, or window title.</p></main>
            </body>"#,
        )
        .build();

    thread::spawn(move || {
        loop {
            progress.set_progress_bar(ProgressBarState::Indeterminate);
            thread::sleep(Duration::from_secs(2));
            for step in 0..=100 {
                if step == 45 {
                    progress.set_progress_bar(ProgressBarState::Paused(0.45));
                    thread::sleep(Duration::from_secs(1));
                }
                progress.set_progress_bar(ProgressBarState::Normal(f64::from(step) / 100.0));
                thread::sleep(Duration::from_millis(35));
            }
            progress.set_progress_bar(ProgressBarState::Error(1.0));
            thread::sleep(Duration::from_secs(1));
            progress.set_progress_bar(ProgressBarState::None);
        }
    });

    event_loop.run(|_| {});
}
