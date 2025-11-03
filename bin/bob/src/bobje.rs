/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::fs;
use std::process::exit;

use crate::args::{Args, Profile};
use crate::bobje;
use crate::executor::ExecutorBuilder;
use crate::manifest::{Dependency, LibraryType, Manifest};
use crate::tasks::android::{
    detect_android, generate_android_dex_tasks, generate_android_final_apk_tasks,
    generate_android_res_tasks, link_android_classpath,
};
use crate::tasks::bundle::{bundle_is_lipo, detect_bundle, generate_bundle_tasks};
use crate::tasks::cx::{
    copy_cx_headers, detect_asm, detect_c, detect_cpp, detect_cx, detect_objc, detect_objcpp,
    generate_asm_tasks, generate_c_tasks, generate_cpp_tasks, generate_cx_test_main,
    generate_ld_cunit_tests, generate_ld_tasks, generate_objc_tasks, generate_objcpp_tasks,
};
use crate::tasks::jvm::{
    CLASSPATH_SEPARATOR, detect_jar, detect_java_kotlin, detect_kotlin, generate_jar_tasks,
    generate_javac_kotlinc_tasks,
};
use crate::tasks::template::{detect_template, process_templates};
use crate::utils::index_files;

// MARK: PackageType
#[derive(Copy, Clone)]
pub(crate) enum PackageType {
    Binary,
    Library { r#type: LibraryType },
    ExternalJar,
}

impl PackageType {
    pub(crate) fn is_binary(&self) -> bool {
        matches!(self, PackageType::Binary)
    }

    pub(crate) fn is_library(&self) -> bool {
        matches!(self, PackageType::Library { .. })
    }

    pub(crate) fn is_external_jar(&self) -> bool {
        matches!(self, PackageType::ExternalJar)
    }
}

// MARK: Bobje
#[derive(Clone)]
pub(crate) struct Bobje {
    pub target_dir: String,
    pub profile: Profile,
    pub is_main: bool,
    pub r#type: PackageType,
    pub name: String,
    pub version: String,
    pub target: Option<String>,
    pub manifest_dir: String,
    pub manifest: Manifest,
    #[cfg(feature = "javac-server")]
    pub use_javac_server: bool,
    pub source_files: Vec<String>,
    pub dependencies: HashMap<String, Bobje>,
    pub dependencies_classpath: Vec<String>,
}

impl Bobje {
    pub(crate) fn new(
        args: &Args,
        manifest_dir: &str,
        executor: &mut ExecutorBuilder,
        is_main: bool,
    ) -> Self {
        // MARK: Read manifest
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

        // Merge platform specific build config
        if cfg!(target_os = "macos")
            && let Some(macos_build) = &manifest.build.macos
        {
            manifest.build.merge(*macos_build.clone());
        }
        if cfg!(target_os = "linux")
            && let Some(linux_build) = &manifest.build.linux
        {
            manifest.build.merge(*linux_build.clone());
        }
        if cfg!(windows)
            && let Some(windows_build) = &manifest.build.windows
        {
            manifest.build.merge(*windows_build.clone());
        }

        // Build target triple
        let target = args
            .target
            .clone()
            .or_else(|| manifest.build.target.clone());

        // MARK: Auto dependencies
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

        // MARK: Build dependencies
        let mut dependencies = HashMap::new();
        let mut dependencies_classpath = Vec::new();
        for (dep_name, dep) in &manifest.dependencies {
            if let Dependency::Path { path } = &dep {
                let dep_bobje =
                    Bobje::new(args, &format!("{manifest_dir}/{path}"), executor, false);
                if !dep_bobje.r#type.is_library() {
                    eprintln!("Dependency '{dep_name}' in {path} is not a library");
                    exit(1);
                }
                dependencies.insert(dep_bobje.name.clone(), dep_bobje);
            }

            if let Dependency::Maven { maven } = &dep {
                let mut parts = maven.split(':');
                let package = parts.next().expect("Can't parse maven string").to_string();
                let name = parts.next().expect("Can't parse maven string").to_string();
                let version = parts.next().expect("Can't parse maven string").to_string();

                // Write dummy pom.xml file
                let tmpdir = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
                let bob_tmp_dir = format!("{}/bob", tmpdir);
                fs::create_dir_all(&bob_tmp_dir).expect("Failed to create bob tmp directory");

                let pom_path = format!("{}/pom.xml", bob_tmp_dir);
                let pom_contents = format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
                <project xmlns="http://maven.apache.org/POM/4.0.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
                    <modelVersion>4.0.0</modelVersion>
                    <groupId>com.example</groupId>
                    <artifactId>simple</artifactId>
                    <version>1.0</version>
                    <dependencies>
                        <dependency>
                            <groupId>{package}</groupId>
                            <artifactId>{name}</artifactId>
                            <version>{version}</version>
                        </dependency>
                    </dependencies>
                </project>
                "#,
                );
                fs::write(&pom_path, pom_contents).expect("Failed to write temporary pom.xml file");

                let classpath_path = format!("{}/classpath", bob_tmp_dir);
                let status = std::process::Command::new("mvn")
                    .args([
                        "dependency:build-classpath",
                        "-f",
                        &pom_path,
                        &format!("-Dmdep.outputFile={}", classpath_path),
                    ])
                    .status()
                    .expect("Failed to run mvn");
                if !status.success() {
                    eprintln!("Failed to build maven classpath for {maven}");
                    exit(1);
                }

                let classpath =
                    fs::read_to_string(&classpath_path).expect("Failed to read classpath output");
                for path in classpath.trim().split(CLASSPATH_SEPARATOR) {
                    dependencies_classpath.push(path.to_string());
                }
            }
        }

