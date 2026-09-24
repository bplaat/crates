# Image

Decode JPEG, PNG/APNG, GIF, BMP, ICO, and QOI images into RGBA pixels. You can
also encode images as JPEG, PNG/APNG, GIF, BMP, or QOI.

## Getting started

```rs
let bytes = std::fs::read("picture.png").expect("read image");
let image = image::decode(&bytes).expect("decode image");
for frame in image.frames() {
    let bitmap = frame.bitmap();
    println!("{}x{}: {:?}", bitmap.width(), bitmap.height(), frame.delay());
}
```

To write an image, create a `Bitmap` from RGBA pixels and call `encode`.
Use `Image::encode` to save a decoded image in another format.

For GIF or APNG, give `encode_animation` a list of full-size frames, their
display times, and a loop count. GIF supports up to 256 colors per frame and
binary transparency. JPEG requires opaque pixels.

## Compression

Normal compression is the default. For PNG/APNG or JPEG, you can spend more
time looking for a smaller file. Set `EncodeOptions::style` to `MaxCompression`
and use `encode_with_options` or `encode_animation_with_options`. It keeps the
smallest result it tries.

| Format   | Normal compression                                                                                                                 | Max compression                                                                                   |
| -------- | ---------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| BMP      | Chooses 24-bit, 32-bit, packed palette, or RLE8 pixels with literal runs.                                                          | Same as normal.                                                                                   |
| GIF      | Uses an exact palette when possible, otherwise reduces colors, then applies LZW. Opaque animation frames store only changed areas. | Same as normal.                                                                                   |
| PNG/APNG | Chooses pixel layout and row filters; uses zlib level 6. APNG frames store only changed areas.                                     | Also tries palettes, a single transparent color, more filters, zlib levels, and full APNG frames. |
| JPEG     | Uses grayscale or 4:2:0 color with standard coding tables.                                                                         | Compares 4:2:0, 4:2:2, 4:4:0, and 4:4:4 color, and builds coding tables for the image.            |
| QOI      | Uses run, index, difference, and luma operations.                                                                                  | Same as normal.                                                                                   |

JPEG's color sampling can change how the image looks between compression styles.
BMP, PNG/APNG, and QOI preserve RGBA pixels.

## Reading

Animated images expose full-size frames, display times, and a loop count.
Decoding applies JPEG orientation, reduces 16-bit PNG samples to 8 bits, and
selects the largest image in an ICO file. HDR, streaming, and ICC color
management are not supported. Decoding has a cumulative 1 GiB allocation limit.

All formats are enabled by default. You can enable only the codecs you need
with the `qoi`, `jpeg`, `png`, `gif`, `bmp`, and `ico` Cargo features.

## Tests and benchmarks

Run `cargo test -p image` for tests or `cargo bench -p image --bench image` for
benchmarks. Tests use reference images from image-rs.

## License

Copyright (c) 2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
