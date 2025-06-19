/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::path::Path;
use std::process::exit;
use std::{env, fs};

use crate::args::{Profile, Subcommand, parse_args};
use crate::executor::Executor;
use crate::manifest::Manifest;
use crate::tasks::android::{
    detect_android, generate_android_dex_tasks, generate_android_final_apk_tasks,
    generate_android_res_tasks, run_android_apk,
};
use crate::tasks::cx::{
    detect_bundle, detect_c, detect_cpp, detect_cx, detect_objc, detect_objcpp,
    generate_bundle_tasks, generate_c_tasks, generate_cpp_tasks, generate_ld_tasks,
    generate_objc_tasks, generate_objcpp_tasks, generate_test_main, run_bundle, run_ld,
    run_ld_tests,
};
use crate::tasks::java::{
    detect_jar, detect_java, generate_jar_tasks, generate_javac_tasks, run_jar, run_java_class,
};
use crate::utils::{format_bytes, index_files};

mod args;
mod executor;
mod log;
mod manifest;
mod sha1;
mod tasks;
mod utils;

// MARK: Subcommands
fn subcommand_clean(target_dir: &str) {
    if !Path::new(target_dir).exists() {
        println!("Removed 0 files");
        return;
    }

    let files = index_files(target_dir);
    let total_size: u64 = files
        .iter()
        .map(|file| fs::metadata(file).expect("Can't read file metadata").len())
        .sum();
    println!(
        "Removed {} files, {} total",
        files.len(),
        format_bytes(total_size)
    );
    fs::remove_dir_all(target_dir).expect("Can't remove target directory");
}

fn subcommand_help() {
    println!(
        r"Usage: bob [SUBCOMMAND] [OPTIONS]

Options:
  -C <dir>, --manifest-dir    Change to directory <dir> before doing anything
  -T <dir>, --target-dir      Write artifacts to directory <dir>
  -r, --release               Build artifacts in release mode

Subcommands:
  clean                       Remove build artifacts
  build                       Build the project
  help                        Print this help message
  rebuild                     Clean and build the project
  run                         Run the build artifact after building
  test                        Run the unit tests
  version                     Print the version number"
    );
}

fn subcommand_version() {
    println!("bob v{}", env!("CARGO_PKG_VERSION"));
}

// MARK: Main
pub(crate) struct Project {
    target_dir: String,
    manifest: Manifest,
    profile: Profile,
    is_test: bool,
    source_files: Vec<String>,
}

fn main() {
    let args = parse_args();

    if args.subcommand == Subcommand::Help {
        subcommand_help();
        return;
    }
    if args.subcommand == Subcommand::Version {
        subcommand_version();
        return;
    }

    // Change working directory to manifest_dir
    if env::set_current_dir(&args.manifest_dir).is_err() {
        eprintln!("Can't change directory to: {}", args.manifest_dir);
        exit(1);
    }

    // Read manifest
    let manifest: Manifest =
        basic_toml::from_str(&fs::read_to_string("bob.toml").unwrap_or_else(|err| {
            eprintln!("Can't read bob.toml file: {}", err);
            exit(1);
        }))
        .unwrap_or_else(|err| {
            eprintln!("Can't parse bob.toml file: {}", err);
            exit(1);
        });

    // Clean build artifacts
    if args.subcommand == Subcommand::Clean {
        subcommand_clean(&args.target_dir);
        return;
    }

    // Rebuild artifacts
    if args.subcommand == Subcommand::Rebuild {
        subcommand_clean(&args.target_dir);
    }

    // Check target directory
    if !Path::new(&args.target_dir).exists() {
        fs::create_dir(&args.target_dir).expect("Failed to create target directory");
    }

    // Generate tasks
    let mut project = Project {
        manifest,
        target_dir: args.target_dir,
        profile: args.profile,
        is_test: args.subcommand == Subcommand::Test,
        source_files: index_files("src/"),
    };

    let mut executor = Executor::new();
    // FIXME: Fix bug where test corrupts target directory
    if detect_cx(&project) && project.is_test {
        generate_test_main(&mut project);
    }
    if detect_c(&project) {
        generate_c_tasks(&project, &mut executor);
    }
    if detect_cpp(&project) {
        generate_cpp_tasks(&project, &mut executor);
    }
    if detect_objc(&project) {
        generate_objc_tasks(&project, &mut executor);
    }
    if detect_objcpp(&project) {
        generate_objcpp_tasks(&project, &mut executor);
    }
    if detect_android() {
        generate_android_res_tasks(&mut project, &mut executor);
    }
    if detect_java(&project) {
        generate_javac_tasks(&project, &mut executor);
    }
    if detect_android() {
        generate_android_dex_tasks(&project, &mut executor);
        generate_android_final_apk_tasks(&project, &mut executor);
    }
    if detect_cx(&project) {
        generate_ld_tasks(&project, &mut executor);
    }
    if detect_jar(&project) {
        generate_jar_tasks(&project, &mut executor);
    }
    if detect_bundle(&project) {
        generate_bundle_tasks(&project, &mut executor);
    }
    executor.execute(&format!("{}/bob.log", &project.target_dir));

    // Run build artifact
    if args.subcommand == Subcommand::Run {
        if detect_bundle(&project) {
            run_bundle(&project);
        }
        if detect_jar(&project) {
            run_jar(&project);
        }
        if detect_android() {
            run_android_apk(&project);
        }
        if detect_cx(&project) {
            run_ld(&project);
        }
        if detect_java(&project) {
            run_java_class(&project);
        }
        eprintln!("No build artifact to run");
    }

    // Run unit tests
    if args.subcommand == Subcommand::Test {
        if detect_cx(&project) {
            run_ld_tests(&project);
        }
        eprintln!("No test artifact to run");
    }
}
