/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

use std::collections::HashMap;
use std::path::Path;
use std::process::exit;
use std::{env, fs};

use crate::args::{Args, Profile, Subcommand, parse_args, subcommand_help};
use crate::executor::Executor;
use crate::manifest::{Dependency, JarDependency, Manifest, PackageType};
use crate::tasks::android::{
    detect_android, generate_android_dex_tasks, generate_android_final_apk_tasks,
    generate_android_res_tasks, link_android_classpath, run_android_apk,
};
use crate::tasks::bundle::{bundle_is_lipo, detect_bundle, generate_bundle_tasks, run_bundle};
use crate::tasks::cx::{
    copy_cx_headers, detect_c, detect_cpp, detect_cx, detect_objc, detect_objcpp, generate_c_tasks,
    generate_cpp_tasks, generate_cx_test_main, generate_ld_cunit_tests, generate_ld_tasks,
    generate_objc_tasks, generate_objcpp_tasks, run_ld, run_ld_cunit_tests,
};
use crate::tasks::jvm::{
    detect_jar, detect_java_kotlin, detect_kotlin, download_extract_jar_tasks, generate_jar_tasks,
    generate_javac_kotlinc_tasks, run_jar, run_java_class, run_junit_tests,
};
use crate::tasks::template::{detect_template, process_templates};
use crate::utils::{format_bytes, index_files, read_env_file};

mod args;
mod executor;
mod log;
mod manifest;
mod tasks;
mod utils;

