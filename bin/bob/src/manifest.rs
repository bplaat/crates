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
}

#[derive(Deserialize)]
pub(crate) struct PackageMetadata {
    pub bundle: Option<BundleMetadata>,
    pub jar: Option<JarMetadata>,
}

#[derive(Deserialize)]
pub(crate) struct BundleMetadata {
    pub iconset: Option<String>,
    pub copyright: String,
}

#[derive(Deserialize)]
pub(crate) struct JarMetadata {
    pub main_class: Option<String>,
}
