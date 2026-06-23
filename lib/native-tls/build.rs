/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! native-tls build script: handles OpenSSL linking for the system (non-vendored) path.
//! When the `vendored` feature is set, rustls is used instead and no linking is needed.

fn main() {
    println!("cargo::rustc-check-cfg=cfg(openssl_v10x)");
    println!("cargo::rustc-check-cfg=cfg(openssl_v4xx)");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let vendored = std::env::var("CARGO_FEATURE_VENDORED").is_ok();

    // On Linux/other without vendored: dynamically link the system OpenSSL.
    // With vendored: rustls handles everything in pure Rust; no linker flags needed.
    if target_os != "macos" && target_os != "windows" && !vendored {
        println!("cargo::rustc-link-lib=ssl");
        println!("cargo::rustc-link-lib=crypto");
        detect_openssl_version();
    }
}

// Emit cfg flags based on the system OpenSSL version:
//   openssl_v10x    -- 1.0.x (uses BIO_new_bio_pair + SSLv23_client_method)
//   openssl_v4xx -- 4.x+  (uses SSL_set1_dnsname instead of SSL_set1_host)
// Only called for dynamic system linking; vendored builds use rustls.
fn detect_openssl_version() {
    // Try pkg-config first (most reliable on Linux/BSD).
    // pkg-config output is just the version string, e.g. "3.0.2\n".
    if let Ok(out) = std::process::Command::new("pkg-config")
        .args(["--modversion", "openssl"])
        .output()
        && out.status.success()
    {
        emit_version_cfg(&out.stdout);
        return;
    }
    // Fallback: ask the openssl binary ("OpenSSL 3.0.2 15 Mar 2022\n").
    if let Ok(out) = std::process::Command::new("openssl")
        .arg("version")
        .output()
        && out.status.success()
    {
        // Skip the "OpenSSL " prefix to get to the version string.
        let version = out.stdout.strip_prefix(b"OpenSSL ").unwrap_or(&out.stdout);
        emit_version_cfg(version);
    }
    // If detection fails, assume modern OpenSSL (1.1+ / 3.x / 4.x).
}

fn emit_version_cfg(version: &[u8]) {
    if version.starts_with(b"1.0.") {
        println!("cargo::rustc-cfg=openssl_v10x");
    } else if version.first().is_some_and(|&b| b >= b'4') {
        println!("cargo::rustc-cfg=openssl_v4xx");
    }
    // 1.1.x / 3.x: no extra cfg needed; they use the default code path.
}