// MARK: Subcommands
fn print_dir_remove_stats(path: &str) {
    let files = index_files(path);
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

fn subcommand_clean(target_dir: &str, print: bool) {
    if !Path::new(target_dir).exists() {
        if print {
            println!("Removed 0 files");
        }
        return;
    }

    if print {
        print_dir_remove_stats(target_dir);
    }
    fs::remove_dir_all(target_dir).expect("Can't remove target directory");
}

fn subcommand_clean_cache() {
    let cache_dir = dirs::cache_dir().expect("Failed to get cache directory");
    let global_bob_cache_dir = format!("{}/bob", cache_dir.display());
    if !Path::new(&global_bob_cache_dir).exists() {
        println!("Removed 0 files");
        return;
    }

    print_dir_remove_stats(&global_bob_cache_dir);
    fs::remove_dir_all(global_bob_cache_dir).expect("Can't remove bob directory");
}

fn subcommand_version() {
    println!("bob v{}", env!("CARGO_PKG_VERSION"));
}

// MARK: Bobje
#[derive(Clone)]
pub(crate) struct Bobje {
    target_dir: String,
    profile: Profile,
    target: Option<String>,
    // ...
    r#type: PackageType,
    name: String,
    version: String,
    manifest_dir: String,
    manifest: Manifest,
    jar: Option<JarDependency>,
    source_files: Vec<String>,
    dependencies: HashMap<String, Bobje>,
}

impl Bobje {
    fn new(args: &Args, manifest_dir: &str, executor: &mut Executor) -> Self {
        // Read manifest
        let manifest_path = format!("{manifest_dir}/bob.toml");
        let mut manifest: Manifest =
            basic_toml::from_str(&fs::read_to_string(&manifest_path).unwrap_or_else(|err| {
                eprintln!("Can't read {manifest_path} file: {err}");
                exit(1);
            }))
            .unwrap_or_else(|err| {
                eprintln!("Can't parse {manifest_path} file: {err}");
                exit(1);
            });
        let source_files = index_files(&format!("{manifest_dir}/src/"));

        // Add libSystem dep when Cx on macOS
        if cfg!(target_os = "macos") && detect_cx(&source_files) {
            manifest.dependencies.insert(
                "libsystem".to_string(),
                Dependency::Library {
                    library: "System".to_string(),
                },
            );
        }
        // Add Foundation framework dep when using Objective-C
        if detect_objc(&source_files) || detect_objcpp(&source_files) {
            manifest.dependencies.insert(
                "foundation".to_string(),
                Dependency::Framework {
                    framework: "Foundation".to_string(),
                },
            );
        }

        // Add Kotlin stdlib when Kotlin is used
        if detect_kotlin(&source_files) {
            // Manual dependency in https://repo1.maven.org/maven2/org/jetbrains/kotlin/kotlin-stdlib/2.0.0/kotlin-stdlib-2.0.0.pom
            // FIXME: Automatically resolve maven dependency trees by fetching and parsing pom.xml
            manifest.dependencies.insert(
                "kotlin-stdlib".to_string(),
                Dependency::Maven {
                    maven: "org.jetbrains.kotlin:kotlin-stdlib:2.0.0".to_string(),
                },
            );
            manifest.dependencies.insert(
                "jetbrains-annotations".to_string(),
                Dependency::Maven {
                    maven: "org.jetbrains:annotations:13.0".to_string(),
                },
            );
        }

        // Add test libraries dependencies
        if args.profile == Profile::Test {
            if detect_cx(&source_files) {
                manifest.dependencies.insert(
                    "cunit".to_string(),
                    Dependency::PkgConfig {
                        pkg_config: "cunit".to_string(),
                    },
                );
            }
            if detect_java_kotlin(&source_files) {
                // https://repo1.maven.org/maven2/junit/junit/4.13.2/junit-4.13.2.pom
                manifest.dependencies.insert(
                    "junit".to_string(),
                    Dependency::Maven {
                        maven: "junit:junit:4.13.2".to_string(),
                    },
                );
                manifest.dependencies.insert(
                    "hamcrest".to_string(),
                    Dependency::Maven {
                        maven: "org.hamcrest:hamcrest-core:1.3".to_string(),
                    },
                );
            }
        }

        // Build dependencies
        let mut dependencies = HashMap::new();
        for (dep_name, dep) in &manifest.dependencies {
            if let Dependency::Path { path } = &dep {
                let dep_bobje = Bobje::new(args, &format!("{manifest_dir}/{path}"), executor);
                if dep_bobje.r#type != PackageType::Library {
                    eprintln!("Dependency '{dep_name}' in {path} is not a library");
                    exit(1);
                }
                dependencies.insert(dep_bobje.name.clone(), dep_bobje);
            }

            if let Dependency::Jar { jar } = &dep {
                let dep_bobje = Bobje::new_external_jar(args, dep_name, jar, executor);
                dependencies.insert(dep_bobje.name.clone(), dep_bobje);
            }

            if let Dependency::Maven { maven } = &dep {
                let mut parts = maven.split(':');
                let package = parts.next().expect("Can't parse maven string").to_string();
                let name = parts.next().expect("Can't parse maven string").to_string();
                let version = parts.next().expect("Can't parse maven string").to_string();

                // NOTE: Fix for kotlin stdlib package
                let package_override = if package == "org.jetbrains.kotlin" {
                    Some("kotlin".to_string())
                } else {
                    None
                };

                let url = format!(
                    "https://repo1.maven.org/maven2/{}/{name}/{version}/{name}-{version}.jar",
                    package.replace(".", "/")
                );
                let jar = JarDependency {
                    package,
                    package_override,
                    version,
                    path: None,
                    url: Some(url),
                };
                let dep_bobje = Bobje::new_external_jar(args, dep_name, &jar, executor);
                dependencies.insert(dep_bobje.name.clone(), dep_bobje);
            }
        }

        // Generate tasks
        let mut bobje = Self {
            target_dir: args.target_dir.clone(),
            profile: args.profile,
            target: args.target.clone(),
            // ...
            r#type: manifest.package.r#type,
            name: manifest.package.name.clone(),
            version: manifest.package.version.clone(),
            manifest_dir: manifest_dir.to_string(),
            manifest,
            jar: None,
            source_files,
            dependencies,
        };

        fn generate_bobje_tasks(bobje: &mut Bobje, executor: &mut Executor) {
            if detect_template(&bobje.source_files) {
                process_templates(bobje, executor);
            }
            if bobje.profile == Profile::Test && detect_cx(&bobje.source_files) {
                generate_cx_test_main(bobje);
            }
            if detect_cx(&bobje.source_files) {
                copy_cx_headers(bobje, executor);
            }
            if detect_c(&bobje.source_files) {
                generate_c_tasks(bobje, executor);
            }
            if detect_cpp(&bobje.source_files) {
                generate_cpp_tasks(bobje, executor);
            }
            if detect_objc(&bobje.source_files) {
                generate_objc_tasks(bobje, executor);
            }
            if detect_objcpp(&bobje.source_files) {
                generate_objcpp_tasks(bobje, executor);
            }
            if detect_android(bobje) {
                generate_android_res_tasks(bobje, executor);
            }
            if detect_java_kotlin(&bobje.source_files) {
                if detect_android(bobje) {
                    link_android_classpath(bobje);
                }
                generate_javac_kotlinc_tasks(bobje, executor);
            }
            if detect_cx(&bobje.source_files) {
                if bobje.profile == Profile::Test {
                    generate_ld_cunit_tests(bobje, executor);
                } else {
                    generate_ld_tasks(bobje, executor);
                }
            }
            if bobje.r#type == PackageType::Binary {
                if detect_android(bobje) {
                    generate_android_dex_tasks(bobje, executor);
                    generate_android_final_apk_tasks(bobje, executor);
                }
                if bobje.profile != Profile::Test && detect_jar(bobje) {
                    generate_jar_tasks(bobje, executor);
                }
            }
        }

        if bobje.r#type == PackageType::Binary && detect_bundle(&bobje) && bundle_is_lipo(&bobje) {
            let mut bobje_x86_64 = bobje.clone();
            bobje_x86_64.target = Some("x86_64-apple-darwin".to_string());
            generate_bobje_tasks(&mut bobje_x86_64, executor);

            let mut bobje_aarch64 = bobje.clone();
            bobje_aarch64.target = Some("aarch64-apple-darwin".to_string());
            generate_bobje_tasks(&mut bobje_aarch64, executor);
        } else {
            generate_bobje_tasks(&mut bobje, executor);
        }
        if bobje.r#type == PackageType::Binary && detect_bundle(&bobje) {
            generate_bundle_tasks(&bobje, executor);
        }

        bobje
    }

    fn new_external_jar(
        args: &Args,
        name: &str,
        jar: &JarDependency,
        executor: &mut Executor,
    ) -> Self {
        let bobje = Self {
            target_dir: args.target_dir.clone(),
            profile: args.profile,
            target: args.target.clone(),
            // ...
            r#type: PackageType::ExternalJar,
            name: name.to_string(),
            version: jar.version.clone(),
            manifest_dir: "".to_string(),
            manifest: Manifest::default(),
            jar: Some(jar.clone()),
            source_files: vec![],
            dependencies: HashMap::new(),
        };
        download_extract_jar_tasks(&bobje, executor, jar);
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

    if args.subcommand == Subcommand::CleanCache {
        subcommand_clean_cache();
        return;
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

    // Read .env file
    _ = read_env_file(".env");

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
    let bobje = Bobje::new(&args, ".", &mut executor);
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
        if detect_cx(&bobje.source_files) {
            run_ld(&bobje);
        }
        if detect_java_kotlin(&bobje.source_files) {
            run_java_class(&bobje);
        }
        eprintln!("No build artifact to run");
    }

    // Run unit tests
    if args.subcommand == Subcommand::Test {
        if detect_cx(&bobje.source_files) {
            run_ld_cunit_tests(&bobje);
        }
        if detect_java_kotlin(&bobje.source_files) {
            run_junit_tests(&bobje);
        }
        eprintln!("No test artifact to run");
    }
}
