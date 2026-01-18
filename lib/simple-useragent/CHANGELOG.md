# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

_Placeholder_

## [0.1.2] - 2025-02-11

### Fixed

- Use `static` instead of `const` for regex codegen to avoid inlining.

## [0.1.1] - 2025-02-04

### Changed

- Remove `serde` and `postcard` dependencies and use Rust codegen instead.
- Updated `regexes.yaml` from [uap-core](https://github.com/ua-parser/uap-core/blob/master/regexes.yaml)

### Added

- Make [README.md](README.md) more complete.

### Fixed

- Fix bug with dollar template variables that where not expanded correctly.

## [0.1.0] - 2025-02-04

_Initial release_

[Unreleased]: https://github.com/bplaat/crates/compare/simple-useragent%2Fv0.1.2...HEAD
[0.1.2]: https://github.com/bplaat/crates/releases/tag/simple-useragent%2Fv0.1.2
[0.1.1]: https://github.com/bplaat/crates/releases/tag/simple-useragent%2Fv0.1.1
[0.1.0]: https://github.com/bplaat/crates/releases/tag/simple-useragent%2Fv0.1.0
