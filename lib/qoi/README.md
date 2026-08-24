# QOI

A small decoder for the [Quite OK Image](https://qoiformat.org/) format.

## Example

```rs
let data = std::fs::read("image.qoi").expect("Can't read image");
let image = qoi::decode(&data).expect("Can't decode image");

println!("{}x{}", image.width(), image.height());
println!("RGBA bytes: {}", image.pixels().len());
```

## Features

- Decodes complete QOI files to RGBA pixels
- Exposes image dimensions, channel count, and color space
- Has no external dependencies

## License

Copyright © 2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
