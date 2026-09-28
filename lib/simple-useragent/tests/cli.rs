/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Command-line behavior tests.

use std::process::Command;

#[test]
fn classifies_multiple_inputs() {
    let output = Command::new(env!("CARGO_BIN_EXE_simple-useragent"))
        .args([
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:134.0) Gecko/20100101 Firefox/134.0",
            "unknown user agent",
        ])
        .output()
        .expect("CLI runs");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 output");
    assert!(stdout.contains("Client: Firefox"));
    assert!(stdout.contains("Client version: 134.0"));
    assert!(stdout.contains("OS: Mac OS X"));
    assert!(stdout.contains("User agent: unknown user agent"));
}

#[test]
fn requires_an_input() {
    let output = Command::new(env!("CARGO_BIN_EXE_simple-useragent"))
        .output()
        .expect("CLI runs");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 output");
    assert!(stderr.contains("Usage: simple-useragent"));
}
