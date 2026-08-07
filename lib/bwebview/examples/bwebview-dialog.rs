/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A bwebview dialog example

use bwebview::{
    Event, EventLoop, FileDialog, MessageButtons, MessageDialog, Theme, WebviewBuilder,
    WebviewEvent, WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .title("Dialog Example")
        .background_color(if event_loop.theme() == Theme::Dark {
            0x222222
        } else {
            0xffffff
        })
        .build();
    let mut webview = WebviewBuilder::new(&window)
        .load_html(
            r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Dialog Example</title>
<style>
:root { color-scheme: light dark; background: #fff; }
@media (prefers-color-scheme: dark) { :root { background: #222; } }
body { font: 16px system-ui, sans-serif; padding: 1rem 2rem; display: flex; flex-direction: column; gap: .75rem; }
button { padding: .5rem 1rem; font-size: 1rem; cursor: pointer; }
#result { margin-top: 1rem; white-space: pre-wrap; font-family: monospace;
          padding: 1rem; border: 1px solid; border-radius: 4px; min-height: 3rem; }
</style>
</head>
<body>
<h1>Dialog Example</h1>
<button onclick="ipc.postMessage('show_message')">Show Message</button>
<button onclick="ipc.postMessage('pick_file')">Open Single File (Text files *.txt, *.md, *.rs, *.toml)</button>
<button onclick="ipc.postMessage('pick_files')">Open Multiple Files (Images *.png, *.jpg, *.jpeg, *.gif)</button>
<button onclick="ipc.postMessage('save_file')">Save File</button>
<div id="result">Result will appear here…</div>
<script>
window.ipc.addEventListener('message', e => {
    document.getElementById('result').textContent = e.data;
});
</script>
</body>
</html>"#,
        )
        .build();

    event_loop.run(move |event| {
        if let Event::Webview(WebviewEvent::MessageReceive(msg)) = event {
            let result = match msg.as_str() {
                "show_message" => format!(
                    "Selected: {:?}",
                    MessageDialog::new()
                        .parent(&window)
                        .title("Dialog Example")
                        .description("Choose an option")
                        .buttons(MessageButtons::YesNoCancel)
                        .show()
                ),

                "pick_file" => match FileDialog::new()
                    .parent(&window)
                    .title("Open a file")
                    .add_filter("Text files", &["txt", "md"])
                    .add_filter("Rust files", &["rs", "toml"])
                    .pick_file()
                {
                    Some(path) => format!("Picked file:\n{}", path.display()),
                    None => "No file selected".to_string(),
                },

                "pick_files" => match FileDialog::new()
                    .parent(&window)
                    .title("Open files")
                    .add_filter("Images", &["png", "jpg", "jpeg", "gif"])
                    .pick_files()
                {
                    Some(paths) => {
                        let list = paths
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect::<Vec<_>>()
                            .join("\n");
                        format!("Picked {} file(s):\n{}", paths.len(), list)
                    }
                    None => "No files selected".to_string(),
                },

                "save_file" => match FileDialog::new()
                    .parent(&window)
                    .title("Save a file")
                    .file_name("output.txt")
                    .add_filter("Text files", &["txt"])
                    .save_file()
                {
                    Some(path) => format!("Save to:\n{}", path.display()),
                    None => "Cancelled".to_string(),
                },

                _ => return,
            };
            webview.send_ipc_message(result);
        }
    });
}
