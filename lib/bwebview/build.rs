/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal windows webview2 example

fn main() {
    #[cfg(windows)]
    {
        // Link with the correct WebView2Loader library based on architecture
        let target = std::env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
        let lib_dir = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"),
        )
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
        let target_env =
            std::env::var("CARGO_CFG_TARGET_ENV").expect("CARGO_CFG_TARGET_ENV not set");
        if target_env == "msvc" {
            println!("cargo:rustc-link-lib=static=WebView2LoaderStatic");
        } else {
            println!("cargo:rustc-link-lib=dylib=WebView2Loader");

            // Copy WebView2Loader.dll to output directory for dynamic linking
            let out_dir =
                std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
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
    }
}
