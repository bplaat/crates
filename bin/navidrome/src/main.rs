/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use bwebview::{
    EventLoop, EventLoopBuilder, EventLoopHandler, InjectionTime, LogicalSize, Theme,
    WebviewBuilder, WebviewHandler, Window, Webview, WindowBuilder, WindowHandler,
};

#[derive(Default)]
struct App {
    window: Option<Window>,
    webview: Option<Webview>,
}

impl EventLoopHandler for App {
    fn on_init(&mut self) {
        #[allow(unused_mut)]
        let mut window_builder = WindowBuilder::new()
            .title("Navidrome")
            .size(LogicalSize::new(1024.0, 768.0))
            .min_size(LogicalSize::new(640.0, 480.0))
            .center()
            .remember_window_state()
            .background_color(0x000000)
            .theme(Theme::Dark)
            .handler(self);
        #[cfg(target_os = "macos")]
        {
            window_builder =
                window_builder.macos_titlebar_style(bwebview::MacosTitlebarStyle::Transparent);
        }
        let window = window_builder.build();

        #[cfg(target_os = "macos")]
        let titlebar_height = window.macos_titlebar_size().height;
        #[cfg(not(target_os = "macos"))]
        let titlebar_height = 28.0f32;

        let styles = format!(
            r#"
            const style = document.createElement('style');
            style.innerHTML = `
               html {{
                    overscroll-behavior: none;
                    cursor: default;
                    -webkit-user-select: none;
                    user-select: none;
                }}
                ::-webkit-scrollbar {{
                    width: 8px;
                    height: 8px;
                }}
                ::-webkit-scrollbar-track,
                ::-webkit-scrollbar-corner {{
                    background-color: #131313;
                }}
                ::-webkit-scrollbar-thumb {{
                    background-color: #444;
                    border-radius: 4px;
                }}
                ::-webkit-scrollbar-thumb:hover {{
                    background-color: #555;
                }}
                .is-bwebview-macos:not(.is-fullscreen) #main-content {{
                    padding-top: {titlebar_height}px !important;
                }}
                .is-bwebview-macos:not(.is-fullscreen) header.MuiAppBar-root {{
                    padding-top: {titlebar_height}px;
                }}
            `;
            document.documentElement.appendChild(style);
            if (navigator.userAgent.includes('bwebview') && navigator.userAgent.includes('Macintosh')) {{
                document.documentElement.classList.add('is-bwebview-macos');
            }}
            window.addEventListener('contextmenu', (e) => e.preventDefault());
            "#
        );

        let mut webview = WebviewBuilder::new(&window)
            .load_url("https://music.bplaat.nl/")
            .handler(self)
            .build();

        webview.evaluate_script(styles.clone());
        webview.add_user_script(styles, InjectionTime::DocumentStart);

        self.window = Some(window);
        self.webview = Some(webview);
    }
}

impl WindowHandler for App {
    fn on_close(&mut self, _window: &mut Window) -> bool {
        EventLoop::quit();
        true
    }

    #[cfg(target_os = "macos")]
    fn on_fullscreen_change(&mut self, _window: &mut Window, is_fullscreen: bool) {
        if let Some(webview) = self.webview.as_mut() {
            if is_fullscreen {
                webview
                    .evaluate_script("document.documentElement.classList.add('is-fullscreen');");
            } else {
                webview
                    .evaluate_script("document.documentElement.classList.remove('is-fullscreen');");
            }
        }
    }
}

impl WebviewHandler for App {}

fn main() {
    let mut app = App::default();
    EventLoopBuilder::new()
        .app_id("nl", "bplaat", "Navidrome")
        .handler(&mut app)
        .build()
        .run();
}
