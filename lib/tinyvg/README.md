# TinyVG

A small parser for binary and textual [TinyVG](https://tinyvg.tech/) vector images.

## Example

```rs
let data = std::fs::read("image.tvg").expect("Can't read image");
let document = tinyvg::parse_auto(&data).expect("Can't parse image");

println!("{}x{}", document.size.width, document.size.height);
println!("Commands: {}", document.commands.len());
```

## Features

- Parses binary `.tvg` and textual `.tvgt` documents
- Produces normalized absolute drawing commands
- Supports fills, strokes, paths, solid colors, and gradients
- Has no external dependencies

## License

Copyright © 2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
