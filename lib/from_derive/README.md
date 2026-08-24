# From Derive

Derive macros for generating bidirectional conversions between matching Rust structs and enums.

## Example

```rs
use from_derive::FromStruct;

struct ApiUser {
    name: String,
}

#[derive(FromStruct)]
#[from_struct(ApiUser)]
struct User {
    name: String,
}

let user = User::from(ApiUser {
    name: "Alice".to_string(),
});
```

## Features

- `FromStruct` converts matching fields using `Into`
- `FromEnum` converts identically named enum variants
- Generates conversions in both directions

## License

Copyright © 2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
