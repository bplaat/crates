// MARK: Main
mod args;
mod transpiler;
mod types;
mod utils;

use std::env;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use args::parse_args;
use transpiler::Transpiler;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn tempfile_path(ext: &str) -> String {
    let mut tmp = env::temp_dir();
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = format!("ccc_{}_{n}{ext}", std::process::id());
    tmp.push(name);
    tmp.to_str().expect("temp dir path is valid UTF-8").to_owned()
}

fn main() {
    let args = parse_args();

    let cc = env::var("CC").unwrap_or_else(|_| "gcc".to_owned());
    let script_dir = {
        let exe = env::current_exe().expect("cannot get current exe path");
        let exe_dir = exe.parent().expect("exe has parent directory").to_owned();
        // During cargo run/test, exe is in target/debug/ — walk up to find std/
        let mut d = exe_dir.clone();
        loop {
            if d.join("std").exists() {
                break d;
            }
            if let Some(parent) = d.parent() {
                d = parent.to_owned();
            } else {
                break exe_dir;
            }
        }
    };
    let std_dir = script_dir.join("std");

    let mut include_paths: Vec<String> = vec![
        ".".to_owned(),
        std_dir.to_str().expect("std dir path is valid UTF-8").to_owned(),
    ];
    include_paths.extend(args.include_paths);

    let mut source_paths = args.files.clone();
    if !args.flag_source && !args.flag_compile
        && let Ok(entries) = std::fs::read_dir(&std_dir)
    {
        for entry in entries.flatten() {
            let p = entry.path();
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "c" || ext == "cc" {
                source_paths.push(p.to_str().expect("source path is valid UTF-8").to_owned());
            }
        }
    }

    let mut transpiler = Transpiler::new(include_paths.clone());
    let mut object_paths: Vec<String> = Vec::new();

    for path in &source_paths {
        if path.ends_with(".o") {
            object_paths.push(path.clone());
            continue;
        }

        let source_path = if path.ends_with(".hh") || path.ends_with(".cc") {
            let sp = if let Some(ref o) = args.output {
                o.clone()
            } else if args.flag_source {
                path.replace(".cc", ".c").replace(".hh", ".h")
            } else {
                tempfile_path(".c")
            };
            transpiler.reset();
            let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("[ERROR] Can't read {}: {}", path, e);
                std::process::exit(1);
            });
            let result = transpiler.transpile(path, path.ends_with(".hh"), &text);
            std::fs::write(&sp, &result).unwrap_or_else(|e| {
                eprintln!("[ERROR] Can't write {}: {}", sp, e);
                std::process::exit(1);
            });
            if args.flag_source {
                std::process::exit(0);
            }
            sp
        } else {
            path.clone()
        };

        let object_path = if args.flag_compile {
            args.output
                .clone()
                .unwrap_or_else(|| path.replace(".cc", ".o").replace(".c", ".o"))
        } else {
            tempfile_path(".o")
        };
        object_paths.push(object_path.clone());

        let mut cmd = Command::new(&cc);
        cmd.args(["--std=c23", "-Wall", "-Wextra", "-Wpedantic", "-Werror"]);
        for inc in &include_paths {
            cmd.arg(format!("-I{}", inc));
        }
        cmd.args(["-c", &source_path, "-o", &object_path]);
        let status = cmd.status().unwrap_or_else(|e| {
            eprintln!("[ERROR] Failed to run compiler: {}", e);
            std::process::exit(1);
        });
        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
        if args.flag_compile {
            std::process::exit(0);
        }
    }

    let exe_path = args.output.clone().unwrap_or_else(|| {
        let base = &args.files[0];
        if cfg!(target_os = "windows") {
            base.replace(".cc", ".exe")
        } else {
            base.replace(".cc", "")
        }
    });

    let mut link_cmd = Command::new(&cc);
    link_cmd.args(&object_paths);
    link_cmd.args(["-o", &exe_path]);
    let status = link_cmd.status().unwrap_or_else(|e| {
        eprintln!("[ERROR] Failed to run linker: {}", e);
        std::process::exit(1);
    });
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    if args.flag_run {
        std::process::exit(
            Command::new(format!("./{}", exe_path))
                .status()
                .map(|s| s.code().unwrap_or(0))
                .unwrap_or(1),
        );
    } else if args.flag_run_leaks {
        if cfg!(target_os = "macos") {
            std::process::exit(
                Command::new("leaks")
                    .args(["--atExit", "--", &format!("./{}", exe_path)])
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
                        &format!("./{}", exe_path),
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
