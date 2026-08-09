# Bassie Webview Rust library

A cross-platform native window library for Rust with Webview and Canvas views.

## Canvas

`Canvas` provides an HTML5 Canvas 2D-inspired API using Cairo on GTK, Core Graphics on macOS,
and Direct2D/DirectWrite on Windows. Drawing is delivered through the existing event loop:

```rust,no_run
use bwebview::{CanvasBuilder, Color, CanvasEvent, Event, EventLoop, WindowBuilder};

let event_loop = EventLoop::new();
let window = WindowBuilder::new().title("Canvas").build();
let mut canvas = CanvasBuilder::new(&window).build();
event_loop.run(move |event| {
    if let Event::Canvas(CanvasEvent::Draw(context)) = event {
        // Frame dimensions are logical pixels; scale_factor() reports native pixels per point.
        context.begin_path();
        context.round_rect(20.0, 20.0, context.width() - 40.0, 120.0, 16.0);
        context.set_fill_style(Color::rgb(30, 120, 80));
        context.fill();
        canvas.request_animation_frame();
    }
});
```

The API also includes path primitives, text, transforms, typed user-event payloads, standard macOS
menu roles, and window cursor selection. Mouse and keyboard input is reported through DOM-style
`WindowEvent` variants. Run the complete native Reversi example with
`cargo run -p bwebview --example bwebview-canvas-reversi --features menu`.
For a visual tour of every drawing primitive and animation, run
`cargo run -p bwebview --example bwebview-canvas-showcase`.

Application windows that follow the system appearance receive `WindowEvent::ThemeChange(Theme)` when the
desktop switches between light and dark mode. Explicit themes set through `WindowBuilder::theme`
or `Window::set_theme` remain fixed.

## Linux runtime dependencies

The Linux backend requires the GTK 3 and WebKitGTK runtime libraries.

### Debian / Ubuntu

Ubuntu 22.04 and other distributions using the original GTK 3 package names:

```sh
sudo apt install libgtk-3-0 libwebkit2gtk-4.1-0
```

Newer distributions using 64-bit `time_t` package names:

```sh
sudo apt install libgtk-3-0t64 libwebkit2gtk-4.1-0
```

Older distributions with WebKitGTK 4.0 can use:

```sh
sudo apt install libgtk-3-0 libwebkit2gtk-4.0-37
```

### Fedora

```sh
sudo dnf install gtk3 webkit2gtk4.1
```

Older Fedora installations with WebKitGTK 4.0 can use:

```sh
sudo dnf install gtk3 webkit2gtk4.0
```

## Platforms

| Platform    | Webview backend                | Canvas backend           | Notes                                     |
| ----------- | ------------------------------ | ------------------------ | ----------------------------------------- |
| Windows     | WebView2 (Chromium/Edge)       | Direct2D + DirectWrite   | Requires WebView2 only for Webview builds |
| macOS       | WKWebView (WebKit)             | Core Graphics + AppKit   | macOS 11.0+                               |
| Linux/other | WebKitGTK (GTK 3 + WebKit2GTK) | GTK 3 + Cairo            | See GTK tiers below                       |

### Linux / GTK tiers

The Linux backend automatically selects the best available WebKitGTK version at build time:

| WebKitGTK package | Min version | GTK min | Distro (stock packages) | Notes                                    |
| ----------------- | ----------- | ------- | ----------------------- | ---------------------------------------- |
| `webkit2gtk-4.1`  | 2.40        | 3.22+   | Ubuntu 22.04+           | Modern API; full custom-protocol support |
| `webkit2gtk-4.0`  | 2.22        | 3.18+   | Ubuntu 18.04+           | JSC GLib API; URI-only custom protocol   |
| `webkit2gtk-4.0`  | 2.20        | 3.18+   | Ubuntu 16.04+           | Legacy JavaScriptCore C API              |

## Screenshots

<table>
<tr>
<td align="center">
<img src="docs/images/screenshots/windows.png" alt="ipc example running on Windows" width="300">
<br>
<a href="examples/ipc/">IPC example</a> running on Windows
</td>
<td align="center">
<img src="docs/images/screenshots/macos.png" alt="ipc example running on macOS" width="300">
<br>
<a href="examples/ipc/">IPC example</a> running on macOS
</td>
<td align="center">
<img src="docs/images/screenshots/gtk.png" alt="ipc example running on Linux (GTK)" width="300">
<br>
<a href="examples/ipc/">IPC example</a> running on Linux (GTK)
</td>
</tr>
</table>

## Features

- **canvas** Enables the native Canvas view and drawing backend (default).
- **webview** Enables the native Webview view and web-engine backend (default).
- **log** Enables logging support by forwarding `console.*` calls to the `log` crate (default).
- **remember_window_state** Adds remembers window position and size between launches options (default).
- **custom_protocol** Adds support for custom protocols, allowing you to serve content from custom URL schemes.
- **dialog** Adds support for native message and file dialogs.
- **file_drop** Adds support for dropping files onto the window, reported as `WindowEvent::DroppedFile`.
- **menu** Adds support for custom macOS menu bar entries, selections are reported as `Event::MacosMenuItem`.
- **progress_bar** Adds support for the Windows taskbar and GTK application launcher progress bars.
- **rust-embed** Adds support for serving embedded assets using the `rust-embed` crate.

## Sources binary blobs

- `webview2/{arm64, x64, x86}/` [Microsoft.Web.WebView2 nuget](https://www.nuget.org/packages/Microsoft.Web.WebView2/)
- `webview2/*.winmd` [Microsoft.Web.WebView2 win32 windmd generator](https://github.com/wravery/webview2-win32md/tree/main)

## License

Copyright © 2025-2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
