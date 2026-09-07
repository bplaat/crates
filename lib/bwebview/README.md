# Bassie Webview Rust library

A cross-platform webview library for Rust with minimal dependencies.

Windows and the main event loop are provided by [bwindow](../bwindow). Use
[bcanvas](../bcanvas) for native 2D content; browser-only applications do not need it.

## Getting Started

Map browser notifications into typed application messages:

```rust,no_run
use bwindow::{Event, EventLoopBuilder, WindowBuilder, WindowId};
use bwebview::{WebviewBuilder, WebviewEvent};

enum AppEvent {
    Browser(WindowId, WebviewEvent),
}

let event_loop = EventLoopBuilder::new().with_user_event::<AppEvent>().build();
let mut window = WindowBuilder::new().title("Browser").build();
let _webview = WebviewBuilder::new(&window)
    .on_event(event_loop.create_proxy(), AppEvent::Browser)
    .load_url("https://example.com")
    .build();

event_loop.run(move |event| {
    if let Event::UserEvent(AppEvent::Browser(id, WebviewEvent::PageTitleChange(title))) = event {
        if id == window.id() {
            window.set_title(title);
        }
    }
});
```

Windowing and the `dialog`, `menu`, `progress_bar`, and `remember_window_state`
features belong to `bwindow`. Browser notifications are optional when no mapper
is installed. The `file_drop` feature forwards to
`bwindow/file_drop` and also enables handling drops on browser content.

A window accepts one content attachment. Closing or dropping the window tears
down its browser content; later browser operations do nothing and `url()` returns
`None`. The [lifecycle example](examples/bwebview-lifecycle.rs) exercises both drop
orders and proxy delivery after the first window closes.
See the [mixed-content example](examples/bwebview-canvas.rs) for browser and canvas
windows sharing one event loop.

## Linux Dependencies

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

| Platform    | Backend                        | Notes                                     |
| ----------- | ------------------------------ | ----------------------------------------- |
| Windows     | WebView2 (Chromium/Edge)       | Requires WebView2 Runtime to be installed |
| macOS       | WKWebView (WebKit)             | macOS 11.0+                               |
| Linux/other | WebKitGTK (GTK 3 + WebKit2GTK) | See GTK tiers below                       |

### Linux / GTK Tiers

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
<a href="./examples/bwebview-ipc.rs">IPC example</a> running on Windows
</td>
<td align="center">
<img src="docs/images/screenshots/macos.png" alt="ipc example running on macOS" width="300">
<br>
<a href="./examples/bwebview-ipc.rs">IPC example</a> running on macOS
</td>
<td align="center">
<img src="docs/images/screenshots/gtk.png" alt="ipc example running on Linux (GTK)" width="300">
<br>
<a href="./examples/bwebview-ipc.rs">IPC example</a> running on Linux (GTK)
</td>
</tr>
</table>

## Features

- **log** - Enables logging support by forwarding `console.*` calls to the `log` crate (default).
- **custom_protocol** - Adds support for serving content from custom URL schemes.
- **file_drop** - Adds support for dropping files onto the window, reported as `WindowEvent::DroppedFile`.
- **rust-embed** - Adds support for serving embedded assets using the `rust-embed` crate.

## License

Copyright © 2025-2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
