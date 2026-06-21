/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
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
    pub lipo: Option<bool>,
    pub resources_dir: Option<String>,
    pub iconset: Option<String>,
    pub icns: Option<String>,
    pub icon: Option<String>,
    pub info_plist: Option<String>,
    pub entitlements: Option<String>,
    pub hardened_runtime: Option<bool>,
    pub plugins: Option<Vec<PluginMetadata>>,
}

/// Metadata for a single app extension plugin bundled inside `Contents/PlugIns/`.
#[derive(Clone, Deserialize)]
pub(crate) struct PluginMetadata {
    /// Display name; also used as the `.appex` bundle name and the binary name inside the bundle.
    pub name: String,
    /// Bundle identifier for the plugin (e.g. `nl.bplaat.BImg.QLPreview`).
    pub identifier: String,
    /// Cargo binary name to build (e.g. `bimg-ql-preview`).
    pub binary: String,
    /// Path to the plugin-specific `Info.plist`, relative to the project root.
    pub info_plist: String,
    /// Optional path to the plugin entitlements file, relative to the project root.
    pub entitlements: Option<String>,
}
