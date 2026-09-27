/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Command-line behavior for the unzip binary.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

fn temp_directory() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "unzip-test-{}-{unique}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&path).expect("create test directory");
    path
}

#[test]
fn lists_and_extracts_without_overwriting() {
    let root = temp_directory();
    let archive = root.join("nested.zip");
    fs::write(&archive, include_bytes!("fixtures/nested.zip")).expect("write archive");
    let list = Command::new(env!("CARGO_BIN_EXE_unzip"))
        .arg("-l")
        .arg(&archive)
        .output()
        .expect("list archive");
    assert!(list.status.success());
    assert_eq!(list.stdout, b"folder/hello.txt\n");

    let destination = root.join("out");
    let extract = || {
        Command::new(env!("CARGO_BIN_EXE_unzip"))
            .arg(&archive)
            .arg("-d")
            .arg(&destination)
            .output()
            .expect("extract archive")
    };
    assert!(extract().status.success());
    assert_eq!(
        fs::read(destination.join("folder/hello.txt")).expect("extracted file"),
        b"hello world"
    );
    assert!(!extract().status.success());
    fs::remove_dir_all(root).expect("remove test directory");
}

#[test]
fn rejects_absolute_archive_paths() {
    let root = temp_directory();
    let archive = root.join("absolute.zip");
    fs::write(&archive, include_bytes!("fixtures/absolute.zip")).expect("write archive");
    let output = Command::new(env!("CARGO_BIN_EXE_unzip"))
        .arg(&archive)
        .arg("-d")
        .arg(root.join("out"))
        .output()
        .expect("extract archive");
    assert!(!output.status.success());
    fs::remove_dir_all(root).expect("remove test directory");
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_directory() {
    use std::os::unix::fs::symlink;

    let root = temp_directory();
    let archive = root.join("nested.zip");
    fs::write(&archive, include_bytes!("fixtures/nested.zip")).expect("write archive");
    let destination = root.join("out");
    fs::create_dir(&destination).expect("create destination");
    symlink(&root, destination.join("folder")).expect("create symlink");
    let output = Command::new(env!("CARGO_BIN_EXE_unzip"))
        .arg(&archive)
        .arg("-d")
        .arg(&destination)
        .output()
        .expect("extract archive");
    assert!(!output.status.success());
    assert!(!root.join("hello.txt").exists());
    fs::remove_dir_all(root).expect("remove test directory");
}
