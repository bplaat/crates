/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A bwebview file dialog example

use bwebview::{Event, EventLoop, FileDialog, WebviewBuilder};

fn main() {
    let event_loop = EventLoop::new();

    let mut webview = WebviewBuilder::new()
        .title("File Dialog Example")
        .load_html(
            r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>File Dialog Example</title>
<style>
body { font-family: sans-serif; padding: 1rem 2rem; display: flex; flex-direction: column; gap: .75rem; }
button { padding: .5rem 1rem; font-size: 1rem; cursor: pointer; }
#result { margin-top: 1rem; white-space: pre-wrap; font-family: monospace;
          background: #f5f5f5; padding: 1rem; border-radius: 4px; min-height: 3rem; }
</style>
</head>
<body>
<h1>File Dialog Example</h1>
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
        if let Event::PageMessageReceived(msg) = event {
            let result = match msg.as_str() {
                "pick_file" => match FileDialog::new()
                    .title("Open a file")
                    .add_filter("Text files", &["txt", "md"])
                    .add_filter("Rust files", &["rs", "toml"])
                    .pick_file()
                {
                    Some(path) => format!("Picked file:\n{}", path.display()),
                    None => "No file selected".to_string(),
                },

                "pick_files" => match FileDialog::new()
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
                    .title("Save a file")
                    .set_file_name("output.txt")
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
