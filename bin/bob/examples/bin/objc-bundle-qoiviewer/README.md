# objc-bundle-qoiviewer

A macOS document-based application that opens and displays QOI image files.
Built using the [bob](../../..) build system with Objective-C and Cocoa.

## Features

- Opens `.qoi` files via File > Open or by double-clicking in Finder
- Transparent images rendered correctly (sRGB, non-premultiplied alpha)
- Pure C QOI decoder (`src/qoi.h`) - no third-party dependencies
- Universal binary (x86_64 + aarch64) via `lipo`
- App sandbox entitlements (`entitlements.plist`)
- Targets macOS 12.0+

## Build and run

```sh
bob build
open target/debug/QoiViewer.app
```

## Install with Quick Look extensions

Two companion extensions live alongside this project:
- `../objc-appex-qoiviewer-ql` -- Quick Look **preview** (Space bar panel)
- `../objc-appex-qoiviewer-ql-thumb` -- Quick Look **thumbnails** (Finder icon/gallery view)

Run the following from the `objc-bundle-qoiviewer` directory:

```sh
# 1. Build all three projects
(cd ../objc-appex-qoiviewer-ql       && bob build)
(cd ../objc-appex-qoiviewer-ql-thumb && bob build)
bob build

# 2. Embed extensions (sign inside-out: extensions first, then app)
APP=target/debug/QoiViewer.app
rm -rf "$APP/Contents/PlugIns"
mkdir -p "$APP/Contents/PlugIns"

ditto ../objc-appex-qoiviewer-ql/target/debug/QoiViewerQL.appex \
      "$APP/Contents/PlugIns/QoiViewerQL.appex"
ditto ../objc-appex-qoiviewer-ql-thumb/target/debug/QoiViewerQLThumb.appex \
      "$APP/Contents/PlugIns/QoiViewerQLThumb.appex"

codesign --force --sign - \
  --entitlements ../objc-appex-qoiviewer-ql/entitlements.plist \
  "$APP/Contents/PlugIns/QoiViewerQL.appex"
codesign --force --sign - \
  --entitlements ../objc-appex-qoiviewer-ql-thumb/entitlements.plist \
  "$APP/Contents/PlugIns/QoiViewerQLThumb.appex"
codesign --force --sign - --entitlements entitlements.plist "$APP"

# 3. Install and register with macOS
mkdir -p ~/Applications
rm -rf ~/Applications/QoiViewer.app
ditto "$APP" ~/Applications/QoiViewer.app
pluginkit -a ~/Applications/QoiViewer.app
open ~/Applications/QoiViewer.app   # triggers Launch Services registration
qlmanage -r
```

After installing, Finder shows QOI previews via Space bar and thumbnails in icon/gallery view.

## License

Copyright (c) 2025 Bastiaan van der Plaat  
Licensed under the [MIT](../../../../../LICENSE) license.
