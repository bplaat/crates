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
        let lib_dir = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"),
        )
        .join("webview2")
        .join(if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else {
            panic!("Unsupported architecture")
        });

        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        if cfg!(target_env = "msvc") {
            println!("cargo:rustc-link-lib=static=WebView2LoaderStatic");
        } else {
            println!("cargo:rustc-link-lib=dylib=WebView2Loader");
            // FIXME: Maybe copy WebView2Loader.dll to output dir?
        }
    }
}
