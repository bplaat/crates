/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal self-hosted deployment service

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use copy_dir::copy_dir;

fn main() {
    // Generate API types from openapi.yaml
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    openapi_generator::generate_schemas_build(
        "openapi.yaml",
        out_dir.join("api.rs"),
        openapi_generator::Generator::Rust,
    );
    openapi_generator::generate_schemas_build(
        "openapi.yaml",
        "web/src/src-gen/api.ts",
        openapi_generator::Generator::TypeScript,
    );

    // Build web frontend
    const NPM: &str = if cfg!(windows) { "npm.cmd" } else { "npm" };

    if !Path::new("web/node_modules").exists() {
        Command::new(NPM)
            .arg("install")
            .current_dir("web")
            .output()
            .expect("Failed to run npm install");
    }

    let npm_mode = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let npm_script = if npm_mode == "release" {
        "build-release"
    } else {
        "build-debug"
    };
    Command::new(NPM)
        .args(["run", npm_script])
        .current_dir("web")
        .output()
        .expect("Failed to build web frontend");

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

    // Copy built assets to $OUT_DIR/web
    if out_dir.join("web").exists() {
        std::fs::remove_dir_all(out_dir.join("web")).expect("Failed to remove old web dir");
    }
    copy_dir("web/dist", out_dir.join("web")).expect("Failed to copy web/dist to $OUT_DIR/web");

    println!("cargo:rerun-if-changed=openapi.yaml");
}
