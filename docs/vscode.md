# VSCode configuration

```json
{
    "recommendations": [
        "EditorConfig.EditorConfig",
        "rust-lang.rust-analyzer",
        "tamasfe.even-better-toml",
        "esbenp.prettier-vscode"
    ],
    "search.exclude": {
        "**/target/**": true,
        "**/sqlite3/**": true,
        "**/node_modules/**": true,
        "**/dist/**": true
    },
    // Rust extensions settings
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer",
        "editor.formatOnSave": true
    },
    "rust-analyzer.rustfmt.extraArgs": ["+nightly"],
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.check.extraArgs": ["--all-features"],
    // Prettier settings
    "[css]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode"
    },
    "[javascript]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode"
    },
    "[javascriptreact]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode"
    },
    "[json]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode"
    }
}
```
