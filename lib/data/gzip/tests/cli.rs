/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(missing_docs)]

use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn compresses_and_decompresses_a_file() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("gzip-test-{}-{nonce}", std::process::id()));
    fs::create_dir(&directory).expect("create test directory");
    let source = directory.join("notes.txt");
    let compressed = directory.join("notes.txt.gz");
    fs::write(&source, b"hello world").expect("write input");

    let executable = env!("CARGO_BIN_EXE_gzip");
    assert!(
        Command::new(executable)
            .arg(&source)
            .status()
            .expect("run gzip")
            .success()
    );
    assert!(!source.exists());
    assert!(compressed.exists());

    fs::write(&source, b"existing file").expect("write destination");
    assert!(
        !Command::new(executable)
            .args(["-d"])
            .arg(&compressed)
            .output()
            .expect("run gzip -d")
            .status
            .success()
    );
    assert_eq!(
        fs::read(&source).expect("read existing file"),
        b"existing file"
    );
    assert!(compressed.exists());

    fs::remove_file(&source).expect("remove destination");
    assert!(
        Command::new(executable)
            .args(["-d"])
            .arg(&compressed)
            .status()
            .expect("run gzip -d")
            .success()
    );
    assert_eq!(fs::read(&source).expect("read output"), b"hello world");
    assert!(!compressed.exists());
    fs::remove_dir_all(&directory).expect("remove test directory");
}
