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
    assert_eq!(
        stdout,
        "User agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:134.0) Gecko/20100101 Firefox/134.0\n\
         Client: Firefox\n\
         Client version: 134.0\n\
         OS: Mac OS X\n\
         OS version: 10.15\n\
         \n\
         User agent: unknown user agent\n\
         Client: Other\n\
         Client version: unknown\n\
         OS: Other\n\
         OS version: unknown\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn requires_an_input() {
    let output = Command::new(env!("CARGO_BIN_EXE_simple-useragent"))
        .output()
        .expect("CLI runs");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 output");
    assert_eq!(
        stderr,
        "Usage: simple-useragent <user-agent> [user-agent ...]\n"
    );
}
