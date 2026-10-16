/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A bwebview file drop example

use bwebview::WebviewBuilder;
use bwindow::{Event, EventLoop, Theme, WindowBuilder, WindowEvent};

const fn background_color(theme: Theme) -> u32 {
    match theme {
        Theme::Light => 0xffffff,
        Theme::Dark => 0x222222,
    }
}

fn main() {
    let event_loop = EventLoop::new();

    let mut window = WindowBuilder::new()
        .title("File Drop Example")
        .background_color(background_color(event_loop.theme()))
        .allow_file_drop(true)
        .build();
    let mut webview = WebviewBuilder::new(&window)
        .load_html(
            r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>File Drop Example</title>
<style>
:root { color-scheme: light dark; background: #fff; }
@media (prefers-color-scheme: dark) { :root { background: #222; } }
body { font: 16px system-ui, sans-serif; padding: 1rem 2rem; }
#result { margin-top: 1rem; white-space: pre-wrap; font-family: monospace;
          padding: 1rem; border: 1px solid; border-radius: 4px; min-height: 3rem; }
</style>
</head>
<body>
<h1>File Drop Example</h1>
<p>Drop one or more files anywhere in this window.</p>
<div id="result"></div>
<script>
window.ipc.addEventListener('message', e => {
    document.getElementById('result').textContent += `${e.data}\n`;
});
</script>
</body>
</html>"#,
        )
        .build();

    event_loop.run(move |event| {
        if let Event::Window(_, WindowEvent::ThemeChanged(theme)) = &event {
            window.set_background_color(background_color(*theme));
        }
        if let Event::Window(_, WindowEvent::DroppedFile(path)) = event {
            webview.send_ipc_message(path.to_string_lossy());
        }
    });
}
