# Bassie Canvas Rust library

Cross-platform 2D drawing for [bwindow](../bwindow) windows using the system-native
graphics API: Core Graphics on macOS, Direct2D on Windows, and Cairo on Linux.

## Features

- Paths, clipping, transforms, solid RGBA colors, and text
- Logical pixel coordinates and native pointer cursors
- One event loop shared with window and webview content

## Getting Started

```rust,no_run
use bcanvas::{CanvasBuilder, Color};
use bwindow::{Event, EventLoopBuilder, WindowBuilder, WindowEvent};

let event_loop = EventLoopBuilder::new().build();
let window = WindowBuilder::new().title("Canvas").build();
let mut canvas = CanvasBuilder::new(&window).build();

event_loop.run(move |event| {
    if let Event::Window(id, WindowEvent::RedrawRequested) = event {
        if id == window.id() {
            canvas.draw(|ctx| {
                ctx.set_fill_style(Color::rgb(30, 120, 200));
                ctx.fill_rect(0.0, 0.0, ctx.width(), ctx.height());
            });
        }
    }
});
```

Drawing state resets each frame; contexts cannot outlive the drawing callback.
Use `request_redraw()` for state changes or `request_animation_frame()` for
display-synchronized animation. Schedule the next animation frame from its
`RedrawRequested` callback. Drawing after the window closes returns `false`.

Keyboard and pointer events use bwindow input types. Images and IME text editing
are not supported.

## Canvas 2D API

The context follows
[Canvas 2D](https://html.spec.whatwg.org/multipage/canvas.html) method names in
Rust's snake_case. Drawing properties use getter/setter pairs such as
`fill_style()` / `set_fill_style(color)` and `global_alpha()` /
`set_global_alpha(alpha)`. Invalid alpha and line-width values are ignored,
preserving the current value. `save()` / `restore()` include drawing styles,
transforms, and clipping, but not the current path.

This is a native subset, not a browser compatibility layer:

- Colors and style choices use `Color` and enums instead of CSS strings.
- `set_font(family, size)` and `set_font_weight(weight)` use typed font settings;
  size is in logical pixels. Read them with `font_family()`, `font_size()`, and
  `font_weight()`.
- `measure_text(text)` returns width, not a browser `TextMetrics` object.
- `ellipse(x, y, rx, ry)` adds a full axis-aligned ellipse, and `round_rect` accepts
  one corner radius.
- Text direction is left-to-right; `Start` and `End` map to left and right.
  Native text baselines are approximations, not browser-identical typography.
- The context is only available inside `canvas.draw`, called from
  `WindowEvent::RedrawRequested`.

## Cursors

Use `canvas.set_cursor(CursorIcon::Pointer)` for clickable content, `Default`
for the arrow, or `Progress` for background work. macOS has no public busy cursor,
so `Progress` currently uses the arrow there.
Cursor changes apply only over the canvas and are ignored after close.

## Offscreen drawing

`OffscreenCanvas` uses the same drawing API without occupying a window. Its
tightly packed, straight-alpha RGBA8 pixels can be encoded, processed on the CPU,
or uploaded directly with `wgpu::Queue::write_texture`:

```rust
use bcanvas::{Color, OffscreenCanvas};

let mut canvas = OffscreenCanvas::new(128, 32);
canvas.draw(|ctx| {
    ctx.set_fill_style(Color::rgb(255, 255, 255));
    ctx.fill_text("60 FPS", 8.0, 20.0);
});
let rgba = canvas.pixels();
assert_eq!(rgba.len(), 128 * 32 * 4);
```

## License

Copyright © 2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
