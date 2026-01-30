/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Manifest {
    pub package: Package,
}

#[derive(Deserialize)]
pub(crate) struct Package {
    pub name: String,
    pub version: String,
    pub metadata: PackageMetadata,
}

#[derive(Deserialize)]
pub(crate) struct PackageMetadata {
    pub bundle: BundleMetadata,
}

#[derive(Clone, Deserialize)]
pub(crate) struct BundleMetadata {
    pub name: String,
    pub identifier: String,
    pub copyright: Option<String>,
    pub minimal_os_version: Option<String>,
    pub resources_dir: Option<String>,
    pub iconset: Option<String>,
    pub icns: Option<String>,
    pub icon: Option<String>,
}
