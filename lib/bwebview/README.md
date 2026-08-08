# Bassie Webview Rust library

A cross-platform webview library for Rust with minimal dependencies

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

| Platform    | Backend                        | Notes                                     |
| ----------- | ------------------------------ | ----------------------------------------- |
| Windows     | WebView2 (Chromium/Edge)       | Requires WebView2 Runtime to be installed |
| macOS       | WKWebView (WebKit)             | macOS 11.0+                               |
| Linux/other | WebKitGTK (GTK 3 + WebKit2GTK) | See GTK tiers below                       |

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
