/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A custom menu bar example
//!
//! The menu bar API is macOS only, on other platforms the window is created without any menus

#[cfg(target_os = "macos")]
use bwebview::{Accelerator, Event, KeyCode, MenuBarBuilder, MenuBuilder, MenuItem, Modifiers};
use bwebview::{EventLoopBuilder, Theme, WebviewBuilder, WindowBuilder};

fn main() {
    let builder = EventLoopBuilder::new().app_id("nl", "bplaat", "BwebviewMenuExample");

    // Menus with standard titles are merged into the default menu bar, unknown titles are inserted
    // as new menus before the Window and Help menus
    #[cfg(target_os = "macos")]
    let builder = builder.macos_set_menu(
        MenuBarBuilder::new()
            .menu(
                MenuBuilder::new("File")
                    .item(
                        MenuItem::new("New Tab", "file.new_tab")
                            .accelerator(Accelerator::new(Modifiers::COMMAND, KeyCode::KeyT)),
                    )
                    .item(
                        MenuItem::new("Open...", "file.open")
                            .accelerator(Accelerator::new(Modifiers::COMMAND, KeyCode::KeyO)),
                    )
                    .item(
                        MenuItem::new("Save", "file.save")
                            .accelerator(Accelerator::new(Modifiers::COMMAND, KeyCode::KeyS)),
                    )
                    .separator(),
            )
            .menu(
                MenuBuilder::new("View")
                    .item(
                        MenuItem::new("Reload", "view.reload")
                            .accelerator(Accelerator::new(Modifiers::COMMAND, KeyCode::KeyR)),
                    )
                    .item(
                        MenuItem::new("Toggle Sidebar", "view.toggle_sidebar").accelerator(
                            Accelerator::new(Modifiers::COMMAND | Modifiers::OPTION, KeyCode::KeyS),
                        ),
                    )
                    .separator()
                    // Items without an accelerator are selected with the mouse only
                    .item(MenuItem::new(
                        "Enter Presentation Mode",
                        "view.presentation",
                    )),
            )
            .menu(MenuBuilder::new("Help").item(
                MenuItem::new("Documentation", "help.docs").accelerator(Accelerator::new(
                    Modifiers::COMMAND | Modifiers::SHIFT,
                    KeyCode::Slash,
                )),
            )),
    );

    let event_loop = builder.build();

    let window = WindowBuilder::new()
        .title("Menu Example")
        .background_color(if event_loop.theme() == Theme::Dark {
            0x222222
        } else {
            0xffffff
        })
        .center()
        .build();
    let mut webview = WebviewBuilder::new(&window)
        .load_html(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Menu Example</title>
<style>
:root { color-scheme: light dark; background: #fff; }
@media (prefers-color-scheme: dark) { :root { background: #222; } }
body { font: 16px system-ui, sans-serif; padding: 1rem 2rem; }
#log { margin-top: 1rem; padding: 1rem; border: 1px solid; border-radius: 4px;
       min-height: 6rem; font-family: monospace; white-space: pre-wrap; }
</style>
</head>
<body>
<h1>Menu Example</h1>
<p>Select an item from the File, View or Help menu (macOS only)</p>
<div id="log">No menu item selected yet</div>
<script>
const log = document.getElementById('log');
let actions = [];
window.ipc.addEventListener('message', e => {
    actions.unshift(e.data);
    log.textContent = actions.slice(0, 10).join('\n');
});
</script>
</body>
</html>"#,
        )
        .build();

    event_loop.run(move |event| {
        let _ = &window;
        #[cfg(target_os = "macos")]
        if let Event::MacosMenuItem(action) = event {
            webview.send_ipc_message(format!("Selected: {action}"));
        }
        #[cfg(not(target_os = "macos"))]
        let _ = (event, &mut webview);
    });
}
