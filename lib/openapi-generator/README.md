# OpenAPI Generator

A small OpenAPI schema generator for Rust and TypeScript.

## Usage

Generate Rust types from an OpenAPI YAML file:

```sh
cargo run -p openapi-generator -- --input openapi.yaml --generator rust --output api.rs
```

Use `typescript` as the generator to create TypeScript definitions instead.

## Features

- Generates Rust structs and enums with Serde support
- Generates TypeScript interfaces and union types
- Supports objects, arrays, maps, references, enums, and primitive types
- Can run from the command line or a Rust build script

## License

Copyright © 2024-2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
