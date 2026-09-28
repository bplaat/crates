# Image

A safe Rust decoder for JPEG, PNG/APNG, GIF, BMP, ICO, QOI, SVG (subset), and
TinyVG. Raster images decode to straight-alpha RGBA8; vector images decode to a
backend-neutral display list.

## Usage

```rs
let bytes = std::fs::read("picture.png").expect("read image");
let image = image::decode(&bytes).expect("decode image");
for frame in image.frames() {
    let bitmap = frame.bitmap();
    println!("{}x{}: {:?}", bitmap.width(), bitmap.height(), frame.delay());
}
```

Animation frames contain the full canvas after blending and before disposal.
Static images have one frame. The decoder supports progressive JPEG, Adam7 PNG,
GIF interlacing, BMP bitfields and RLE, and animation disposal. It applies JPEG
EXIF orientation and converts 16-bit PNG samples to RGBA8. ICO selects the
largest embedded image. Decoding has a cumulative 1 GiB allocation limit.

All formats are enabled by default. Use `default-features = false` with the
`qoi`, `jpeg`, `png`, `gif`, `bmp`, `ico`, `svg`, or `tinyvg` features to select
decoders. Encoding, incremental decoding, HDR output, and ICC color management
are not supported.

## SVG subset

Static SVG supports common geometry, viewports, transforms, CSS presentation
styles, gradients, clipping, masks, markers, and paint order. Valid but
unsupported content, including text, images, filters, scripts, animation, and
external resources, is skipped.

## Tests and benchmarks

Run `cargo test -p image` and `cargo bench -p image --bench image`. Raster tests
compare decoded pixels against references in `tests/reference`. Fixture sources:

| Fixtures        | Upstream revision                                                                         |
| --------------- | ----------------------------------------------------------------------------------------- |
| Raster corpus   | image-rs `6812e732343b0eaaeeef90ca751dd0e73430fb15`                                       |
| SVG reftests    | WPT `987a2d0c1a45f1a193f1f05d9508c4efc307c3bf`                                            |
| TinyVG examples | `e7c4c624fbe9276740fff2a9b6231ff86e01a89c` and `b8d8c7e88ed221f2ce1100f9e25b5c6e7e6dc78d` |

Raster benchmark throughput counts decoded canvas pixels across all frames.

## License

Copyright (c) 2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
