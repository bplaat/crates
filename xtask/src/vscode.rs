/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs;

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::ser::PrettyFormatter;
use serde_json::{Serializer, Value};

use crate::Xtask;
use crate::metadata::platform_exclude_paths;

impl Xtask {
    pub(crate) fn configure_vscode(&self) -> Result<()> {
        let metadata = self.cargo_metadata()?;
        let excludes = platform_exclude_paths(&metadata, self.os, &self.root)?;
        let settings_path = self.root.join(".vscode/settings.json");
        let contents = fs::read_to_string(&settings_path)
            .with_context(|| format!("failed to read {}", settings_path.display()))?;
        let mut settings: Value = serde_json::from_str(&contents)
            .with_context(|| format!("failed to parse {}", settings_path.display()))?;
        set_rust_analyzer_excludes(&mut settings, excludes)?;

        let mut output = Vec::new();
        let formatter = PrettyFormatter::with_indent(b"    ");
        let mut serializer = Serializer::with_formatter(&mut output, formatter);
        settings.serialize(&mut serializer)?;
        output.push(b'\n');

        if output != contents.as_bytes() {
            fs::write(&settings_path, output)
                .with_context(|| format!("failed to write {}", settings_path.display()))?;
            println!("Configured VS Code for {}", self.os.name());
        }
        Ok(())
    }
}

fn set_rust_analyzer_excludes(
    settings: &mut Value,
    excludes: impl IntoIterator<Item = String>,
) -> Result<()> {
    let settings = settings
        .as_object_mut()
        .context("VS Code settings must be a JSON object")?;
    settings.insert(
        "rust-analyzer.files.exclude".to_owned(),
        Value::Array(excludes.into_iter().map(Value::String).collect()),
    );
    Ok(())
}

// MARK: Tests
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn rust_analyzer_excludes_are_added_without_changing_other_settings() -> Result<()> {
        let mut settings = json!({ "editor.formatOnSave": true });
        set_rust_analyzer_excludes(
            &mut settings,
            ["bin/macos-app".to_owned(), "bin/windows-app".to_owned()],
        )?;
        assert_eq!(
            settings,
            json!({
                "editor.formatOnSave": true,
                "rust-analyzer.files.exclude": ["bin/macos-app", "bin/windows-app"]
            })
        );
        Ok(())
    }

    #[test]
    fn rust_analyzer_excludes_replace_stale_platform_paths() -> Result<()> {
        let mut settings = json!({ "rust-analyzer.files.exclude": ["bin/old"] });
        set_rust_analyzer_excludes(&mut settings, ["bin/current".to_owned()])?;
        assert_eq!(
            settings["rust-analyzer.files.exclude"],
            json!(["bin/current"])
        );
        Ok(())
    }
}
