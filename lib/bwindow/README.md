# Bassie Window Rust library

Native windows and one main event loop for Rust. Supports AppKit, Win32, and GTK 3.
Window contents are provided by sibling crates `bwebview` and `bcanvas`.

## Getting Started

```rust,no_run
use bwindow::{EventLoopBuilder, WindowBuilder};

let event_loop = EventLoopBuilder::new().build();
let window = WindowBuilder::new().title("Window").build();

event_loop.run(move |_event| {
    // Keep the window alive until the event loop exits.
    let _ = &window;
});
```

Use `EventLoopBuilder::new().with_user_event::<AppEvent>()` for typed worker and
content events. Every window event carries a `WindowId`. Closing the last window
or calling a proxy's `exit()` returns from `run`; sends after shutdown fail.

A window accepts one content attachment. Its native handle and resize/close
hooks let content backends integrate without accessing private window state.
Dropping a window closes it and invalidates its attachment.
Window setters do nothing after close, and geometry getters return zero. Use
`is_closed()` to distinguish a closed window.

## Input Events

Native input uses DOM-like names with typed Rust payloads:

| Window event               | DOM equivalent             |
| -------------------------- | -------------------------- |
| `KeyDown` / `KeyUp`        | `keydown` / `keyup`        |
| `MouseDown` / `MouseUp`    | `mousedown` / `mouseup`    |
| `MouseMove` / `MouseLeave` | `mousemove` / `mouseleave` |
| `Wheel`                    | `wheel`                    |
| `Focus` / `Blur`           | `focus` / `blur`           |

`KeyboardEvent.code` identifies the physical key, while `key` is layout-dependent.
`KeyCode::KeyA` corresponds to DOM `code = "KeyA"`, not the deprecated numeric
`keyCode`. `repeat` marks auto-repeat presses. Press/release is expressed by the
event variant, not duplicated in its payload.

```rust,no_run
use bwindow::{Event, EventLoopBuilder, Key, WindowEvent};

let event_loop = EventLoopBuilder::new().build();
event_loop.run(|event| {
    if let Event::Window(_, WindowEvent::KeyDown(key)) = event {
        if !key.repeat && matches!(&key.key, Key::Character(text) if text == "r") {
            println!("R pressed");
        }
        if key.modifiers.meta_key() {
            println!("Meta held");
        }
    }
});
```

Modifiers provide `shift_key()`, `ctrl_key()`, `alt_key()`, and `meta_key()`.
`Modifiers::META` means Command on macOS and Windows/Super elsewhere; `COMMAND`,
`SUPER`, and `OPTION` remain aliases for native terminology.

Mouse positions are logical client-area pixels, analogous to `clientX`/`clientY`.
Wheel deltas carry pixel or line units, with positive values scrolling right/down.
Close requests support `prevent_default()` and `default_prevented()`; other events
are not cancelable. There is no DOM tree, capture/bubbling, synthesized `click`, or
IME text-input API. Key mappings are a subset of DOM values, and modifier/dead-key
reporting is not yet browser-equivalent. `MouseButton::Other` retains native button
numbers rather than DOM `button` values.

`RedrawRequested` and `CloseRequested` retain native window terminology: they are
not DOM `requestAnimationFrame` or `beforeunload` events.

## Window Content

- Empty window: use `WindowBuilder` alone.
- Native 2D drawing: attach `bcanvas::CanvasBuilder` and paint during that window's
  `RedrawRequested` callback.
- Web content: attach `bwebview::WebviewBuilder` and map browser notifications into
  your application's typed events with `on_event(proxy, mapper)`.

The siblings can share one event loop without depending on each other.
Run `cargo run -p bwebview --example bwebview-canvas` for a mixed-content example;
add `-- --smoke` to exercise IPC and closing the browser before the canvas.

## Features

- `remember_window_state` (default): persist native window placement.
- `dialog`: native message and file dialogs.
- `file_drop`: native file-drop events.
- `menu`: custom macOS menus.
- `progress_bar`: Windows taskbar and GTK launcher progress.

## Linux Dependencies

Install the GTK 3 runtime libraries (GTK 3.18 or newer). Window-only applications
do not require WebKitGTK. Set `BWINDOW_LIB_DIR` to a nonstandard library directory.

## License

Copyright © 2025-2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
