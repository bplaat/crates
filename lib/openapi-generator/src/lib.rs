/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! OpenAPI Generator

use std::str::FromStr;

use serde_yaml::Value;

mod generators;
pub mod openapi;
mod utils;

/// Generate schemas for usage in build.rs
pub fn generate_schemas(spec_path: &str, output_path: &str) {
    let spec_path = format!(
        "{}/{}",
        std::env::var("CARGO_MANIFEST_DIR").expect("Should exist"),
        spec_path
    );
    let output_file = format!(
        "{}/{}",
        std::env::var("OUT_DIR").expect("Should exists"),
        output_path
    );
    generate(&spec_path, Generator::Rust, &output_file);
    println!("cargo::rerun-if-changed=build.rs");
}

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

/// Generate schemas
pub fn generate(spec_path: &str, generator: Generator, output_path: &str) {
    // Read spec file
    let text = std::fs::read_to_string(spec_path).expect("Failed to read spec file");
    let yaml = serde_yaml::from_str::<Value>(&text).expect("Failed to parse YAML");

    // Resolve $refs
    let resolved_yaml = resolve_refs(yaml.clone(), &yaml);
    let spec = serde_yaml::from_value::<openapi::OpenApi>(resolved_yaml)
        .expect("Failed to deserialize YAML");

    // Run generator
    match generator {
        Generator::Rust => generators::rust::generator(spec, output_path),
        Generator::TypeScript => generators::typescript::generator(spec, output_path),
    }
}

// Resolve $ref in serde_yaml::Value
fn resolve_refs(value: Value, root: &Value) -> Value {
    match value {
        Value::Mapping(mut map) => {
            if let Some(Value::String(ref_path)) = map.get(Value::String("$ref".to_string())) {
                let parts: Vec<&str> = ref_path.split('/').collect();
                if parts[0] == "#" {
                    let mut current = root;
                    for part in &parts[1..] {
                        if let Value::Mapping(ref_map) = current {
                            current = ref_map
                                .get(Value::String(part.to_string()))
                                .expect("Failed to resolve reference");
                        }
                    }
                    return resolve_refs(current.clone(), root);
                }
            }
            for (_, v) in map.iter_mut() {
                *v = resolve_refs(v.clone(), root);
            }
            Value::Mapping(map)
        }
        Value::Sequence(seq) => {
            Value::Sequence(seq.into_iter().map(|v| resolve_refs(v, root)).collect())
        }
        _ => value,
    }
}
