/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple note-taking app

use std::path::Path;
use std::process::Command;

use copy_dir::copy_dir;

fn main() {
    // Generate openapi file
    let out_dir = std::env::var("OUT_DIR").expect("Should be some");
    openapi_generator::generate_schemas_build(
        "openapi.yaml",
        format!("{out_dir}/api.rs"),
        openapi_generator::Generator::Rust,
    );
    openapi_generator::generate_schemas_build(
        "openapi.yaml",
        "web/src/api.ts",
        openapi_generator::Generator::TypeScript,
    );

    // Build web frontend
    #[cfg(windows)]
    const NPM: &str = "npm.cmd";
    #[cfg(not(windows))]
    const NPM: &str = "npm";

    // Install npm packages if needed
    if !Path::new("web/node_modules").exists() {
        Command::new(NPM)
            .arg("ci")
            .arg("--prefer-offline")
            .current_dir("web")
            .output()
            .expect("Failed to run npm install");
    }

    // Invalidate build when web assets change
    fn print_rerun(dir: &Path) {
        for entry in std::fs::read_dir(dir).expect("Failed to read dir") {
            let path = entry.expect("Failed to read entry").path();
            if path.is_dir() {
                print_rerun(&path);
            } else {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    println!("cargo:rerun-if-changed=web/index.html");
    print_rerun(Path::new("web/src"));

    // Build frontend
    Command::new(NPM)
        .arg("run")
        .arg(if cfg!(debug_assertions) {
            "build-debug"
        } else {
            "build-release"
        })
        .current_dir("web")
        .output()
        .expect("Failed to run npm run build");

    // Copy built assets to OUT_DIR/web
    let out_dir = std::env::var("OUT_DIR").expect("Should be some");
    copy_dir("web/dist", Path::new(&out_dir).join("web"))
        .expect("Failed to copy web/dist files to $OUT_DIR");
}
