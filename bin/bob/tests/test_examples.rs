/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Build examples test

use std::fs;
use std::process::Command;

#[test]
fn test_run_tests_examples() {
    let examples_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");
    let bob_bin = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/debug/bob");
    for entry in fs::read_dir(examples_dir).expect("Failed to read examples directory") {
        let entry = entry.expect("Failed to read directory entry");
        if entry.path().is_dir() {
            let project_name = entry.file_name().to_string_lossy().to_string();
            if project_name.starts_with("objc-") || project_name.starts_with("java-") {
                continue;
            }

            let output = Command::new(bob_bin)
                .arg("test")
                .current_dir(entry.path())
                .output()
                .expect("Failed to execute bob build command");
            if !output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!(
                    "Failed to test example {}:\nstdout: {}\nstderr: {}",
                    entry.path().display(),
                    stdout,
                    stderr
                );
            }
        }
    }
}
