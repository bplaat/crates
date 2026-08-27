/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Integration tests for the ccc transpiler.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use regex::Regex;

const SANITIZER_CFLAGS: &str =
    "-O1 -g -fsanitize=address,undefined -fno-omit-frame-pointer -fno-sanitize-recover=all";
const SANITIZER_LDFLAGS: &str = "-fsanitize=address,undefined";
const OPTIMIZED_CFLAGS: &str = "-O1 -g";

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(group_index: usize) -> Result<Self, String> {
        let path = env::temp_dir().join(format!(
            "ccontinue_test_suite_{}_{}",
            std::process::id(),
            group_index
        ));
        fs::create_dir_all(&path).map_err(|error| error.to_string())?;
        Ok(Self { path })
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

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

fn build_group(
    directory: &Path,
    group_index: usize,
    test_files: &[&str],
) -> Result<String, String> {
    let ccc_bin = env!("CARGO_BIN_EXE_ccc");
    let source_path = directory.join(format!("group_{group_index}.cc"));
    let exe_path = directory
        .join(format!("group_{group_index}{}", env::consts::EXE_SUFFIX))
        .to_str()
        .expect("temp path is valid UTF-8")
        .to_owned();
    let mut merged_source = String::new();
    let mut dispatcher = String::from(
        "\nint main(int argc, char** argv) {\n\
             if (argc != 2) return 127;\n",
    );
    let cast_regex =
        Regex::new(r"\bcast<([A-Za-z_][A-Za-z0-9_]*)>").expect("valid cast expression regex");
    let instanceof_regex = Regex::new(r"\binstanceof<([A-Za-z_][A-Za-z0-9_]*)>")
        .expect("valid instanceof expression regex");
    for test_file in test_files {
        let stem = Path::new(test_file)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("test filename is valid UTF-8");
        let source = fs::read_to_string(test_file).map_err(|error| error.to_string())?;
        let renamed_main = format!("int test_{stem}_main(void)");
        let mut helper_declarations = String::new();
        for captures in cast_regex.captures_iter(&source) {
            helper_declarations += &format!(
                "static {} _cast_{}(void* obj);\n",
                &captures[1], &captures[1]
            );
        }
        for captures in instanceof_regex.captures_iter(&source) {
            helper_declarations +=
                &format!("static bool _instanceof_{}(void* obj);\n", &captures[1]);
        }
        if source.contains(" in ") {
            helper_declarations += "static IIterable _cast_IIterable(void* obj);\n";
        }
        let replacement = format!("{helper_declarations}{renamed_main}");
        let renamed_source = source.replacen("int main(void)", &replacement, 1);
        if renamed_source == source {
            return Err(format!("can't find main function in {test_file}"));
        }
        merged_source.push_str(&renamed_source);
        merged_source.push('\n');
        dispatcher +=
            &format!("    if (strcmp(argv[1], \"{stem}\") == 0) return test_{stem}_main();\n");
    }
    dispatcher += "    return 127;\n}\n";
    merged_source += &dispatcher;
    fs::write(&source_path, merged_source).map_err(|error| error.to_string())?;

    let std_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/std");
    let (cflags, ldflags) = compiler_flags();
    let result = Command::new(ccc_bin)
        .arg(&source_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("-I")
        .arg(std_dir)
        .env("CC", "clang")
        .env("CFLAGS", cflags)
        .env("LDFLAGS", ldflags)
        .env("CCONTINUE_SANITIZE_STD", "1")
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

fn run_normal(
    exe_path: &str,
    test_name: &str,
    expected_exit: i32,
    expected_stdout: &str,
) -> Result<(), String> {
    let mut command = Command::new(exe_path);
    command
        .arg(test_name)
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

fn run_group(group_index: usize, test_files: &[&str]) {
    let directory = TestDirectory::new(group_index).expect("create test directory");
    let executable =
        build_group(&directory.path, group_index, test_files).expect("build test group");
    let mut failures = Vec::new();
    for test_file in test_files {
        let (expected_exit, expected_stdout) = parse_test_meta(test_file);
        let test_name = Path::new(test_file)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("test filename is valid UTF-8");
        if let Err(error) = run_normal(&executable, test_name, expected_exit, &expected_stdout) {
            failures.push(format!("{test_name}: {error}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} fixture(s) failed:\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
