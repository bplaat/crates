# cargo-bundle

A simple Cargo plugin that builds macOS app bundles.

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

This builds your crate for x86_64 and arm64, links both architectures into a universal binary,
creates an `.icns` file from the optional iconset, and creates a macOS app bundle in the
`target/bundle` directory.

## License

Copyright © 2025-2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
