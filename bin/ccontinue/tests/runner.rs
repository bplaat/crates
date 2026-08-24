/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Integration tests for the ccc transpiler.

#![allow(dead_code)]

use std::path::Path;
use std::process::Command;
use std::{env, fs};

const SANITIZER_CFLAGS: &str =
    "-O1 -g -fsanitize=address,undefined -fno-omit-frame-pointer -fno-sanitize-recover=all";
const SANITIZER_LDFLAGS: &str = "-fsanitize=address,undefined";
const OPTIMIZED_CFLAGS: &str = "-O1 -g";

const fn compiler_flags() -> (&'static str, &'static str) {
    if cfg!(target_os = "windows") {
        (OPTIMIZED_CFLAGS, "")
    } else {
        (SANITIZER_CFLAGS, SANITIZER_LDFLAGS)
    }
}

fn parse_test_meta(filepath: &str) -> (i32, String) {
    let content = fs::read_to_string(filepath).expect("read test file");
    let mut expected_exit = 0i32;
    let mut out_lines: Vec<String> = Vec::new();
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("// EXIT: ") {
            expected_exit = rest.trim().parse().unwrap_or(0);
            continue;
        }
        if let Some(rest) = line.strip_prefix("// OUT: ") {
            out_lines.push(rest.to_owned());
            continue;
        }
        if !line.is_empty() && !line.starts_with("//") {
            break;
        }
    }
    let expected_stdout = if out_lines.is_empty() {
        String::new()
    } else {
        out_lines.join("\n") + "\n"
    };
    (expected_exit, expected_stdout)
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

fn build_test(test_file: &str) -> Result<String, String> {
    let ccc_bin = env!("CARGO_BIN_EXE_ccc");
    let stem = Path::new(test_file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("ccc_test");
    let exe_path = env::temp_dir()
        .join(format!("ccc_test_{stem}{}", env::consts::EXE_SUFFIX))
        .to_str()
        .expect("temp path is valid UTF-8")
        .to_owned();
    let std_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/std");
    let (cflags, ldflags) = compiler_flags();
    let result = Command::new(ccc_bin)
        .arg(test_file)
        .arg("-o")
        .arg(&exe_path)
        .arg("-I")
        .arg(std_dir)
        .env("CC", "clang")
        .env("CFLAGS", cflags)
        .env("LDFLAGS", ldflags)
        .output()
        .map_err(|e| format!("failed to run ccc: {e}"))?;

    if !result.status.success() || !Path::new(&exe_path).exists() {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_owned();
        let stdout = String::from_utf8_lossy(&result.stdout).trim().to_owned();
        return Err(if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            "unknown build error".to_owned()
        });
    }

    Ok(exe_path)
}

fn run_normal(exe_path: &str, expected_exit: i32, expected_stdout: &str) -> Result<(), String> {
    let mut command = Command::new(exe_path);
    command
        .env("ASAN_OPTIONS", "halt_on_error=1")
        .env("UBSAN_OPTIONS", "halt_on_error=1:print_stacktrace=1");
    let result = command
        .output()
        .map_err(|e| format!("failed to run {exe_path}: {e}"))?;
    let actual_exit = result.status.code().unwrap_or(-1);
    if actual_exit != expected_exit {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_owned();
        return Err(format!(
            "exit code {actual_exit} (expected {expected_exit})\n{stderr}"
        ));
    }
    let actual_stdout = normalize_newlines(&String::from_utf8_lossy(&result.stdout));
    if actual_stdout != expected_stdout {
        let exp_repr = format!("{:?}", &expected_stdout[..expected_stdout.len().min(300)]);
        let got_repr = format!("{:?}", &actual_stdout[..actual_stdout.len().min(300)]);
        return Err(format!(
            "stdout mismatch\n    expected: {exp_repr}\n    got:      {got_repr}"
        ));
    }
    Ok(())
}

fn run_test(test_file: &str) {
    let (expected_exit, expected_stdout) = parse_test_meta(test_file);

    let exe_path = match build_test(test_file) {
        Ok(p) => p,
        Err(e) => panic!("build error: {e}"),
    };

    let run_result = run_normal(&exe_path, expected_exit, &expected_stdout);

    // Clean up binary
    let _ = fs::remove_file(&exe_path);

    if let Err(e) = run_result {
        panic!("{}", e);
    }
}

#[test]
fn normalizes_windows_newlines() {
    assert_eq!(normalize_newlines("first\r\nsecond\r\n"), "first\nsecond\n");
}

include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
