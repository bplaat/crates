/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;
use std::process::exit;
use std::{env, fs};

use crate::args::{Args, Profile, Subcommand, parse_args, subcommand_help};
use crate::executor::Executor;
use crate::manifest::Manifest;
use crate::tasks::android::{
    detect_android, generate_android_dex_tasks, generate_android_final_apk_tasks,
    generate_android_res_tasks, link_android_classpath, run_android_apk,
};
use crate::tasks::bundle::{bundle_is_lipo, detect_bundle, generate_bundle_tasks, run_bundle};
use crate::tasks::cx::{
    copy_cx_headers, detect_c, detect_cpp, detect_cx, detect_objc, detect_objcpp, generate_c_tasks,
    generate_cpp_tasks, generate_cx_test_main, generate_ld_tasks, generate_objc_tasks,
    generate_objcpp_tasks, run_ld, run_ld_tests,
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
fn subcommand_clean(target_dir: &str, print: bool) {
    if !Path::new(target_dir).exists() {
        if print {
            println!("Removed 0 files");
        }
        return;
    }

    if print {
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
    }
    fs::remove_dir_all(target_dir).expect("Can't remove target directory");
}

fn subcommand_version() {
    println!("bob v{}", env!("CARGO_PKG_VERSION"));
}

// MARK: Bobje
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum BobjeType {
    Binary,
    Library,
}

#[derive(Clone)]
pub(crate) struct Bobje {
    target_dir: String,
    profile: Profile,
    target: Option<String>,
    is_test: bool,
    // ...
    r#type: BobjeType,
    manifest_dir: String,
    manifest: Manifest,
    source_files: Vec<String>,
    dependencies: HashMap<String, Bobje>,
}

impl Bobje {
    fn new(args: &Args, manifest_dir: &str, r#type: BobjeType, executor: &mut Executor) -> Self {
        // Read manifest
        let manifest_path = format!("{}/bob.toml", manifest_dir);
        let manifest: Manifest =
            basic_toml::from_str(&fs::read_to_string(&manifest_path).unwrap_or_else(|err| {
                eprintln!("Can't read {} file: {}", manifest_path, err);
                exit(1);
            }))
            .unwrap_or_else(|err| {
                eprintln!("Can't parse {} file: {}", manifest_path, err);
                exit(1);
            });

        // Build dependencies
        let mut dependencies = HashMap::new();
        for dep in manifest.dependencies.values() {
            let dep_bobje = Bobje::new(
                args,
                &format!("{}/{}", manifest_dir, dep.path),
                BobjeType::Library,
                executor,
            );
            dependencies.insert(dep_bobje.manifest.package.name.clone(), dep_bobje);
        }

        // Generate tasks
        let mut bobje = Self {
            target_dir: args.target_dir.clone(),
            profile: args.profile,
            target: args.target.clone(),
            is_test: args.subcommand == Subcommand::Test,
            // ...
            r#type,
            manifest_dir: manifest_dir.to_string(),
            manifest,
            source_files: index_files(&format!("{}/src/", manifest_dir)),
            dependencies,
        };

        fn generate_bobje_tasks(bobje: &mut Bobje, executor: &mut Executor) {
            // FIXME: Fix bug where test corrupts target directory
            if bobje.r#type == BobjeType::Binary && detect_cx(bobje) && bobje.is_test {
                generate_cx_test_main(bobje);
            }
            if detect_cx(bobje) {
                copy_cx_headers(bobje, executor);
            }
            if detect_c(bobje) {
                generate_c_tasks(bobje, executor);
            }
            if detect_cpp(bobje) {
                generate_cpp_tasks(bobje, executor);
            }
            if detect_objc(bobje) {
                generate_objc_tasks(bobje, executor);
            }
            if detect_objcpp(bobje) {
                generate_objcpp_tasks(bobje, executor);
            }
            if detect_android(bobje) {
                generate_android_res_tasks(bobje, executor);
            }
            if detect_java(bobje) {
                if detect_android(bobje) {
                    link_android_classpath(bobje);
                }
                generate_javac_tasks(bobje, executor);
            }
            if detect_cx(bobje) {
                generate_ld_tasks(bobje, executor);
            }
            if bobje.r#type == BobjeType::Binary && detect_android(bobje) {
                generate_android_dex_tasks(bobje, executor);
                generate_android_final_apk_tasks(bobje, executor);
            }
            if bobje.r#type == BobjeType::Binary && detect_jar(bobje) {
                generate_jar_tasks(bobje, executor);
            }
        }

        if r#type == BobjeType::Binary && detect_bundle(&bobje) && bundle_is_lipo(&bobje) {
            let mut bobje_x86_64 = bobje.clone();
            bobje_x86_64.target = Some("x86_64-apple-darwin".to_string());
            generate_bobje_tasks(&mut bobje_x86_64, executor);

            let mut bobje_aarch64 = bobje.clone();
            bobje_aarch64.target = Some("aarch64-apple-darwin".to_string());
            generate_bobje_tasks(&mut bobje_aarch64, executor);
        } else {
            generate_bobje_tasks(&mut bobje, executor);
        }
        if r#type == BobjeType::Binary && detect_bundle(&bobje) {
            generate_bundle_tasks(&bobje, executor);
        }

        bobje
    }

    fn out_dir(&self) -> String {
        format!("{}/{}", self.target_dir, self.profile)
    }

    fn out_dir_with_target(&self) -> String {
        if let Some(target) = &self.target {
            format!("{}/{}/{}", self.target_dir, target, self.profile)
        } else {
            self.out_dir()
        }
    }
}

// MARK: Main
fn main() {
    let args = parse_args();
    #[cfg(windows)]
    if !args.verbose && env::var("NO_COLOR").is_err() && env::var("CI").is_err() {
        enable_ansi_support::enable_ansi_support().expect("Can't enable ANSI support");
    }

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

    // Check if bob.toml exists
    if !Path::new("bob.toml").exists() {
        eprintln!("Can't find bob.toml file");
        exit(1);
    }

    // Clean build artifacts
    if args.subcommand == Subcommand::Clean {
        subcommand_clean(&args.target_dir, true);
        return;
    }
    // Rebuild artifacts
    if args.subcommand == Subcommand::Rebuild {
        subcommand_clean(&args.target_dir, false);
    }

    // Check target directory
    if !Path::new(&args.target_dir).exists() {
        fs::create_dir(&args.target_dir).expect("Failed to create target directory");
    }

    // Build main bobje
    let mut executor = Executor::new();
    let bobje = Bobje::new(&args, ".", BobjeType::Binary, &mut executor);
    executor.execute(
        &format!("{}/bob.log", &args.target_dir),
        args.verbose,
        args.thread_count,
    );

    // Run build artifact
    if args.subcommand == Subcommand::Run {
        if detect_bundle(&bobje) {
            run_bundle(&bobje);
        }
        if detect_jar(&bobje) {
            run_jar(&bobje);
        }
        if detect_android(&bobje) {
            run_android_apk(&bobje);
        }
        if detect_cx(&bobje) {
            run_ld(&bobje);
        }
        if detect_java(&bobje) {
            run_java_class(&bobje);
        }
        eprintln!("No build artifact to run");
    }

    // Run unit tests
    if args.subcommand == Subcommand::Test {
        if detect_cx(&bobje) {
            run_ld_tests(&bobje);
        }
        eprintln!("No test artifact to run");
    }
}
