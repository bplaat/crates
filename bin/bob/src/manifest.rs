/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Manifest {
    pub package: Package,
    pub build: Option<Build>,
}

#[derive(Deserialize)]
pub(crate) struct Package {
    pub name: String,
    pub identifier: Option<String>,
    pub version: String,
    pub metadata: Option<PackageMetadata>,
}

#[derive(Deserialize)]
pub(crate) struct Build {
    pub cflags: Option<String>,
    pub ldflags: Option<String>,
    pub javac_flags: Option<String>,
    pub javac_classpath: Option<String>,
}

#[derive(Deserialize)]
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
