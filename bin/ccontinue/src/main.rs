/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

mod args;
mod temp;
mod transpiler;
mod types;
mod utils;

use std::collections::HashMap;
use std::process::Command;

use args::parse_args;
use rust_embed::Embed;
use temp::TempFileManager;
use transpiler::Transpiler;

#[derive(Embed)]
#[folder = "std"]
struct StdFiles;

/// Set up standard library files: extract embedded files, transpile .cc files,
/// and write outputs to temp_dir/ccontinue for compiler access.
/// Returns (std_source_paths, std_temp_dir, transpiler_with_embedded_includes).
fn setup_std_files(
    temp_mgr: &TempFileManager,
    include_paths: &[String],
) -> (Vec<String>, String, Transpiler) {
    // Build an in-memory map of embedded .hh files for the transpiler
    let mut embedded_includes: HashMap<String, String> = HashMap::new();
    // Collect embedded .c / .h / .cc files that need compiler access
    let mut std_c_files: Vec<(String, Vec<u8>)> = Vec::new(); // (filename, content)

    for filename in StdFiles::iter() {
        let file = StdFiles::get(filename.as_ref()).expect("embedded file exists");
        if filename.ends_with(".hh") {
            let text = String::from_utf8_lossy(&file.data).into_owned();
            embedded_includes.insert(filename.into_owned(), text);
        } else {
            std_c_files.push((filename.into_owned(), file.data.to_vec()));
        }
    }

    // Get the std temp directory
    let std_temp_dir = temp_mgr
        .base_dir()
        .to_str()
        .expect("std temp dir is valid UTF-8")
        .to_owned();

    let mut std_source_paths: Vec<String> = Vec::new();

    // Prepare a transpiler seeded with the embedded .hh map to transpile std .cc files
    let mut std_transpiler = Transpiler::new(include_paths.to_vec());
    std_transpiler.set_embedded_includes(embedded_includes.clone());

    for (filename, content) in &std_c_files {
        if filename.ends_with(".h") || filename.ends_with(".c") {
            // Plain C files: write directly for the compiler
            let dest = temp_mgr.base_dir().join(filename);
            std::fs::write(&dest, content).unwrap_or_else(|e| {
                eprintln!("[ERROR] Can't write std file {filename}: {e}");
                std::process::exit(1);
            });
            if filename.ends_with(".c") {
                std_source_paths.push(dest.to_str().expect("dest path is valid UTF-8").to_owned());
            }
        } else if filename.ends_with(".cc") {
            // CCC sources: transpile in-memory, write .c output to temp dir
            let text = String::from_utf8_lossy(content).into_owned();
            std_transpiler.reset();
            let c_output = std_transpiler.transpile(filename, false, &text);
            let out_name = filename.replace(".cc", ".c");
            let dest = temp_mgr.base_dir().join(&out_name);
            std::fs::write(&dest, &c_output).unwrap_or_else(|e| {
                eprintln!("[ERROR] Can't write transpiled std file {out_name}: {e}");
                std::process::exit(1);
            });
            std_source_paths.push(dest.to_str().expect("dest path is valid UTF-8").to_owned());
        }
    }

    (std_source_paths, std_temp_dir, std_transpiler)
}

