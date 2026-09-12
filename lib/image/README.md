# Image

A small, safe Rust decoder for JPEG, PNG/APNG, GIF, BMP, ICO, QOI, SVG (subset), and TinyVG
images. Raster decoding returns straight-alpha RGBA8 pixels; vector decoding returns
a backend-neutral display list.

## Example

```rust
let bytes = std::fs::read("picture.png").expect("read image");
let image = image::decode(&bytes).expect("decode image");
println!("{}x{}", image.width(), image.height());
for frame in image.frames() {
    println!("{} RGBA bytes, {:?}", frame.pixels().len(), frame.delay());
}
```

Each animation frame contains the complete canvas after blending and before
disposal. Static images have one frame. Animation delays and loop counts are
preserved, and JPEG EXIF orientation is applied to the pixels and dimensions.

The decoder supports common 8-bit JPEG, PNG/APNG, GIF, BMP, ICO, and QOI variants,
including progressive JPEG, Adam7 PNG, GIF interlacing, BMP bitfields/RLE, and
animation disposal. ICO decoding selects its largest image and supports embedded
PNG or the same DIB variants as the BMP decoder. It does not support 16-bit
PNG/APNG, HDR output, encoding, incremental decoding, or ICC color management.
Decoding has a cumulative 512 MiB allocation limit.

All formats are enabled by default. Disable default features and select from
`qoi`, `jpeg`, `png`, `gif`, `bmp`, `ico`, `svg`, and `tinyvg` to build only the
required decoders.

## SVG support

The decoder supports this practical static SVG subset:

- Documents: static UTF-8 SVG with root or nested viewports, `viewBox`,
  `preserveAspectRatio`, clipping, percentages, and standard units.
- Geometry: all path commands, rectangles, circles, ellipses, lines, polygons,
  polylines, and complete transform lists.
- Appearance: inherited presentation attributes and inline `style=""` values for
  fills, strokes, opacity, visibility, fill rules, caps, joins, and dashes.
- Paint and reuse: CSS colors, linear or radial gradients, local `defs` and `use`,
  clip paths, and nested alpha or luminance masks.

Unsupported content is skipped when the XML is valid. This includes text, images,
filters, patterns, markers, stylesheets, blend modes, animation, scripts and events,
links, `foreignObject`, and external resources.

## Tests

Run `cargo run -p image --bin generate-fixtures` to regenerate fixtures with
ImageMagick and libjpeg-turbo. The checked-in fixtures let normal builds and the
test suite run without those tools. Pass `-- --ico-only` to regenerate only the
ICO fixtures.

## License

Copyright (c) 2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
