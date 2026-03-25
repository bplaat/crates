/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use bwebview::{
    EventLoopBuilder, InjectionTime, LogicalSize, Theme, WebviewBuilder, WindowBuilder,
};

fn main() {
    let event_loop = EventLoopBuilder::new()
        .app_id("nl", "bplaat", "Navidrome")
        .build();

    #[allow(unused_mut)]
    let mut window_builder = WindowBuilder::new()
        .title("Navidrome")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state()
        .background_color(0x000000)
        .theme(Theme::Dark);
    #[cfg(target_os = "macos")]
    {
        window_builder =
            window_builder.macos_titlebar_style(bwebview::MacosTitlebarStyle::Transparent);
    }
    let window = window_builder.build();

    let mut webview = WebviewBuilder::new(&window)
        .load_url("https://music.bplaat.nl/")
        .build();

    #[cfg(target_os = "macos")]
    let titlebar_height = window.macos_titlebar_size().height;
    #[cfg(not(target_os = "macos"))]
    let titlebar_height = 28.0;

    // Inject some styles to make the player look better
    webview.evaluate_script(
                format!(r#"
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
                )
            );

    // Inject some styles to make the player look better
    #[cfg(target_os = "macos")]
    let titlebar_height = window.macos_titlebar_size().height;
    #[cfg(not(target_os = "macos"))]
    let titlebar_height = 28.0;
    webview.add_user_script(
                format!(r#"
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

                    // Fix: WKWebView sends a 'play' media remote command when a BT headset disconnects.
                    // Intercept the MediaSession play action and block the first one after a device change.
                    (function () {{
                        let blockNextPlay = false;
                        navigator.mediaDevices.addEventListener('devicechange', () => {{
                            const anyPlaying = Array.from(document.querySelectorAll('audio, video'))
                                .some(m => !m.paused);
                            if (!anyPlaying) blockNextPlay = true;
                        }});
                        ['pointerdown', 'keydown'].forEach(type =>
                            window.addEventListener(type, () => {{ blockNextPlay = false; }}, true)
                        );
                        const origSet = navigator.mediaSession.setActionHandler.bind(navigator.mediaSession);
                        navigator.mediaSession.setActionHandler = function (action, handler) {{
                            if (action === 'play' && handler) {{
                                return origSet(action, function (details) {{
                                    if (blockNextPlay) {{ blockNextPlay = false; return; }}
                                    handler.call(this, details);
                                }});
                            }}
                            return origSet(action, handler);
                        }};
                    }})();
                    "#
                ),
                InjectionTime::DocumentStart
            );

    #[allow(unused)]
    event_loop.run(move |event| {
        #[cfg(target_os = "macos")]
        if let bwebview::Event::Window(bwebview::WindowEvent::MacosFullscreenChange(
            is_fullscreen,
        )) = event
        {
            if is_fullscreen {
                webview.evaluate_script("document.documentElement.classList.add('is-fullscreen');");
            } else {
                webview
                    .evaluate_script("document.documentElement.classList.remove('is-fullscreen');");
            }
        }
    });
}
