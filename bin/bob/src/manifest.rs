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
    pub package: Package,
    pub build: Build,
    pub dependencies: HashMap<String, Dependency>,
}

#[derive(Default, Copy, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PackageType {
    #[default]
    #[serde(alias = "bin")]
    Binary,
    #[serde(alias = "lib")]
    Library,
    #[serde(skip)]
    ExternalJar,
}

#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct Package {
    pub name: String,
    pub id: Option<String>,
    pub version: String,
    pub r#type: PackageType,
    pub metadata: PackageMetadata,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Dependency {
    pub path: Option<String>,
    pub library: Option<String>,
    pub pkg_config: Option<String>,
    pub framework: Option<String>,
    pub jar: Option<JarDependency>,
    pub maven: Option<String>,
}

#[derive(Clone, Deserialize)]
pub(crate) struct JarDependency {
    pub package: String,
    pub package_override: Option<String>,
    pub version: String,
    pub path: Option<String>,
    pub url: Option<String>,
}

#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct Build {
    pub cflags: String,
    pub ldflags: String,
    pub javac_flags: String,
    pub kotlinc_flags: String,
    pub classpath: Vec<String>,
}

#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct PackageMetadata {
    pub bundle: Option<BundleMetadata>,
    pub jar: Option<JarMetadata>,
    pub android: Option<AndroidMetadata>,
}

// MARK: Bundle
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

// MARK: Jar
#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct JarMetadata {
    pub main_class: Option<String>,
    pub proguard_keep: Vec<String>,
}

// MARK: Android
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
