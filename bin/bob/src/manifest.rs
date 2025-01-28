/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(dead_code)] // FIXME: Remove this line

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Manifest {
    pub package: Package,
}

#[derive(Deserialize)]
pub(crate) struct Package {
    pub name: String,
    pub identifier: Option<String>,
    pub version: String,
    pub metadata: Option<PackageMetadata>,
}

#[derive(Deserialize)]
pub(crate) struct PackageMetadata {
    pub jar: Option<JarMetadata>,
}

#[derive(Deserialize)]
pub(crate) struct JarMetadata {
    pub main_class: String,
}