/// Transpile and compile user source files (.cc, .hh, .c).
/// Returns (object_paths, embedded_includes for next stage).
#[allow(clippy::too_many_arguments)]
fn transpile_and_compile_sources(
    temp_mgr: &TempFileManager,
    transpiler: &mut Transpiler,
    include_paths: &[String],
    source_paths: &[String],
    output: &Option<String>,
    flag_source: bool,
    flag_compile: bool,
    cc: &str,
) -> Vec<String> {
    let mut object_paths: Vec<String> = Vec::new();

    for path in source_paths {
        if path.ends_with(".o") {
            object_paths.push(path.clone());
            continue;
        }

        let source_path = if path.ends_with(".hh") || path.ends_with(".cc") {
            let sp = if flag_source {
                if let Some(o) = output {
                    o.clone()
                } else {
                    path.replace(".cc", ".c").replace(".hh", ".h")
                }
            } else {
                temp_mgr.temp_file(".c")
            };
            transpiler.reset();
            let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("[ERROR] Can't read {path}: {e}");
                std::process::exit(1);
            });
            let result = transpiler.transpile(path, path.ends_with(".hh"), &text);
            std::fs::write(&sp, &result).unwrap_or_else(|e| {
                eprintln!("[ERROR] Can't write {sp}: {e}");
                std::process::exit(1);
            });
            if flag_source {
                std::process::exit(0);
            }
            sp
        } else {
            path.clone()
        };

        let object_path = if flag_compile {
            output
                .clone()
                .unwrap_or_else(|| path.replace(".cc", ".o").replace(".c", ".o"))
        } else {
            temp_mgr.temp_file(".o")
        };
        object_paths.push(object_path.clone());

        let mut cmd = Command::new(cc);
        cmd.args(["--std=c11", "-Wall", "-Wextra", "-Wpedantic", "-Werror"]);
        for inc in include_paths {
            cmd.arg(format!("-I{inc}"));
        }
        cmd.args(["-c", &source_path, "-o", &object_path]);
        let status = cmd.status().unwrap_or_else(|e| {
            eprintln!("[ERROR] Failed to run compiler: {e}");
            std::process::exit(1);
        });
        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
        if flag_compile {
            std::process::exit(0);
        }
    }

    object_paths
}

/// Link object files and optionally run the resulting executable.
fn link_and_run(
    object_paths: &[String],
    output: &Option<String>,
    files: &[String],
    cc: &str,
    flag_run: bool,
    flag_run_leaks: bool,
) {
    let exe_path = output.clone().unwrap_or_else(|| {
        let base = &files[0];
        if cfg!(target_os = "windows") {
            base.replace(".cc", ".exe")
        } else {
            base.replace(".cc", "")
        }
    });

    let mut link_cmd = Command::new(cc);
    link_cmd.args(object_paths);
    link_cmd.args(["-o", &exe_path]);
    let status = link_cmd.status().unwrap_or_else(|e| {
        eprintln!("[ERROR] Failed to run linker: {e}");
        std::process::exit(1);
    });
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    if flag_run {
        std::process::exit(
            Command::new(format!("./{exe_path}"))
                .status()
                .map(|s| s.code().unwrap_or(0))
                .unwrap_or(1),
        );
    } else if flag_run_leaks {
        if cfg!(target_os = "macos") {
            std::process::exit(
                Command::new("leaks")
                    .args(["--atExit", "--", &format!("./{exe_path}")])
                    .status()
                    .map(|s| s.code().unwrap_or(0))
                    .unwrap_or(1),
            );
        } else if cfg!(target_os = "linux") {
            std::process::exit(
                Command::new("valgrind")
                    .args([
                        "--leak-check=full",
                        "--show-leak-kinds=all",
                        "--track-origins=yes",
                        &format!("./{exe_path}"),
                    ])
                    .status()
                    .map(|s| s.code().unwrap_or(0))
                    .unwrap_or(1),
            );
        } else {
            eprintln!("[ERROR] Memory leak checks are not supported on this platform");
            std::process::exit(1);
        }
    }
}

fn main() {
    let args = parse_args();
    let cc = std::env::var("CC").unwrap_or_else(|_| "gcc".to_owned());
    let temp_mgr = TempFileManager::new();

    // Build include paths
    let std_temp_str = temp_mgr
        .base_dir()
        .to_str()
        .expect("std temp dir is valid UTF-8")
        .to_owned();
    let mut include_paths: Vec<String> = vec![".".to_owned(), std_temp_str];
    include_paths.extend(args.include_paths.clone());

    // Set up standard library files
    let (std_source_paths, _std_temp_dir, std_transpiler) =
        setup_std_files(&temp_mgr, &include_paths);

    // Prepare source list
    let mut source_paths = args.files.clone();
    if !args.flag_source && !args.flag_compile {
        source_paths.extend(std_source_paths);
    }

    // Transpile and compile user sources
    let mut transpiler = std_transpiler;
    let object_paths = transpile_and_compile_sources(
        &temp_mgr,
        &mut transpiler,
        &include_paths,
        &source_paths,
        &args.output,
        args.flag_source,
        args.flag_compile,
        &cc,
    );

    // Link and optionally run
    link_and_run(
        &object_paths,
        &args.output,
        &args.files,
        &cc,
        args.flag_run,
        args.flag_run_leaks,
    );
}
