/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [winres](https://crates.io/crates/winres) crate

#![cfg(windows)]
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path;

/// Windows resource
pub struct WindowsResource {
    props: HashMap<String, String>,
    icon: Option<String>,
    manifest: Option<String>,
}

impl Default for WindowsResource {
    fn default() -> Self {
        let mut props = HashMap::new();
        props.insert(
            "FileVersion".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
        props.insert(
            "ProductVersion".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
        props.insert(
            "ProductName".to_string(),
            env!("CARGO_PKG_NAME").to_string(),
        );
        props.insert(
            "FileDescription".to_string(),
            env!("CARGO_PKG_DESCRIPTION").to_string(),
        );
        Self {
            props,
            icon: None,
            manifest: None,
        }
    }
}

impl WindowsResource {
    /// Create a new Windows resource
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a value for the resource
    pub fn set(&mut self, name: &str, value: &str) -> &mut Self {
        self.props.insert(name.to_string(), value.to_string());
        self
    }

    /// Set the icon for the resource
    pub fn set_icon(&mut self, path: &str) -> &mut Self {
        self.icon = Some(path.to_string());
        self
    }

    /// Set the manifest for the resource
    pub fn set_manifest(&mut self, manifest: &str) -> &mut Self {
        self.manifest = Some(manifest.to_string());
        self
    }

    /// Compile the resource
    pub fn compile(&self) -> std::io::Result<()> {}
}
