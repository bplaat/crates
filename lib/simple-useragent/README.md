# Simple UserAgent parser

A simple user agent parser library based on the uap-core regexes

## Example

A simple example that parses a user agent string and reads the parsed fields:

```rs
fn main() {
    // Create a user agent parser
    let parser = simple_useragent::UserAgentParser::new();

    // Parse a user agent string
    let ua = parser.parse(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:134.0) Gecko/20100101 Firefox/134.0",
    );

    // Print the parsed user agent fields
    println!("Client family: {}", ua.client.family); // -> "Firefox"
    println!("Client version: {:?}", ua.client.version); // -> Some("134.0")
    println!("OS family: {}", ua.os.family); // -> "Mac OS X"
    println!("OS version: {:?}", ua.os.version); // -> Some("10.15")
}
```

## Features

- **serde**: Enable serialization and deserialization derives of the structs with [serde](https://serde.rs/).

## Documentation

See the [documentation](https://docs.rs/simple-useragent) for more information.

## License

Copyright © 2024-2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
