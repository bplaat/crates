/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Browser and canvas windows sharing one typed event loop.
//! Pass --smoke to verify IPC routing, redraws, and closing the browser first.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use bcanvas::{CanvasBuilder, Color};
use bwebview::{WebviewBuilder, WebviewEvent};
use bwindow::{Event, EventLoopBuilder, LogicalSize, WindowBuilder, WindowEvent, WindowId};

enum AppEvent {
    Browser(WindowId, WebviewEvent),
    BrowserClosed,
}

fn main() {
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    let event_loop = EventLoopBuilder::new()
        .with_user_event::<AppEvent>()
        .build();
    let proxy = event_loop.create_proxy();
    let late_proxy = event_loop.create_proxy();
    if smoke {
        let watchdog = event_loop.create_proxy();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(10));
            let _ = watchdog.exit();
        });
    }
    let mut canvas_window = WindowBuilder::new()
        .title("Native canvas")
        .size(LogicalSize::new(480.0, 320.0))
        .build();
    let mut browser_window = WindowBuilder::new()
        .title("Browser controls")
        .size(LogicalSize::new(480.0, 320.0))
        .build();
    let canvas_id = canvas_window.id();
    let browser_id = browser_window.id();
    let mut canvas = CanvasBuilder::new(&canvas_window).build();
    let mut browser = WebviewBuilder::new(&browser_window)
        .on_event(event_loop.create_proxy(), AppEvent::Browser)
        .load_html(
            r#"<!doctype html><meta charset="utf-8">
            <style>body{font:18px system-ui;padding:32px}button{font:inherit;padding:12px}</style>
            <h1>One event loop, two contents</h1>
            <button onclick="window.ipc.postMessage('paint')">Repaint the native canvas</button>"#,
        )
        .build();
    let mut blue = false;
    let mut browser_closed = false;
    let completed = Rc::new(Cell::new(false));
    let result = completed.clone();
    event_loop.run(move |event| match event {
        Event::Window(id, WindowEvent::RedrawRequested) if id == canvas_id => {
            assert!(canvas.draw(|ctx| {
                ctx.set_fill_style(if blue {
                    Color::rgb(35, 110, 210)
                } else {
                    Color::rgb(40, 140, 100)
                });
                ctx.fill_rect(0.0, 0.0, ctx.width(), ctx.height());
                ctx.set_fill_style(Color::rgb(255, 255, 255));
                ctx.set_font("sans-serif", 22.0);
                ctx.fill_text("Rendered without a browser", 24.0, 64.0);
            }));
            if smoke && browser_closed {
                completed.set(true);
                canvas_window.close();
            }
        }
        Event::UserEvent(AppEvent::Browser(id, event)) => {
            assert_eq!(id, browser_id);
            match event {
                WebviewEvent::PageLoadFinish if smoke && !browser_closed => {
                    browser.evaluate_script("window.ipc.postMessage('paint')");
                }
                WebviewEvent::MessageReceive(message) if message == "paint" => {
                    blue = !blue;
                    canvas_window.request_redraw();
                    if smoke && !browser_closed {
                        browser_window.close();
                        assert!(browser.url().is_none());
                        proxy
                            .send_user_event(AppEvent::BrowserClosed)
                            .expect("closing browser stopped loop");
                    }
                }
                _ => {}
            }
        }
        Event::UserEvent(AppEvent::BrowserClosed) => {
            browser_closed = true;
            assert!(!canvas.draw(|_| panic!("paint context escaped callback")));
            canvas_window.request_redraw();
        }
        _ => {}
    });
    if smoke {
        assert!(result.get(), "mixed-content smoke test timed out");
        assert!(late_proxy.send_user_event(AppEvent::BrowserClosed).is_err());
    }
}
