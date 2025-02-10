/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! OpenAPI Generator

#![forbid(unsafe_code)]

use std::path::Path;
use std::str::FromStr;

mod generators;
pub mod openapi;
mod utils;

/// Generator type
pub enum Generator {
    /// Rust generator
    Rust,
    /// TypeScript generator
    TypeScript,
}

impl FromStr for Generator {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rust" => Ok(Generator::Rust),
            "typescript" => Ok(Generator::TypeScript),
            _ => Err("Invalid generator".to_string()),
        }
    }
}

/// Generate schemas for build.rs
pub fn generate_schemas_build(spec_path: impl AsRef<Path>, output_path: impl AsRef<Path>) {
    generate_schemas(spec_path.as_ref(), Generator::Rust, output_path.as_ref());
    println!("cargo::rerun-if-changed=build.rs");
}

/// Generate schemas
pub fn generate_schemas(
    spec_path: impl AsRef<Path>,
    generator: Generator,
    output_path: impl AsRef<Path>,
) {
    fn inner(spec_path: &Path, generator: Generator, output_path: &Path) {
        // Read spec file
        let text = std::fs::read_to_string(spec_path).expect("Failed to read spec file");
        let spec =
            serde_yaml::from_str::<openapi::OpenApi>(&text).expect("Failed to deserialize yaml");

        // Run generator
        match generator {
            Generator::Rust => {
                generators::rust::generate_schemas(spec.components.schemas, output_path)
            }
            Generator::TypeScript => {
                generators::typescript::generate_schemas(spec.components.schemas, output_path)
            }
        }
    }
    inner(spec_path.as_ref(), generator, output_path.as_ref());
}
