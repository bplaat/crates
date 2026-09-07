/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Desktop smoke test for independent frames, teardown, and proxy lifetime.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use bcanvas::{
    CanvasBuilder, CanvasRenderingContext2d, Color, CursorIcon, FontWeight, LineCap, LineJoin,
    TextAlign, TextBaseline,
};
use bwindow::{Event, EventLoopBuilder, WindowBuilder, WindowEvent};

enum AppEvent {
    AfterPaint,
}

fn main() {
    let event_loop = EventLoopBuilder::new()
        .with_user_event::<AppEvent>()
        .build();
    let proxy = event_loop.create_proxy();
    let watchdog = event_loop.create_proxy();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(5));
        let _ = watchdog.exit();
    });
    let mut first = WindowBuilder::new()
        .title("Canvas lifecycle: first")
        .build();
    let mut last = WindowBuilder::new().title("Canvas lifecycle: last").build();
    let first_id = first.id();
    let last_id = last.id();
    // Closing an unshown window must not stop the other pre-run windows.
    let mut transient = WindowBuilder::new().title("Already closed").build();
    transient.close();
    assert!(transient.is_closed());
    let mut first_canvas = Some(CanvasBuilder::new(&first).build());
    let mut last_canvas = CanvasBuilder::new(&last).build();
    last_canvas.set_cursor(CursorIcon::Progress);
    assert!(!last_canvas.draw(|_| panic!("outside paint")));
    assert!(CanvasBuilder::new(&first).try_build().is_err());
    let completed = Rc::new(Cell::new(false));
    let result = completed.clone();
    let mut painted_first = false;
    let mut painted_last = false;
    let mut closing = false;
    event_loop.run(move |event| match event {
        Event::Window(id, WindowEvent::RedrawRequested) if id == first_id => {
            let canvas = first_canvas.as_mut().expect("live first canvas");
            canvas.set_cursor(CursorIcon::Pointer);
            assert!(canvas.draw(|ctx| {
                check_drawing_state(ctx);
                ctx.set_fill_style(Color::rgb(30, 120, 200));
                ctx.fill_rect(0.0, 0.0, ctx.width(), ctx.height());
                ctx.save(); // An unbalanced save is unwound at the end of the frame.
                ctx.translate(10.0, 20.0);
            }));
            assert!(!canvas.draw(|_| panic!("frame already consumed")));
            painted_first = true;
            proxy.send_user_event(AppEvent::AfterPaint).expect("proxy");
        }
        Event::Window(id, WindowEvent::RedrawRequested) if id == last_id => {
            last_canvas.set_cursor(CursorIcon::Default);
            assert!(last_canvas.draw(|ctx| {
                ctx.set_fill_style(Color::rgb(200, 80, 40));
                ctx.fill_rect(0.0, 0.0, ctx.width(), ctx.height());
            }));
            painted_last = true;
            proxy.send_user_event(AppEvent::AfterPaint).expect("proxy");
        }
        Event::UserEvent(AppEvent::AfterPaint) => {
            assert!(!last_canvas.draw(|_| panic!("native paint scope leaked")));
            if closing {
                // A queued proxy event still reaches the remaining window.
                last.close();
                assert!(last.is_closed());
                last.set_title("Closed");
                last.set_size(bwindow::LogicalSize::new(200.0, 200.0));
                last.set_background_color(0xffffff);
                assert_eq!(last.size().width, 0.0);
                assert_eq!(last.position().x, 0.0);
                last_canvas.request_redraw();
                last_canvas.set_cursor(CursorIcon::Pointer);
                assert!(!last_canvas.draw(|_| panic!("closed canvas")));
                completed.set(true);
            } else if painted_first && painted_last {
                drop(first_canvas.take());
                first.close();
                first.request_redraw();
                closing = true;
                proxy
                    .send_user_event(AppEvent::AfterPaint)
                    .expect("first close stopped loop");
            }
        }
        _ => {}
    });
    assert!(result.get(), "canvas lifecycle smoke test timed out");
}

fn check_drawing_state(ctx: &mut CanvasRenderingContext2d<'_>) {
    check_default_drawing_state(ctx);
    ctx.restore(); // An empty stack does not change the defaults.
    check_default_drawing_state(ctx);
    ctx.save();
    ctx.set_fill_style(Color::rgb(255, 0, 0));
    ctx.set_stroke_style(Color::rgb(0, 0, 255));
    ctx.set_global_alpha(0.5);
    ctx.set_line_width(0.5);
    ctx.set_line_cap(LineCap::Round);
    ctx.set_line_join(LineJoin::Bevel);
    ctx.set_font("monospace", 0.5);
    ctx.set_font_weight(FontWeight::Bold);
    ctx.set_text_align(TextAlign::Center);
    ctx.set_text_baseline(TextBaseline::Middle);
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, 2.0] {
        ctx.set_global_alpha(value);
        assert_eq!(ctx.global_alpha(), 0.5);
    }
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, 0.0] {
        ctx.set_line_width(value);
        ctx.set_font("serif", value);
        assert_eq!(ctx.line_width(), 0.5);
        assert_eq!(ctx.font_family(), "monospace");
        assert_eq!(ctx.font_size(), 0.5);
    }
    assert_eq!(ctx.fill_style(), Color::rgb(255, 0, 0));
    assert_eq!(ctx.stroke_style(), Color::rgb(0, 0, 255));
    assert_eq!(ctx.line_cap(), LineCap::Round);
    assert_eq!(ctx.line_join(), LineJoin::Bevel);
    assert_eq!(ctx.font_weight(), FontWeight::Bold);
    assert_eq!(ctx.text_align(), TextAlign::Center);
    assert_eq!(ctx.text_baseline(), TextBaseline::Middle);
    ctx.set_global_alpha(0.0);
    assert_eq!(ctx.global_alpha(), 0.0);
    ctx.set_global_alpha(1.0);
    assert_eq!(ctx.global_alpha(), 1.0);
    ctx.restore();
    check_default_drawing_state(ctx);
}

fn check_default_drawing_state(ctx: &CanvasRenderingContext2d<'_>) {
    assert_eq!(ctx.fill_style(), Color::rgb(0, 0, 0));
    assert_eq!(ctx.stroke_style(), Color::rgb(0, 0, 0));
    assert_eq!(ctx.global_alpha(), 1.0);
    assert_eq!(ctx.line_width(), 1.0);
    assert_eq!(ctx.line_cap(), LineCap::Butt);
    assert_eq!(ctx.line_join(), LineJoin::Miter);
    assert_eq!(ctx.font_family(), "sans-serif");
    assert_eq!(ctx.font_size(), 10.0);
    assert_eq!(ctx.font_weight(), FontWeight::Normal);
    assert_eq!(ctx.text_align(), TextAlign::Start);
    assert_eq!(ctx.text_baseline(), TextBaseline::Alphabetic);
}
