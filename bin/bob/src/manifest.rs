/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct Manifest {
    pub workspace: Workspace,
    pub package: Package,
    #[serde(rename = "lib")]
    pub library: Option<Library>,
    pub build: Build,
    pub dependencies: HashMap<String, Dependency>,
}

// MARK: Workspace
#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct Workspace {
    pub members: Vec<String>,
}

// MARK: Package
#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct Package {
    pub name: String,
    pub id: Option<String>,
    pub version: String,
    pub metadata: PackageMetadata,
}

#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct PackageMetadata {
    pub bundle: Option<BundleMetadata>,
    pub jar: Option<JarMetadata>,
    pub android: Option<AndroidMetadata>,
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub(crate) struct BundleMetadata {
    pub lipo: bool,
    pub resources_dir: String,
    pub iconset: Option<String>,
    pub copyright: Option<String>,
}

impl Default for BundleMetadata {
    fn default() -> Self {
        Self {
            lipo: false,
            resources_dir: "res".to_string(),
            iconset: None,
            copyright: None,
        }
    }
}

#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct JarMetadata {
    pub main_class: Option<String>,
    pub proguard_keep: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub(crate) struct AndroidMetadata {
    pub min_sdk_version: u32,
    pub target_sdk_version: u32,
    pub main_activity: String,
    pub keystore_file: String,
    pub key_alias: String,
    pub keystore_password: String,
    pub key_password: String,
    pub resources_dir: String,
    pub assets_dir: String,
    pub proguard_keep: Vec<String>,
}

impl Default for AndroidMetadata {
    fn default() -> Self {
        Self {
            min_sdk_version: 21,
            target_sdk_version: 36,
            main_activity: ".MainActivity".to_string(),
            keystore_file: "keystore.jks".to_string(),
            key_alias: "android".to_string(),
            keystore_password: "android".to_string(),
            key_password: "android".to_string(),
            resources_dir: "res".to_string(),
            assets_dir: "assets".to_string(),
            proguard_keep: Vec::new(),
        }
    }
}

// MARK: Library
#[derive(Default, Copy, Clone, Deserialize)]
pub(crate) enum LibraryType {
    #[default]
    #[serde(rename = "staticlib")]
    Static,
    #[serde(rename = "dylib")]
    Dynamic,
}

#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct Library {
    pub r#type: LibraryType,
}

// MARK: Build
#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct Build {
    // Platform specific
    pub macos: Option<Box<Build>>,
    pub linux: Option<Box<Build>>,
    pub windows: Option<Box<Build>>,

    pub asflags: String,
    pub cflags: String,
    pub ldflags: String,
    pub target: Option<String>,
    pub arch: Option<String>,
    pub entry: Option<String>,
    pub javac_flags: String,
    pub kotlinc_flags: String,
    pub classpath: Vec<String>,
}

impl Build {
    pub(crate) fn merge(&mut self, other_build: Build) {
        if !other_build.asflags.is_empty() {
            self.asflags = other_build.asflags;
        }
        if !other_build.cflags.is_empty() {
            self.cflags = other_build.cflags;
        }
        if !other_build.ldflags.is_empty() {
            self.ldflags = other_build.ldflags;
        }
        if other_build.target.is_some() {
            self.target = other_build.target;
        }
        if other_build.arch.is_some() {
            self.arch = other_build.arch;
        }
        if other_build.entry.is_some() {
            self.entry = other_build.entry;
        }
        if !other_build.javac_flags.is_empty() {
            self.javac_flags = other_build.javac_flags;
        }
        if !other_build.kotlinc_flags.is_empty() {
            self.kotlinc_flags = other_build.kotlinc_flags;
        }
        if !other_build.classpath.is_empty() {
            self.classpath = other_build.classpath;
        }
    }
}

// MARK: Dependencies
#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum Dependency {
    Path {
        path: String,
    },
    Library {
        library: String,
    },
    PkgConfig {
        #[serde(rename = "pkg-config")]
        pkg_config: String,
    },
    Framework {
        framework: String,
    },
    Jar {
        jar: JarDependency,
    },
    Maven {
        maven: String,
    },
}

#[derive(Clone, Deserialize)]
pub(crate) struct JarDependency {
    pub package: String,
    pub package_override: Option<String>,
    pub version: String,
    pub path: Option<String>,
    pub url: Option<String>,
}
