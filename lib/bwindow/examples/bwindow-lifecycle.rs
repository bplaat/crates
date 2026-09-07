/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Desktop smoke check for empty windows and typed proxy delivery.

use std::cell::Cell;
use std::rc::Rc;

use bwindow::{Event, EventLoopBuilder, WindowBuilder, WindowEvent};

enum AppEvent {
    CloseFirst,
    CloseLast,
}

fn main() {
    let event_loop = EventLoopBuilder::new()
        .with_user_event::<AppEvent>()
        .build();
    let proxy = event_loop.create_proxy();
    let late_proxy = event_loop.create_proxy();
    let mut first = WindowBuilder::new().title("First empty window").build();
    let mut last = WindowBuilder::new().title("Last empty window").build();
    let delivered = Rc::new(Cell::new(false));
    let result = delivered.clone();
    let mut created = 0;
    event_loop.run(move |event| match event {
        Event::Window(_, WindowEvent::Create) => {
            created += 1;
            if created == 2 {
                proxy
                    .send_user_event(AppEvent::CloseFirst)
                    .expect("loop closed early");
            }
        }
        Event::UserEvent(AppEvent::CloseFirst) => {
            first.close();
            proxy
                .send_user_event(AppEvent::CloseLast)
                .expect("first close stopped loop");
        }
        Event::UserEvent(AppEvent::CloseLast) => {
            delivered.set(true);
            last.close();
        }
        _ => {}
    });
    assert!(result.get());
    assert!(late_proxy.exit().is_err());
}
