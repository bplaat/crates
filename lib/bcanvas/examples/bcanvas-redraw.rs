/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Measures input-style redraw requests through the native event/paint pipeline.
//! Run in release mode; timings end at paint entry, not screen presentation.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use bcanvas::{CanvasBuilder, Color};
use bwindow::{Event, EventLoopBuilder, WindowBuilder, WindowEvent};

fn main() {
    let event_loop = EventLoopBuilder::new().with_user_event::<()>().build();
    let proxy = event_loop.create_proxy();
    let watchdog = event_loop.create_proxy();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(15));
        let _ = watchdog.exit();
    });
    let (next, requests) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        while requests.recv().is_ok() {
            std::thread::sleep(Duration::from_millis(20));
            if proxy.send_user_event(()).is_err() {
                break;
            }
        }
    });
    let mut window = WindowBuilder::new().title("Canvas redraw latency").build();
    let mut canvas = CanvasBuilder::new(&window).build();
    let completed = Rc::new(Cell::new(false));
    let result = completed.clone();
    let mut started = false;
    let mut requested = None::<Instant>;
    let mut samples = Vec::with_capacity(120);
    event_loop.run(move |event| match event {
        Event::UserEvent(()) => {
            requested = Some(Instant::now());
            // A burst must coalesce, not enqueue 100 native paint callbacks.
            for _ in 0..100 {
                window.request_redraw();
            }
        }
        Event::Window(_, WindowEvent::RedrawRequested) => {
            let elapsed = requested.take().map(|start| start.elapsed());
            assert!(canvas.draw(|ctx| {
                ctx.set_fill_style(Color::rgb(30, 120, 200));
                ctx.fill_rect(0.0, 0.0, ctx.width(), ctx.height());
            }));
            if let Some(elapsed) = elapsed {
                samples.push(elapsed);
                if samples.len() == 120 {
                    let samples = &mut samples[20..];
                    samples.sort_unstable();
                    let median = samples[50].as_secs_f64() * 1000.0;
                    let p95 = samples[95].as_secs_f64() * 1000.0;
                    println!("redraw: 100 samples after 20 warmup, median {median:.2} ms, p95 {p95:.2} ms");
                    completed.set(true);
                    window.close();
                    return;
                }
            }
            if !started || elapsed.is_some() {
                started = true;
                next.send(()).expect("redraw worker");
            }
        }
        _ => {}
    });
    worker.join().expect("redraw worker");
    assert!(result.get(), "redraw benchmark did not finish");
}
