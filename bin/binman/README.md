# Binman

A safe, minimal and open source Windows cleanup tool.

Binman scans disposable Windows data, application caches and developer tooling, then asks for confirmation before deleting anything. It measures file counts and sizes when doing so is reasonably fast; tool-managed operations such as DISM, Docker and old Rust toolchains are clearly marked when their size is not estimated.

Cleanup behavior is defined primarily by the bundled [`rules.json`](rules.json) catalog. Binman deliberately avoids registry cleaning, browser history, cookies, credentials, personal downloads, restore points, drivers and Windows update rollback data.

## Features

- Scan-only preview with per-category sizes and file counts when available
- Windows, browser, application, gaming and developer caches
- All detected cleanup categories selected by default, with full control before cleaning
- Impact, recovery and side-effect warnings for expensive or irreversible cleanup
- Review and explicit confirmation before permanent cleanup
- Live scan and cleanup progress with skipped-file reporting
- Active-process checks that skip cleanup when application state cannot be verified safely
- Optional administrator restart for protected Windows cleanup rules
- Safe DISM component cleanup without `/ResetBase`
- Rust toolchain cleanup that keeps the latest stable and nightly plus active, default and project-pinned toolchains
- Declarative, validated cleanup rules with specialized code only for system and tool APIs
- Native light and dark themes using `bwebview` and Petite Vue
- No telemetry, advertisements or bundled offers; Binman makes no network requests itself, although applications may later redownload removed caches

## License

Copyright © 2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
