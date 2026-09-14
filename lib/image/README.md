# Image

A small, safe Rust decoder for JPEG, PNG/APNG, GIF, BMP, ICO, QOI, SVG (subset), and TinyVG
images. Raster decoding returns straight-alpha RGBA8 pixels; vector decoding returns
a backend-neutral display list.

## Example

```rs
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

The decoder supports common JPEG, PNG/APNG, GIF, BMP, ICO, and QOI variants,
including progressive JPEG, Adam7 PNG, GIF interlacing, BMP bitfields/RLE, and
animation disposal. PNG samples wider than 8 bits are reduced to the same RGBA8
output used by the other raster formats. ICO decoding selects its largest image
and supports embedded PNG or the same DIB variants as the BMP decoder. It does
not support HDR output, encoding, incremental decoding, or ICC color management.
Decoding has a cumulative 512 MiB allocation limit.
TinyVG variable-width path strokes preserve their width transitions as tapered,
round-capped outlines.

All formats are enabled by default. Disable default features and select from
`qoi`, `jpeg`, `png`, `gif`, `bmp`, `ico`, `svg`, and `tinyvg` to build only the
required decoders.

## SVG support

The decoder supports this practical static SVG subset:

- Documents: static UTF-8 SVG with root or nested viewports, `viewBox`,
  `preserveAspectRatio`, the standard 300x150 default viewport, zero-sized
  view-box suppression, clipping, percentages, and standard units.
- Geometry: SVG 1.1 path commands with valid-prefix recovery, rectangles, circles,
  ellipses, lines, polygons, polylines, CSS geometry properties, path-length dash
  normalization on paths and shapes, transform lists, and CSS
  transforms with transform origins.
- Appearance: inherited presentation attributes, inline `style=""` values, and
  bounded embedded stylesheets with basic type, class, ID, compound, and direct-child selectors
  for fills, strokes, opacity, visibility, fill rules, caps, joins, dashes,
  complete inherited paint order, z-index ordering, overflow, transforms, blend
  modes, isolation, and gradient color interpolation.
- Paint and reuse: CSS colors, paint fallbacks, inherited linear or radial gradient
  templates, local `defs`, viewport-aware `symbol` and `use`, clip paths, and nested
  alpha or luminance masks, and bounded, viewport-clipped local markers with
  context paint.

Unsupported content is skipped when the XML is valid. This includes text, images,
filters, patterns, complex or external stylesheets, animation, scripts
and events, links, `foreignObject`, and external resources.

## Tests

The raster tests use a compact subset of the image-rs test corpus pinned at
`6812e732343b0eaaeeef90ca751dd0e73430fb15`. Its independently generated PNG
references were converted once to flat RGBA8 files, so tests compare expected
pixels directly without another decoder or test dependency. The checked-in
subset covers
16-bit and interlaced PNG, APNG disposal and blending, GIF animation and
interlacing, BMP depth/bitfield/top-down variants, ICO masks and PNG payloads,
progressive JPEG, and QOI. Inputs are stored below `tests/images` and flat RGBA8
output below `tests/reference`. The QOI cases include `edgecase`, `qoi_logo`,
`testcard`, and `testcard_rgba` from the format project's published
[`qoi_test_images.zip`](https://qoiformat.org/qoi_test_images.zip), identified by
SHA-256 `bd557fb208222478d9eefcae59fb473d10e047fd7a8885fcff48861f86599165`.
The runner discovers each format directory and pairs inputs with either a
same-name `.rgba` file or sorted `.anim_NN.rgba` frames. Adding a fixture does not
require another Rust path table; pinned per-format counts catch accidental corpus
changes and every reference must be used.

The normal offline SVG tests also include all 93 currently passing WPT reftests
whose test and SVG reference total at most 8 KiB. They are pinned at WPT revision
`987a2d0c1a45f1a193f1f05d9508c4efc307c3bf`. Test documents are stored below
`tests/images/svg`, and their 41 shared or named SVG references are stored below
`tests/reference/svg`. The image crate validates every declared pair and the
macOS renderer performs WPT-compatible pixel comparisons, including each test's
fuzzy tolerance.

The TinyVG corpus contains the official shield, app icon, flowchart, chart,
comic, tiger, and `everything` examples with losslessly optimized copies of their
published PNG references. The website examples are pinned at revision
`e7c4c624fbe9276740fff2a9b6231ff86e01a89c`; `everything` is pinned from the
examples repository at revision `b8d8c7e88ed221f2ce1100f9e25b5c6e7e6dc78d`.
The image crate validates each TinyVG document and its reference dimensions, and
the macOS renderer renders every document and compares its pixels with the
published PNG reference.

## License

Copyright (c) 2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
