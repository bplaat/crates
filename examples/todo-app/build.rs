/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A Todo GUI example

use std::fs;
use std::path::Path;
use std::process::Command;

use copy_dir::copy_dir;

fn main() {
    #[cfg(windows)]
    const NPM: &str = "npm.cmd";
    #[cfg(not(windows))]
    const NPM: &str = "npm";

    // Install npm packages
    if !Path::new("web/node_modules").exists() {
        Command::new(NPM)
            .arg("ci")
            .arg("--prefer-offline")
            .current_dir("web")
            .output()
            .expect("Failed to run npm install");
    }

    // Run npm run build
    fn print_rerun_if_changed(dir: &Path) {
        for entry in fs::read_dir(dir).expect("Failed to read dir") {
            let path = entry.expect("Failed to read entry").path();
            if path.is_dir() {
                print_rerun_if_changed(&path);
            } else {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    println!("cargo:rerun-if-changed=web/index.html");
    print_rerun_if_changed(Path::new("web/src"));

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

    // Copy all files from web/dist to OUT_DIR
    let out_dir = std::env::var("OUT_DIR").expect("Should be some");
    copy_dir("web/dist", Path::new(&out_dir).join("web"))
        .expect("Failed to copy web/dist files to $OUT_DIR");
}
