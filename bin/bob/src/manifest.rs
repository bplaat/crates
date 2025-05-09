/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Manifest {
    pub package: Package,
    #[serde(default)]
    pub build: Build,
}

#[derive(Deserialize)]
pub(crate) struct Package {
    pub name: String,
    pub identifier: Option<String>,
    pub version: String,
    #[serde(default)]
    pub metadata: PackageMetadata,
}

#[derive(Deserialize)]
pub(crate) struct Build {
    pub cflags: String,
    pub ldflags: String,
    pub target: Option<String>,
    pub targets: Vec<String>,
    pub lipo: bool,
    pub javac_flags: String,
    pub classpath: Option<String>,
}
impl Default for Build {
    fn default() -> Self {
        Self {
            cflags: String::new(),
            ldflags: String::new(),
            target: None,
            targets: Vec::new(),
            lipo: false,
            javac_flags: String::new(),
            classpath: None,
        }
    }
}

#[derive(Default, Deserialize)]
pub(crate) struct PackageMetadata {
    pub bundle: Option<BundleMetadata>,
    pub jar: Option<JarMetadata>,
    pub android: Option<AndroidMetadata>,
}

#[derive(Deserialize)]
pub(crate) struct BundleMetadata {
    pub iconset: Option<String>,
    pub copyright: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct JarMetadata {
    pub main_class: Option<String>,
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
        }
    }
}
