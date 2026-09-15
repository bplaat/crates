/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Build the embedded PlaatMoney web application.

use std::env;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::Command;

use copy_dir::copy_dir;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    const NPM: &str = if cfg!(windows) { "npm.cmd" } else { "npm" };

    println!("cargo:rerun-if-changed=../../.npmrc");
    println!("cargo:rerun-if-changed=../../package.json");
    println!("cargo:rerun-if-changed=../../package-lock.json");
    {
        let npm_lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(env::temp_dir().join("bplaat-crates-npm-install.lock"))
            .expect("Failed to open npm install lock file");
        npm_lock
            .lock()
            .expect("Failed to lock npm install lock file");
        if !Path::new("../../node_modules/.package-lock.json").exists() {
            let status = Command::new(NPM)
                .arg("ci")
                .current_dir("../..")
                .status()
                .expect("Failed to run npm ci");
            assert!(status.success(), "npm ci failed with {status}");
        }
    }

    print_rerun(Path::new("web"));
    let status = Command::new(NPM)
        .arg("run")
        .arg(if cfg!(debug_assertions) {
            "build-debug"
        } else {
            "build-release"
        })
        .current_dir("web")
        .status()
        .expect("Failed to run frontend build");
    assert!(status.success(), "frontend build failed with {status}");

    copy_dir("web/dist", out_dir.join("web")).expect("Failed to copy frontend assets");
}

fn print_rerun(dir: &Path) {
    for entry in std::fs::read_dir(dir).expect("Failed to read frontend directory") {
        let path = entry.expect("Failed to read frontend entry").path();
        if path.is_dir() {
            let file_name = path.file_name().expect("Path should have a file name");
            if file_name != "dist" && file_name != "node_modules" {
                print_rerun(&path);
            }
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
