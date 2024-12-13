# VSCode configuration

```json
{
    "recommendations": [
        "EditorConfig.EditorConfig",
        "rust-lang.rust-analyzer",
        "tamasfe.even-better-toml"
    ],
    "search.exclude": {
        "**/target/**": true,
        "**/sqlite3/**": true
    },
    // Rust extensions settings
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer",
        "editor.formatOnSave": true
    },
    "rust-analyzer.rustfmt.extraArgs": ["+nightly"],
    "rust-analyzer.check.command": "clippy"
}
```
