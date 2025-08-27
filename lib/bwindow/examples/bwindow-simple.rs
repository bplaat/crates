/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple bwindow example

use bwindow::event::Event;
use bwindow::event_loop::EventLoopBuilder;
use bwindow::window::{Window, WindowBuilder};

fn main() {
    let event_loop = EventLoopBuilder::build();

    let mut window = WindowBuilder::new().title("A simple window").build();

    fn update_window_title(window: &mut Window) {
        let size = window.size();
        window.set_title(format!("A simple window ({}x{})", size.width, size.height));
    }

    event_loop.run(move |event| match event {
        Event::WindowCreated => {
            println!("Window created");
            update_window_title(&mut window);
        }
        Event::WindowMoved(point) => {
            println!("Window moved: {}x{}", point.x, point.y);
        }
        Event::WindowResized(size) => {
            println!("Window resized: {}x{}", size.width, size.height);
            update_window_title(&mut window);
        }
        Event::WindowClosed => {
            println!("Window closed");
        }
        _ => {}
    });
}
