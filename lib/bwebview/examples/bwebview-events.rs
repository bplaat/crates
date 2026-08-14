/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Prints native window, mouse, wheel, and keyboard events.

use bwebview::{
    Event, EventLoop, KeyboardEvent, LogicalSize, MouseEvent, Theme, WindowBuilder, WindowEvent,
    WindowEvents,
};

fn main() {
    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new()
        .title("bwebview Window Events")
        .size(LogicalSize::new(640.0, 480.0))
        .center()
        .enable_events(
            WindowEvents::MOVE
                | WindowEvents::RESIZE
                | WindowEvents::THEME_CHANGE
                | WindowEvents::FOCUS
                | WindowEvents::MOUSE
                | WindowEvents::WHEEL
                | WindowEvents::KEYBOARD,
        )
        .build();

    event_loop.run(move |event| {
        if let Event::Window(event) = event {
            print_window_event(event)
        }
    });
}

fn print_window_event(event: WindowEvent) {
    match event {
        WindowEvent::Create => println!("window create"),
        WindowEvent::Move(point) => println!("window move: x={} y={}", point.x, point.y),
        WindowEvent::Resize(size) => {
            println!("window resize: width={} height={}", size.width, size.height);
        }
        WindowEvent::Focus => println!("window focus"),
        WindowEvent::Blur => println!("window blur"),
        WindowEvent::ThemeChange(theme) => println!(
            "window theme change: {}",
            match theme {
                Theme::Light => "light",
                Theme::Dark => "dark",
            }
        ),
        WindowEvent::MouseDown(event) => print_mouse_event("mouse down", &event),
        WindowEvent::MouseUp(event) => print_mouse_event("mouse up", &event),
        WindowEvent::MouseMove(event) => print_mouse_event("mouse move", &event),
        WindowEvent::MouseEnter(event) => print_mouse_event("mouse enter", &event),
        WindowEvent::MouseLeave(event) => print_mouse_event("mouse leave", &event),
        WindowEvent::Click(event) => print_mouse_event("click", &event),
        WindowEvent::Wheel(event) => {
            print_mouse_event("wheel", &event.mouse);
            println!(
                "  delta: x={} y={} z={} mode={}",
                event.delta_x, event.delta_y, event.delta_z, event.delta_mode
            );
        }
        WindowEvent::KeyDown(event) => print_keyboard_event("key down", &event),
        WindowEvent::KeyUp(event) => print_keyboard_event("key up", &event),
        WindowEvent::CloseRequested(_) => println!("window close requested"),
        #[cfg(feature = "file_drop")]
        WindowEvent::DroppedFile(path) => println!("window dropped file: {}", path.display()),
        #[cfg(target_os = "macos")]
        WindowEvent::MacosFullscreenChange(fullscreen) => {
            println!("window macOS fullscreen change: {fullscreen}");
        }
    }
}

fn print_mouse_event(name: &str, event: &MouseEvent) {
    println!(
        "{name}: client=({}, {}) screen=({}, {}) movement=({}, {}) button={} buttons={} detail={} alt={} ctrl={} meta={} shift={}",
        event.client_x,
        event.client_y,
        event.screen_x,
        event.screen_y,
        event.movement_x,
        event.movement_y,
        event.button,
        event.buttons,
        event.detail,
        event.alt_key,
        event.ctrl_key,
        event.meta_key,
        event.shift_key,
    );
}

fn print_keyboard_event(name: &str, event: &KeyboardEvent) {
    println!(
        "{name}: key={} code={} location={} repeat={} composing={} alt={} ctrl={} meta={} shift={}",
        event.key,
        event.code,
        event.location,
        event.repeat,
        event.is_composing,
        event.alt_key,
        event.ctrl_key,
        event.meta_key,
        event.shift_key,
    );
}
