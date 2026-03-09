/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A bwebview ipc example

use bwebview::{
    Event, EventLoopBuilder, LogicalSize, Theme, WebviewBuilder, WebviewEvent, WindowBuilder,
    WindowEvent,
};
use serde::{Deserialize, Serialize};

const APP_HTML: &str = include_str!("../app.html");

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum IpcMessage {
    Hello { name: String },
}

fn main() {
    let event_loop = EventLoopBuilder::new()
        .app_id("nl", "bplaat", "WebviewIpcExample")
        .build();

    let mut window = WindowBuilder::new()
        .title("Webview IPC Example")
        .size(LogicalSize::new(1024.0, 768.0))
        .min_size(LogicalSize::new(640.0, 480.0))
        .center()
        .remember_window_state()
        .theme(Theme::Dark)
        .build();
    let mut webview = WebviewBuilder::new(&window).load_html(APP_HTML).build();

    event_loop.run(move |event| match event {
        // Window events
        Event::Window(WindowEvent::Create) => {
            println!("Window created");
        }
        Event::Window(WindowEvent::Move(point)) => {
            println!("Window moved: {point:?}");
        }
        Event::Window(WindowEvent::Resize(size)) => {
            println!("Window resized: {}x{}", size.width, size.height);
            window.set_title(format!(
                "Webview IPC Example ({}x{})",
                size.width, size.height
            ));
        }
        Event::Window(WindowEvent::Close) => {
            println!("Window closed");
        }
        #[cfg(target_os = "macos")]
        Event::Window(WindowEvent::MacosFullscreenChange(is_fullscreen)) => {
            println!("Window fullscreen changed: {is_fullscreen}");
        }
        Event::Window(WindowEvent::Focus) => {
            println!("Window focused");
        }
        Event::Window(WindowEvent::Unfocus) => {
            println!("Window unfocused");
        }
        Event::Window(WindowEvent::KeyDown { key, modifiers }) => {
            println!(
                "KeyDown: {:?} (shift={} ctrl={} alt={} meta={})",
                key, modifiers.shift, modifiers.ctrl, modifiers.alt, modifiers.meta,
            );
        }
        Event::Window(WindowEvent::KeyUp { key, .. }) => {
            println!("KeyUp: {:?}", key);
        }
        Event::Window(WindowEvent::Char(ch)) => {
            println!("Char: {ch:?}");
        }
        Event::Window(WindowEvent::MouseDown { button, position }) => {
            println!("MouseDown: {:?} ({}, {})", button, position.x, position.y);
        }
        Event::Window(WindowEvent::MouseUp { button, position }) => {
            println!("MouseUp: {:?} ({}, {})", button, position.x, position.y);
        }
        Event::Window(WindowEvent::MouseMove(position)) => {
            println!("MouseMove: ({}, {})", position.x, position.y);
        }
        Event::Window(WindowEvent::MouseEnter) => {
            println!("MouseEnter");
        }
        Event::Window(WindowEvent::MouseLeave) => {
            println!("MouseLeave");
        }
        Event::Window(WindowEvent::MouseWheel { delta_x, delta_y }) => {
            println!("MouseWheel: ({delta_x}, {delta_y})");
        }

        // Webview events
        Event::Webview(WebviewEvent::PageLoadStart) => {
            println!("Page load started");
        }
        Event::Webview(WebviewEvent::PageLoadFinish) => {
            println!("Page load finished");
            let message = IpcMessage::Hello {
                name: "Webview".to_string(),
            };
            webview.send_ipc_message(
                serde_json::to_string(&message).expect("Should serialize message"),
            );
        }
        Event::Webview(WebviewEvent::PageTitleChange(title)) => {
            println!("Title changed: {title}");
            window.set_title(title);
        }
        Event::Webview(WebviewEvent::MessageReceive(message)) => {
            match serde_json::from_str(&message).expect("Can't parse message") {
                IpcMessage::Hello { name } => {
                    println!("Hello, {name}!");
                }
            }
        }

        _ => {}
    });
}
