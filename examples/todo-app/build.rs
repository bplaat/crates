/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A Todo GUI example

use std::path::Path;
use std::process::Command;
use std::{env, fs};

fn main() {
    // Install npm packages
    if !Path::new("web/node_modules").exists() {
        Command::new("npm")
            .arg("install")
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

    Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir("web")
        .output()
        .expect("Failed to run npm run build");

    // Copy all files in web/dist to OUT_DIR
    Command::new("cp")
        .arg("-rf")
        .arg("web/dist")
        .arg(format!(
            "{}/web",
            env::var("OUT_DIR").expect("$OUT_DIR not set")
        ))
        .output()
        .expect("Failed to copy web/dist files to $OUT_DIR");
}
