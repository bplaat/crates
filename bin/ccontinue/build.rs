/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("README.md")]

use std::fs;
use std::path::Path;

fn main() {
    // Generate test functions for each .cc file in tests/
    let tests_dir = Path::new("tests");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set by cargo");
    let dest = Path::new(&out_dir).join("generated_tests.rs");

    // On Windows there is no C compiler or leak checker available; skip all integration tests
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        fs::write(&dest, "").expect("write generated tests file");
        return;
    }

    let mut test_fns = String::new();
    let mut entries: Vec<_> = fs::read_dir(tests_dir)
        .expect("tests/ dir should exist")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "cc").unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in &entries {
        let path = entry.path();
        let stem = path
            .file_stem()
            .expect("path has file stem")
            .to_str()
            .expect("stem is valid UTF-8");
        // Sanitize name: digits and letters only (replace non-alnum with _)
        let fn_name: String = stem
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        let path_str = path.to_str().expect("path is valid UTF-8");
        test_fns.push_str(&format!(
            "#[test]\nfn test_{fn_name}() {{\n    run_test({path_str:?});\n}}\n\n"
        ));
        println!("cargo:rerun-if-changed={path_str}");
    }

    fs::write(&dest, test_fns).expect("write generated tests file");
    println!("cargo:rerun-if-changed=tests/");
}
