/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::Os;
use crate::utils::normalize_path;

pub(crate) struct InstallableApp {
    pub(crate) package: String,
    pub(crate) name: String,
}

pub(crate) fn packages(metadata: &Value) -> Result<&[Value]> {
    metadata
        .get("packages")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .context("cargo metadata has no packages")
}

pub(crate) fn platform_excludes(metadata: &Value, os: Os) -> BTreeSet<String> {
    packages(metadata)
        .unwrap_or_default()
        .iter()
        .filter(|package| !supports_os(package, os))
        .filter_map(|package| {
            package
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect()
}

pub(crate) fn supports_os(package: &Value, os: Os) -> bool {
    package
        .pointer("/metadata/platforms")
        .and_then(Value::as_array)
        .is_none_or(|platforms| {
            platforms
                .iter()
                .any(|platform| platform.as_str() == Some(os.name()))
        })
}

pub(crate) fn add_excludes(command: &mut Command, excludes: &BTreeSet<String>) {
    for package in excludes {
        command.args(["--exclude", package]);
    }
}

pub(crate) fn features_without_swap(
    metadata: &Value,
    package_name: &str,
    swap_feature: &str,
) -> Result<Vec<String>> {
    let package = packages(metadata)?
        .iter()
        .find(|package| package.get("name").and_then(Value::as_str) == Some(package_name))
        .with_context(|| format!("package not found in cargo metadata: {package_name}"))?;
    let features = package
        .get("features")
        .and_then(Value::as_object)
        .context("Cargo package has no features object")?;
    let remaining = features
        .keys()
        .filter(|feature| feature.as_str() != "default" && feature.as_str() != swap_feature)
        .cloned()
        .collect::<Vec<_>>();
    if remaining.is_empty() {
        Ok(Vec::new())
    } else {
        Ok(vec!["--features".to_owned(), remaining.join(",")])
    }
}

pub(crate) fn package_for_directory<'a>(
    metadata: &'a Value,
    directory: &Path,
    root: &Path,
) -> Result<&'a str> {
    let manifest = directory.join("Cargo.toml");
    packages(metadata)?
        .iter()
        .find(|package| {
            package
                .get("manifest_path")
                .and_then(Value::as_str)
                .is_some_and(|path| normalize_path(Path::new(path), root) == manifest)
        })
        .and_then(|package| package.get("name"))
        .and_then(Value::as_str)
        .with_context(|| format!("could not determine package for {}", directory.display()))
}

// MARK: Tests
#[cfg(test)]
mod tests {
    use std::env;

    use serde_json::json;

    use super::*;

    #[test]
    fn platform_support_defaults_to_every_os() {
        let package = json!({ "metadata": {} });
        assert!(supports_os(&package, Os::Linux));
        assert!(supports_os(&package, Os::Macos));
        assert!(supports_os(&package, Os::Windows));
    }

    #[test]
    fn platform_support_uses_cargo_metadata() {
        let package = json!({ "metadata": { "platforms": ["linux", "windows"] } });
        assert!(supports_os(&package, Os::Linux));
        assert!(!supports_os(&package, Os::Macos));
        assert!(supports_os(&package, Os::Windows));
    }

    #[test]
    fn backend_swap_feature_is_excluded() -> Result<()> {
        let metadata = json!({
            "packages": [{
                "name": "example",
                "features": {
                    "default": [],
                    "bundled": [],
                    "chrono": [],
                    "uuid": []
                }
            }]
        });
        assert_eq!(
            features_without_swap(&metadata, "example", "bundled")?,
            ["--features", "chrono,uuid"]
        );
        Ok(())
    }

    #[test]
    fn backend_swap_can_leave_no_features() -> Result<()> {
        let metadata = json!({
            "packages": [{
                "name": "example",
                "features": { "default": [], "vendored": [] }
            }]
        });
        assert!(features_without_swap(&metadata, "example", "vendored")?.is_empty());
        Ok(())
    }

    #[test]
    fn platform_excludes_lists_unsupported_packages() {
        let metadata = json!({
            "packages": [
                { "name": "everywhere", "metadata": {} },
                { "name": "unix-only", "metadata": { "platforms": ["linux", "macos"] } },
                { "name": "win-only", "metadata": { "platforms": ["windows"] } }
            ]
        });
        let excludes = platform_excludes(&metadata, Os::Windows);
        assert!(excludes.contains("unix-only"));
        assert!(!excludes.contains("everywhere"));
        assert!(!excludes.contains("win-only"));
    }

    #[test]
    fn packages_errors_when_absent() {
        assert!(packages(&json!({})).is_err());
    }

    #[test]
    fn add_excludes_appends_flags_in_order() {
        let mut command = Command::new("cargo");
        let excludes = ["alpha".to_owned(), "beta".to_owned()]
            .into_iter()
            .collect::<BTreeSet<_>>();
        add_excludes(&mut command, &excludes);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(args, ["--exclude", "alpha", "--exclude", "beta"]);
    }

    #[test]
    fn package_for_directory_matches_absolute_and_relative_manifests() -> Result<()> {
        let root = env::temp_dir().join("workspace");
        let metadata = json!({
            "packages": [
                { "name": "foo", "manifest_path": root.join("lib/foo/Cargo.toml").to_string_lossy() },
                { "name": "bar", "manifest_path": "lib/bar/Cargo.toml" }
            ]
        });
        assert_eq!(
            package_for_directory(&metadata, &root.join("lib/foo"), &root)?,
            "foo"
        );
        assert_eq!(
            package_for_directory(&metadata, &root.join("lib/bar"), &root)?,
            "bar"
        );
        assert!(package_for_directory(&metadata, &root.join("lib/missing"), &root).is_err());
        Ok(())
    }
}
