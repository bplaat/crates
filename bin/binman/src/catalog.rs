/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Catalog {
    pub(crate) version: u32,
    pub(crate) groups: Vec<CleanupGroup>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CleanupGroup {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) rules: Vec<CleanupDefinition>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CleanupDefinition {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) default_selected: bool,
    #[serde(default)]
    pub(crate) requires_administrator: bool,
    #[serde(default)]
    pub(crate) process_names: Vec<String>,
    #[serde(flatten)]
    pub(crate) kind: CleanupKind,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum CleanupKind {
    Paths { rules: Vec<PathRule> },
    RecycleBin,
    WindowsUpdate,
    DeliveryOptimization,
    DismComponentStore,
    DockerBuildCache,
    UvCache,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PathRule {
    pub(crate) root: PathRoot,
    pub(crate) path: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum PathRoot {
    UserProfile,
    LocalAppData,
    RoamingAppData,
    ProgramData,
    Windows,
    SystemDrive,
    ProgramFiles,
    ProgramFilesX86,
}

impl PathRoot {
    pub(crate) fn resolve(self) -> Option<PathBuf> {
        #[cfg(windows)]
        {
            use directories::windows;

            match self {
                Self::UserProfile => windows::profile_dir(),
                Self::LocalAppData => windows::local_app_data_dir(),
                Self::RoamingAppData => windows::roaming_app_data_dir(),
                Self::ProgramData => windows::program_data_dir(),
                Self::Windows => windows::windows_dir(),
                Self::SystemDrive => {
                    windows::windows_dir().and_then(|path| path.parent().map(Path::to_path_buf))
                }
                Self::ProgramFiles => windows::program_files_dir(),
                Self::ProgramFilesX86 => windows::program_files_x86_dir(),
            }
        }

        #[cfg(not(windows))]
        {
            let name = match self {
                Self::UserProfile => "USERPROFILE",
                Self::LocalAppData => "LOCALAPPDATA",
                Self::RoamingAppData => "APPDATA",
                Self::ProgramData => "ProgramData",
                Self::Windows => "WINDIR",
                Self::SystemDrive => "SystemDrive",
                Self::ProgramFiles => "ProgramFiles",
                Self::ProgramFilesX86 => "ProgramFiles(x86)",
            };
            let value = std::env::var_os(name)?;
            if matches!(self, Self::SystemDrive) {
                Some(PathBuf::from(format!("{}\\", value.to_string_lossy())))
            } else {
                Some(PathBuf::from(value))
            }
        }
    }
}

impl Catalog {
    pub(crate) fn load() -> Result<Self> {
        let catalog: Self = serde_json::from_str(include_str!("../rules.json"))?;
        catalog.validate()?;
        Ok(catalog)
    }

    fn validate(&self) -> Result<()> {
        if self.version != 1 {
            bail!("Unsupported cleanup catalog version {}", self.version);
        }
        let mut ids = HashSet::new();
        for group in &self.groups {
            if group.id.is_empty() || group.name.is_empty() {
                bail!("Cleanup groups must have an id and name");
            }
            for cleanup in &group.rules {
                if !ids.insert(&cleanup.id) {
                    bail!("Duplicate cleanup rule id: {}", cleanup.id);
                }
                if cleanup.id.is_empty() || cleanup.name.is_empty() {
                    bail!("Cleanup rules must have an id and name");
                }
                if let CleanupKind::Paths { rules } = &cleanup.kind {
                    if rules.is_empty() {
                        bail!("Path cleanup {} has no rules", cleanup.id);
                    }
                    for rule in rules {
                        validate_relative_pattern(&rule.path)
                            .map_err(|error| anyhow!("{}: {error}", cleanup.id))?;
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn rules(&self) -> impl Iterator<Item = &CleanupDefinition> {
        self.groups.iter().flat_map(|group| group.rules.iter())
    }
}

fn validate_relative_pattern(value: &str) -> Result<()> {
    if value.is_empty() || Path::new(value).is_absolute() || value.starts_with(['\\', '/']) {
        bail!("cleanup patterns must be non-empty relative paths");
    }
    if Path::new(value)
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    {
        bail!("cleanup patterns may not escape their root");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_is_valid() {
        let catalog = Catalog::load().expect("catalog should load");
        assert!(!catalog.groups.is_empty());
        assert!(catalog.rules().count() >= 63);

        let ids: HashSet<_> = catalog.rules().map(|rule| rule.id.as_str()).collect();
        for id in [
            "windows-user-temp",
            "windows-update-downloads",
            "edge-cache",
            "chrome-cache",
            "firefox-cache",
            "brave-cache",
            "teams-cache",
            "discord-cache",
            "slack-cache",
            "steam-cache",
            "vscode-cache",
            "gpu-shader-caches",
            "visual-studio-cache",
            "rustup-cache",
            "go-build-cache",
            "maven-cache",
            "jetbrains-cache",
            "android-cache",
            "docker-build-cache",
            "uv-cache",
            "steam-shader-cache",
            "unreal-engine-cache",
        ] {
            assert!(ids.contains(id), "missing release cleanup rule {id}");
        }
    }

    #[test]
    fn unsafe_relative_patterns_are_rejected() {
        assert!(validate_relative_pattern("../outside").is_err());
        assert!(validate_relative_pattern("C:\\Windows").is_err());
        assert!(validate_relative_pattern("\\\\server\\share").is_err());
        assert!(validate_relative_pattern("safe/cache").is_ok());
    }
}
