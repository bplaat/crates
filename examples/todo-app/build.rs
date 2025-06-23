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
    #[cfg(windows)]
    const NPM: &str = "npm.cmd";
    #[cfg(not(windows))]
    const NPM: &str = "npm";

    // Install npm packages
    if !Path::new("web/node_modules").exists() {
        Command::new(NPM)
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
    let out_dir = env::var("OUT_DIR").expect("$OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("web");
    if dest_path.exists() {
        fs::remove_dir_all(&dest_path).expect("Failed to remove old web dir");
    }
    fs::create_dir_all(&dest_path).expect("Failed to create web dir");

    // Recursively copy directory
    fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let dst_path = dst.join(entry.file_name());
            if ty.is_dir() {
                copy_dir_all(&entry.path(), &dst_path)?;
            } else {
                fs::copy(entry.path(), dst_path)?;
            }
        }
        Ok(())
    }
    copy_dir_all(Path::new("web/dist"), &dest_path)
        .expect("Failed to copy web/dist files to $OUT_DIR");
}
