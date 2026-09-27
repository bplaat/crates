/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Command-line behavior for the base64 binary.

use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str], input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_base64"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start base64");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input)
        .expect("write input");
    child.wait_with_output().expect("wait for base64")
}

#[test]
fn encodes_and_decodes_wrapped_input() {
    let input = vec![b'a'; 80];
    let encoded = run(&[], &input);
    assert!(encoded.status.success());
    assert_eq!(
        encoded
            .stdout
            .split(|byte| *byte == b'\n')
            .next()
            .map(<[u8]>::len),
        Some(76)
    );
    let decoded = run(&["-d"], &encoded.stdout);
    assert!(decoded.status.success());
    assert_eq!(decoded.stdout, input);
}

#[test]
fn rejects_invalid_input() {
    assert!(!run(&["-d"], b"invalid!").status.success());
}
