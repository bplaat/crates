/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Command-line behavior for the OpenAPI generator.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

fn temp_directory() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "openapi-generator-test-{}-{unique}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&path).expect("create test directory");
    path
}

const SPEC: &str = "openapi: 3.0.0\n\
info:\n  title: Test\n  version: 1.0.0\n\
paths: {}\n\
components:\n  schemas:\n    Person:\n      type: object\n      required: [name]\n      properties:\n        name:\n          type: string\n";

#[test]
fn generates_rust_with_default_arguments() {
    let root = temp_directory();
    fs::write(root.join("openapi.yaml"), SPEC).expect("write spec");
    let output = Command::new(env!("CARGO_BIN_EXE_openapi-generator"))
        .current_dir(&root)
        .output()
        .expect("run generator");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = fs::read_to_string(root.join("api.rs")).expect("read generated Rust");
    assert!(generated.contains("pub(crate) struct Person"));
    assert!(generated.contains("pub name: String"));
    fs::remove_dir_all(root).expect("remove test directory");
}

#[test]
fn generates_typescript_with_explicit_arguments() {
    let root = temp_directory();
    let spec = root.join("schema.yaml");
    let output_path = root.join("generated/api.ts");
    fs::write(&spec, SPEC).expect("write spec");
    let output = Command::new(env!("CARGO_BIN_EXE_openapi-generator"))
        .args(["--input", spec.to_str().expect("UTF-8 path")])
        .args(["--generator", "typescript"])
        .args(["--output", output_path.to_str().expect("UTF-8 path")])
        .output()
        .expect("run generator");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = fs::read_to_string(output_path).expect("read generated TypeScript");
    assert!(generated.contains("export interface Person"));
    assert!(generated.contains("name: string"));
    fs::remove_dir_all(root).expect("remove test directory");
}

#[test]
fn rejects_invalid_arguments() {
    let binary = env!("CARGO_BIN_EXE_openapi-generator");
    for (args, message) in [
        (&["--unknown"][..], "Unknown argument"),
        (&["--input"][..], "Invalid argument"),
        (&["--generator", "invalid"][..], "Invalid generator"),
    ] {
        let output = Command::new(binary)
            .args(args)
            .output()
            .expect("run generator");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(message));
    }
}
