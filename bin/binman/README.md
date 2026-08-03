# Binman

A safe, minimal and open source Windows cleanup tool.

Binman scans regenerable system and application caches, shows exactly how much space each category uses, and asks for confirmation before deleting anything. It uses an audited bundled catalog and deliberately avoids registry cleaning, browser history, cookies, credentials, downloads, restore points, drivers and Windows update rollback data.

## Features

- Scan-only preview with per-category sizes and file counts
- Windows, browser, application and developer caches
- Review and confirmation before permanent cleanup
- Live scan and cleanup progress with skipped-file reporting
- Safe DISM component cleanup without `/ResetBase`
- Native light and dark themes using `bwebview` and Petite Vue
- No telemetry, advertisements, bundled offers or network access

## Development

Run Binman normally while developing:

```sh
cargo run -p binman
```

Binman starts without a UAC prompt. Rules that modify protected system data are disabled unless Binman was explicitly started as Administrator.

## License

Copyright © 2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
