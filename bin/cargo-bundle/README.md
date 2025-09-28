# cargo-bundle

A simple Cargo plugin the builds macOS app bundles

## Usage

Add app bundle metadata to your `Cargo.toml`:

```toml
[package.metadata.bundle]
name = "ExampleApp"
identifier = "com.example.App"
copyright = "Copyright © 2025 Anonymous"
iconset = "path/to/icon.iconset" # optional
```

Then run `cargo bundle` with the path argument, when using a Cargo workspace run the command in the root of the workspace:

```sh
cargo bundle --path path/to/your/crate
```

This will build your crate for x86_64 and arm64 link it together as a Universal binary. Create a `.icns` file from you optional iconset. And create a macOS app bundle in the `target/bundle` directory.

## License

Copyright © 2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
