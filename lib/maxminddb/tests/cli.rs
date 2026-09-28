/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Command-line behavior for the maxminddb-lookup binary.

use std::path::PathBuf;
use std::process::Command;

fn test_database() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/GeoLite2-City-Test.mmdb")
}

#[test]
fn looks_up_known_ip_address() {
    let output = Command::new(env!("CARGO_BIN_EXE_maxminddb-lookup"))
        .arg(test_database())
        .arg("89.160.20.128")
        .output()
        .expect("run maxminddb-lookup");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 output");
    assert!(stdout.contains("IP address: 89.160.20.128"));
    assert!(stdout.contains("Country:         Sweden (SE)"));
}

#[test]
fn reports_missing_address() {
    let output = Command::new(env!("CARGO_BIN_EXE_maxminddb-lookup"))
        .arg(test_database())
        .arg("192.0.2.1")
        .output()
        .expect("run maxminddb-lookup");
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .expect("UTF-8 output")
            .contains("No data found for this IP address.")
    );
}

#[test]
fn rejects_invalid_arguments() {
    let binary = env!("CARGO_BIN_EXE_maxminddb-lookup");
    for args in [vec![], vec!["missing.mmdb"]] {
        let output = Command::new(binary)
            .args(args)
            .output()
            .expect("run maxminddb-lookup");
        assert!(!output.status.success());
        assert!(
            String::from_utf8(output.stderr)
                .expect("UTF-8 output")
                .contains("Usage: maxminddb-lookup")
        );
    }

    let invalid_ip = Command::new(binary)
        .arg(test_database())
        .arg("invalid-ip")
        .output()
        .expect("run maxminddb-lookup");
    assert!(!invalid_ip.status.success());
    assert!(
        String::from_utf8(invalid_ip.stderr)
            .expect("UTF-8 output")
            .contains("not a valid IP address")
    );
}

#[test]
fn reports_missing_database() {
    let output = Command::new(env!("CARGO_BIN_EXE_maxminddb-lookup"))
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/missing.mmdb"))
        .arg("89.160.20.128")
        .output()
        .expect("run maxminddb-lookup");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .expect("UTF-8 output")
            .contains("failed to open")
    );
}
