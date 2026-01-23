/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple note-taking app

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use copy_dir::copy_dir;

fn main() {
    // Generate openapi file
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    openapi_generator::generate_schemas_build(
        "openapi.yaml",
        out_dir.join("api.rs"),
        openapi_generator::Generator::Rust,
    );
    openapi_generator::generate_schemas_build(
        "openapi.yaml",
        "web/src-gen/api.ts",
        openapi_generator::Generator::TypeScript,
    );

    // Build web frontend
    const NPM: &str = if cfg!(windows) { "npm.cmd" } else { "npm" };

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
                let file_name = path.file_name().expect("Should have a file name");
                if file_name == "dist" || file_name == "node_modules" {
                    continue;
                }
                print_rerun(&path);
            } else {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    print_rerun(Path::new("web"));

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

    // Copy built assets to OUT_DIR/web);
    copy_dir("web/dist", out_dir.join("web")).expect("Failed to copy web/dist files to $OUT_DIR");
}
