/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! OpenAPI Generator

use serde_yaml::Value;

mod generators;
mod openapi;

/// Generate crate from an OpenAPI file for usage in build.rs
pub fn generate_schemas(path: &str) {
    let input_file = format!(
        "{}/{}",
        std::env::var("CARGO_MANIFEST_DIR").expect("Should exist"),
        path
    );
    let output_dir = std::env::var("OUT_DIR").expect("Should exists");
    generate_internal(&input_file, Generator::Rust, &output_dir);
    println!("cargo::rerun-if-changed=build.rs");
}

enum Generator {
    Rust,
}

fn generate_internal(input: &str, generator: Generator, output: &str) {
    // Read input file
    let text = std::fs::read_to_string(input).expect("Failed to read input file");
    let yaml = serde_yaml::from_str::<Value>(&text).expect("Failed to parse YAML");

    // Resolve $refs
    let resolved_yaml = resolve_refs(yaml.clone(), &yaml);
    let spec = serde_yaml::from_value::<openapi::OpenApi>(resolved_yaml)
        .expect("Failed to deserialize YAML");

    // Run generator
    match generator {
        Generator::Rust => generators::rust::generator(spec, output),
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
