# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

_Placeholder_

## [0.1.4] - 2026-10-16

### Added

- Add a `simple-useragent` CLI for inspecting one or more user-agent strings.
- Add CLI integration tests and benchmarks for parser creation and parsing.
- Document command-line usage and package installation.

### Changed

- Raise the minimum supported Rust version to 1.85.

### Fixed

- Restore package metadata, include the license file, and specify a registry version for the benchmark dependency so the crate can be packaged for publication.
- Update the homepage for the crate's current location.

## [0.1.3] - 2026-08-25

### Changed

- Update the `regex` dependency baseline to 1.13.
- Update the bundled uap-core user-agent and operating-system rules.
- Clean up build-time rule generation without changing parser behavior.

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

[Unreleased]: https://github.com/bplaat/bron/compare/simple-useragent%2Fv0.1.4...HEAD
[0.1.4]: https://github.com/bplaat/bron/releases/tag/simple-useragent%2Fv0.1.4
[0.1.3]: https://github.com/bplaat/bron/releases/tag/simple-useragent%2Fv0.1.3
[0.1.2]: https://github.com/bplaat/bron/releases/tag/simple-useragent%2Fv0.1.2
[0.1.1]: https://github.com/bplaat/bron/releases/tag/simple-useragent%2Fv0.1.1
[0.1.0]: https://github.com/bplaat/bron/releases/tag/simple-useragent%2Fv0.1.0
