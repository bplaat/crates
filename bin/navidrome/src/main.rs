/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A [music.bplaat.nl](https://music.bplaat.nl/) webview wrapper

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use bwebview::{Event, EventLoopBuilder, LogicalSize, Theme, WebviewBuilder};

fn main() {
    let event_loop = EventLoopBuilder::new()
        .app_id("nl.bplaat.Navidrome")
        .build();

    #[allow(unused_mut)]
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
            webview_builder.macos_titlebar_style(bwebview::MacosTitlebarStyle::Transparent);
    }
    let mut webview = webview_builder.load_url("https://music.bplaat.nl/").build();

    event_loop.run(move |event| match event {
        #[cfg(target_os = "macos")]
        Event::MacosWindowFullscreenChanged(is_fullscreen) => {
            if is_fullscreen {
                webview.evaluate_script("document.body.classList.add('is-fullscreen');");
            } else {
                webview.evaluate_script("document.body.classList.remove('is-fullscreen');");
            }
        }
        Event::PageLoadFinished => {
            // Inject some styles to make the player look better
            webview.evaluate_script(
                r#"
                    const style = document.createElement('style');
                    style.innerHTML = `
                       html {
                            overscroll-behavior: none;
                            cursor: default;
                            -webkit-user-select: none;
                            user-select: none;
                        }
                        ::-webkit-scrollbar {
                            width: 8px;
                            height: 8px;
                        }
                        ::-webkit-scrollbar-track,
                        ::-webkit-scrollbar-corner {
                            background-color: #131313;
                        }
                        ::-webkit-scrollbar-thumb {
                            background-color: #444;
                            border-radius: 4px;
                        }
                        ::-webkit-scrollbar-thumb:hover {
                            background-color: #555;
                        }
                        body.is-bwebview-macos:not(.is-fullscreen) #main-content {
                            padding-top: 28px !important;
                        }
                        body.is-bwebview-macos:not(.is-fullscreen) header.MuiAppBar-root {
                            padding-top: 28px;
                        }
                    `;
                    document.head.appendChild(style);
                    if (navigator.userAgent.includes('bwebview') && navigator.userAgent.includes('Macintosh')) {
                        document.body.classList.add('is-bwebview-macos');
                    }
                    window.addEventListener('contextmenu', (e) => e.preventDefault());
                    "#,
            );
        }

        _ => {}
    });
}
