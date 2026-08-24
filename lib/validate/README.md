# Validate

A simple struct validation library for Rust.

## Example

```rs
use validate::Validate;

#[derive(Validate)]
struct User {
    #[validate(ascii, length(min = 3, max = 25))]
    name: String,
    #[validate(range(min = 18))]
    age: u8,
}

let user = User {
    name: "Alice".to_string(),
    age: 30,
};
assert!(user.validate().is_ok());
```

## Features

- ASCII, length, and numeric range validation rules
- Optional email and URL validation rules
- Custom validators with optional context
- Multiple error messages per field
- Optional Serde support for validation reports

## License

Copyright © 2024-2025 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
