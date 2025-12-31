/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("README.md")]

use std::env;
use std::path::PathBuf;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set") == "windows" {
        // Link with the correct WebView2Loader library based on architecture
        let target = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
        let lib_dir =
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"))
                .join("webview2")
                .join(if target == "x86_64" {
                    "x64"
                } else if target == "aarch64" {
                    "arm64"
                } else if target == "x86" {
                    "x86"
                } else {
                    panic!("Unsupported architecture")
                });

        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
        match target_env.as_str() {
            "msvc" => {
                println!("cargo:rustc-link-lib=static=WebView2LoaderStatic");
            }
            "gnu" => {
                println!("cargo:rustc-link-lib=dylib=WebView2Loader");

                // Copy WebView2Loader.dll to output directory for dynamic linking
                let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
                std::fs::copy(
                    lib_dir.join("WebView2Loader.dll"),
                    out_dir
                        .parent()
                        .expect("Should be some")
                        .parent()
                        .expect("Should be some")
                        .parent()
                        .expect("Should be some")
                        .join("WebView2Loader.dll"),
                )
                .expect("Failed to copy WebView2Loader.dll");
            }
            other => {
                panic!("unsupported target environment: {other}");
            }
        }
    }
}
