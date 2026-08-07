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

#[cfg(windows)]
use bwebview::WindowsProgressBarState;
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    windows
))]
use bwebview::{Event, EventLoopBuilder, Theme, WebviewBuilder, WindowBuilder};

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
    let event_loop = EventLoopBuilder::new()
        .app_id("nl", "bplaat", "BwebviewProgressBarExample")
        .build();

    let progress = event_loop.create_proxy();
    let mut window = WindowBuilder::new()
        .title("Progress Bar Example")
        .background_color(if event_loop.theme() == Theme::Dark {
            0x222222
        } else {
            0xffffff
        })
        .center()
        .build();
    let mut _webview = WebviewBuilder::new(&window)
        .load_html(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Progress Bar Example</title>
<style>
:root { color-scheme: light dark; background: #fff; }
@media (prefers-color-scheme: dark) { :root { background: #222; } }
body { font: 16px system-ui, sans-serif; height: 100vh; margin: 0; display: flex; align-items: center; justify-content: center; text-align: center; }
</style>
</head>
<body>
<main><h1>Application progress</h1><p>Watch the taskbar or application launcher</p></main>
</body>
</html>"#,
        )
        .build();

    thread::spawn(move || {
        loop {
            progress.send_user_event("indeterminate".to_owned());
            thread::sleep(Duration::from_secs(2));
            for step in 0..=100 {
                if step == 45 {
                    progress.send_user_event("paused:0.45".to_owned());
                    thread::sleep(Duration::from_secs(1));
                }
                progress.send_user_event(format!("normal:{}", f64::from(step) / 100.0));
                thread::sleep(Duration::from_millis(35));
            }
            progress.send_user_event("error:1".to_owned());
            thread::sleep(Duration::from_secs(1));
            progress.send_user_event("none".to_owned());
            thread::sleep(Duration::from_secs(1));
        }
    });

    event_loop.run(move |event| {
        let _ = &_webview;
        let Event::UserEvent(data) = event else {
            return;
        };
        #[cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd"
        ))]
        if data == "none" {
            window.gtk_set_progress_bar(None);
        } else if data == "indeterminate" {
            window.gtk_set_progress_bar(Some(2.0));
        } else if let Some(progress) = data.split_once(':').map(|(_, progress)| progress) {
            window.gtk_set_progress_bar(Some(progress.parse().unwrap_or_default()));
        }

        #[cfg(windows)]
        if data == "none" {
            window.windows_set_progress_bar(None, WindowsProgressBarState::Normal);
        } else if data == "indeterminate" {
            window.windows_set_progress_bar(Some(0.0), WindowsProgressBarState::Indeterminate);
        } else if let Some(progress) = data.strip_prefix("normal:") {
            window.windows_set_progress_bar(
                Some(progress.parse().unwrap_or_default()),
                WindowsProgressBarState::Normal,
            );
        } else if let Some(progress) = data.strip_prefix("paused:") {
            window.windows_set_progress_bar(
                Some(progress.parse().unwrap_or_default()),
                WindowsProgressBarState::Paused,
            );
        } else if let Some(progress) = data.strip_prefix("error:") {
            window.windows_set_progress_bar(
                Some(progress.parse().unwrap_or_default()),
                WindowsProgressBarState::Error,
            );
        }
    });
}
