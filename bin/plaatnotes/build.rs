/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple note-taking app

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD_NO_PAD as BASE64_NO_PAD;
use copy_dir::copy_dir;

fn main() {
    // Generate test password hash at compile time
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let salt = b"test_salt_fixed!";
    let hash = pbkdf2::pbkdf2_hmac_sha256(b"password123", salt, 100_000, 32);
    let test_password_hash = format!(
        "$pbkdf2-sha256$t=100000${}${}",
        BASE64_NO_PAD.encode(salt),
        BASE64_NO_PAD.encode(hash)
    );
    std::fs::write(out_dir.join("test_password_hash.txt"), test_password_hash)
        .expect("Failed to write test_password_hash.txt");

    // Generate openapi file
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
