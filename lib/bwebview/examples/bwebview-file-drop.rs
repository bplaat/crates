/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A bwebview file drop example

use bwebview::{Event, EventLoop, WebviewBuilder, WindowBuilder, WindowEvent};

fn main() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .title("File Drop Example")
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
body { font: 16px system-ui, sans-serif; padding: 1rem 2rem; background-color: #fff; color: #111; }
#result { margin-top: 1rem; white-space: pre-wrap; font-family: monospace;
          background: #f5f5f5; padding: 1rem; border-radius: 4px; min-height: 3rem; }
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
        if let Event::Window(WindowEvent::DroppedFile(path)) = event {
            webview.send_ipc_message(path.to_string_lossy());
        }
    });
}
