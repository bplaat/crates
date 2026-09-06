# Image

A small, safe Rust decoder for JPEG, PNG/APNG, GIF, BMP, and QOI images. It uses
one complete-file API and returns straight-alpha RGBA8 pixels.

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

The decoder supports common 8-bit JPEG, PNG/APNG, GIF, BMP, and QOI variants,
including progressive JPEG, Adam7 PNG, GIF interlacing, BMP bitfields/RLE, and
animation disposal. It does not support 16-bit PNG/APNG, HDR output, encoding,
incremental decoding, or ICC color management. Decoding has a cumulative 512 MiB
allocation limit.

All formats are enabled by default. Disable default features and select from
`qoi`, `jpeg`, `png`, `gif`, and `bmp` to build only the required decoders. The
`png` feature is the only feature that enables the optional `miniz_oxide`
dependency.

Run `cargo run -p image --bin generate-fixtures` to regenerate fixtures with
ImageMagick and libjpeg-turbo. The checked-in fixtures let normal builds and the
test suite run without those tools.

## License

Copyright (c) 2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