        // MARK: Generate tasks
        let mut bobje = Self {
            target_dir: args.target_dir.clone(),
            profile: args.profile,
            is_main,
            r#type: if let Some(library) = &manifest.library {
                PackageType::Library {
                    r#type: library.r#type,
                }
            } else {
                PackageType::Binary
            },
            name: manifest.package.name.clone(),
            version: manifest.package.version.clone(),
            target,
            manifest_dir: manifest_dir.to_string(),
            manifest,
            #[cfg(feature = "javac-server")]
            use_javac_server: !args.disable_javac_server,
            source_files,
            dependencies,
            dependencies_classpath,
        };

        let mut visit_bobje = |bobje: &mut Bobje| {
            if detect_template(&bobje.source_files) {
                process_templates(bobje, executor);
            }
            if bobje.is_main && bobje.profile == Profile::Test && detect_cx(&bobje.source_files) {
                generate_cx_test_main(bobje);
            }
            if detect_cx(&bobje.source_files) {
                copy_cx_headers(bobje, executor);
            }
            if detect_asm(&bobje.source_files) {
                generate_asm_tasks(bobje, executor);
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
            if bobje.r#type.is_binary() {
                if detect_android(bobje) {
                    generate_android_dex_tasks(bobje, executor);
                    generate_android_final_apk_tasks(bobje, executor);
                }
                if bobje.profile != Profile::Test && detect_jar(bobje) {
                    generate_jar_tasks(bobje, executor);
                }
            }
        };

        if bobje.r#type.is_binary() && detect_bundle(&bobje) && bundle_is_lipo(&bobje) {
            let mut bobje_x86_64 = bobje.clone();
            bobje_x86_64.target = Some("x86_64-apple-darwin".to_string());
            visit_bobje(&mut bobje_x86_64);

            let mut bobje_aarch64 = bobje.clone();
            bobje_aarch64.target = Some("aarch64-apple-darwin".to_string());
            visit_bobje(&mut bobje_aarch64);
        } else {
            visit_bobje(&mut bobje);
        }
        if bobje.r#type.is_binary() && detect_bundle(&bobje) {
            generate_bundle_tasks(&bobje, executor);
        }

        bobje
    }

    pub(crate) fn out_dir(&self) -> String {
        format!("{}/{}", self.target_dir, self.profile)
    }

    pub(crate) fn out_dir_with_target(&self) -> String {
        if let Some(target) = &self.target {
            format!("{}/{}/{}", self.target_dir, target, self.profile)
        } else {
            self.out_dir()
        }
    }
}
