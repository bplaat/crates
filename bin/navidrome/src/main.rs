/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A [navidrome.plaatsoft.nl](https://navidrome.plaatsoft.nl/) webview wrapper

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use tiny_webview::{Event, EventLoopBuilder, LogicalSize, Theme, WebviewBuilder};

#[allow(unused_mut, unused_variables)]
fn main() {
    let event_loop = EventLoopBuilder::build();

    let mut webview_builder = WebviewBuilder::new()
        .title("Navidrome")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state()
        .background_color(0x000000)
        .theme(Theme::Dark);
    #[cfg(target_os = "macos")]
    {
        webview_builder =
            webview_builder.macos_titlebar_style(tiny_webview::MacosTitlebarStyle::Transparent);
    }
    let mut webview = webview_builder
        .load_url("https://navidrome.plaatsoft.nl/")
        .build();

    event_loop.run(move |event| {
        if let Event::PageLoadFinished = event {
            // Inject some styles to make the player look better
            #[cfg(target_os = "macos")]
            webview.evaluate_script(
                r#"
                const scrollbarStyle = document.createElement('style');
                scrollbarStyle.innerHTML = `
                html, body {
                    overscroll-behavior: none;
                    cursor: default;
                    -webkit-user-select: none;
                    user-select: none;
                }
                body {
                    padding-top: 28px;
                }
                header.MuiAppBar-root {
                    padding-top: 28px;
                }

                ::-webkit-scrollbar {
                    width: 8px;
                    height: 8px;
                }
                ::-webkit-scrollbar-track {
                    background-color: #131313;
                }
                ::-webkit-scrollbar-thumb {
                    background-color: #444;
                    border-radius: 4px;
                }
                ::-webkit-scrollbar-thumb:hover {
                    background-color: #555;
                }
                `;
                document.head.appendChild(scrollbarStyle);
                window.addEventListener('contextmenu', (e) => e.preventDefault());
                "#,
            );
        }
    });
}
