# Bassie Window Rust library

Cross-platform native windows and a main event loop for Rust, using AppKit on macOS,
Win32 on Windows, and GTK 3 on Linux. Add [bcanvas](../bcanvas) for native 2D
drawing or [bwebview](../bwebview) for web content.

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
content events. Every window event includes a `WindowId`. Closing the last window
or calling a proxy's `exit()` stops the event loop.

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
are not cancelable. Key mappings are a subset of DOM values, and IME text input is
not yet supported.

Use `request_redraw()` for state changes and `request_animation_frame()` for one
display-synchronized animation frame. Requests made before an animation frame is
delivered coalesce. Schedule the next frame from its `RedrawRequested` callback
to animate continuously at the display's native cadence.

## Window Content

- Empty window: use `WindowBuilder` alone.
- Native 2D drawing: attach `bcanvas::CanvasBuilder` and paint during that window's
  `RedrawRequested` callback.
- Web content: attach `bwebview::WebviewBuilder` and map browser notifications into
  your application's typed events with `on_event(proxy, mapper)`.

## Features

- `remember_window_state` (default): persist native window placement.
- `dialog`: native message and file dialogs.
- `file_drop`: native file-drop events.
- `menu`: custom macOS menus.
- `progress_bar`: Windows taskbar and GTK launcher progress.

## Linux

The Linux backend requires the GTK 3.18 or newer runtime library. Development
headers are not required.

Debian / Ubuntu:

```sh
sudo apt install libgtk-3-0
```

On distributions using 64-bit `time_t`, install `libgtk-3-0t64` instead.

Fedora:

```sh
sudo dnf install gtk3
```

## License

Copyright © 2025-2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
