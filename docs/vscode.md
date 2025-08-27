# VSCode configuration

```json
{
    "recommendations": [
        "EditorConfig.EditorConfig",
        "rust-lang.rust-analyzer",
        "tamasfe.even-better-toml",
        "esbenp.prettier-vscode",
        "xaver.clang-format"
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

    // Web extensions settings
    "[html]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode",
        "editor.formatOnSave": true
    },
    "[css]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode",
        "editor.formatOnSave": true
    },
    "[javascript]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode",
        "editor.formatOnSave": true
    },
    "[javascriptreact]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode",
        "editor.formatOnSave": true
    },
    "[typescript]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode",
        "editor.formatOnSave": true
    },
    "[typescripttreact]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode",
        "editor.formatOnSave": true
    },
    "[json]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode",
        "editor.formatOnSave": true
    },
    "[yaml]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode",
        "editor.formatOnSave": true
    },

    // Clang-format extension
    "[c]": {
        "editor.defaultFormatter": "xaver.clang-format",
        "editor.formatOnSave": true
    },
    "[cpp]": {
        "editor.defaultFormatter": "xaver.clang-format",
        "editor.formatOnSave": true
    },
    "[objective-c]": {
        "editor.defaultFormatter": "xaver.clang-format",
        "editor.formatOnSave": true
    },
    "[objective-cpp]": {
        "editor.defaultFormatter": "xaver.clang-format",
        "editor.formatOnSave": true
    },
    "[java]": {
        "editor.defaultFormatter": "xaver.clang-format",
        "editor.formatOnSave": true
    }
}
```
