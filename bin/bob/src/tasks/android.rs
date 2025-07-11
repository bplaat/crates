/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::Path;
use std::process::{Command, exit};
use std::{env, fs};

use crate::executor::Executor;
use crate::manifest::AndroidMetadata;
use crate::tasks::jvm::{find_modules, get_class_name};
use crate::utils::{index_files, write_file_when_different};
use crate::{Bobje, Profile};

// MARK: Android vars
struct AndroidVars {
    id: String,
    android_metadata: AndroidMetadata,
    platform_jar: String,
    command_line_tools_path: String,
    build_tools_path: String,
    platform_tools_path: String,
}

impl AndroidVars {
    fn new(bobje: &Bobje) -> Self {
        let id = bobje.manifest.package.id.as_ref().unwrap_or_else(|| {
            eprintln!("Manifest package id is required");
            exit(1);
        });
        let android_metadata = bobje
            .manifest
            .package
            .metadata
            .android
            .clone()
            .unwrap_or_default();
        let android_home = env::var("ANDROID_HOME").expect("$ANDROID_HOME env var must be set");
        let platform_jar = format!(
            "{}/platforms/android-{}/android.jar",
            android_home, android_metadata.target_sdk_version
        );

        // Determine command_line_tools_path: prefer 'latest', else pick highest versioned folder
        let cmdline_tools_dir = format!("{}/cmdline-tools", android_home);
        let latest_path = format!("{}/latest/bin", cmdline_tools_dir);
        let command_line_tools_path = if Path::new(&latest_path).exists() {
            latest_path
        } else {
            // Find highest x.x versioned folder
            let mut highest_version: Option<(u32, u32, String)> = None;
            if let Ok(entries) = fs::read_dir(&cmdline_tools_dir) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();
                    // Match folders like "10.0" or "8.1"
                    if let Some((major, minor)) = file_name_str
                        .split_once('.')
                        .and_then(|(maj, min)| maj.parse::<u32>().ok().zip(min.parse::<u32>().ok()))
                    {
                        if highest_version
                            .as_ref()
                            .is_none_or(|(h_maj, h_min, _)| (major, minor) > (*h_maj, *h_min))
                        {
                            highest_version = Some((major, minor, file_name_str.to_string()));
                        }
                    }
                }
            }
            if let Some((_, _, folder)) = highest_version {
                format!("{}/{}/bin", cmdline_tools_dir, folder)
            } else {
                eprintln!("No valid cmdline-tools found in {cmdline_tools_dir}");
                exit(1);
            }
        };

        let build_tools_path = format!(
            "{}/build-tools/{}.0.0",
            android_home, android_metadata.target_sdk_version
        );
        let platform_tools_path = format!("{}/platform-tools", android_home);
        Self {
            id: id.clone(),
            android_metadata: android_metadata.clone(),
            platform_jar,
            command_line_tools_path,
            build_tools_path,
            platform_tools_path,
        }
    }
}

pub(crate) fn detect_android(bobje: &Bobje) -> bool {
    Path::new(&format!("{}/AndroidManifest.xml", bobje.manifest_dir)).exists()
}

// MARK: Android resources
pub(crate) fn generate_android_res_tasks(bobje: &mut Bobje, executor: &mut Executor) {
    let vars = AndroidVars::new(bobje);

    let res_dir = format!(
        "{}/{}",
        bobje.manifest_dir, vars.android_metadata.resources_dir
    );
    if !Path::new(&res_dir).exists() {
        return;
    }

    // aapt2_compile tasks
    let compiled_res_dir = format!("{}/res/{}", bobje.out_dir(), vars.id.replace('.', "/"));
    for res_file in index_files(&res_dir) {
        let mut compiled_res_file = format!(
            "{}/{}.flat",
            compiled_res_dir,
            res_file
                .trim_start_matches(&res_dir)
                .trim_start_matches(['/', '\\'])
                .replace(['/', '\\'], "_")
        );
        if compiled_res_file.contains("/values") {
            compiled_res_file = compiled_res_file.replace(".xml", ".arsc");
        }
        executor.add_task_cmd(
            format!(
                "{}/aapt2 compile --no-crunch {} -o {}",
                vars.build_tools_path, res_file, compiled_res_dir
            ),
            vec![res_file],
            vec![compiled_res_file.clone()],
        );
    }

    // aapt2_link task
    let r_java_path = format!(
        "{}/src-gen/{}/R.java",
        bobje.out_dir(),
        vars.id.replace('.', "/")
    );
    bobje.source_files.push(r_java_path.clone());

    if bobje.r#type == crate::BobjeType::Binary {
        let dest = format!("{}/{}-unaligned.apk", bobje.out_dir(), bobje.name);

        let mut link_inputs = vec![format!("{}/AndroidManifest.xml", bobje.manifest_dir)];
        let mut link_command = vec![
            format!("{}/aapt2", vars.build_tools_path),
            "link".to_string(),
        ];
        fn add_bobje_resources(
            bobje: &Bobje,
            link_command: &mut Vec<String>,
            link_inputs: &mut Vec<String>,
        ) {
            for dependency_bobje in bobje.dependencies.values() {
                add_bobje_resources(dependency_bobje, link_command, link_inputs);
            }
            if detect_android(bobje) {
                let android_metadata = bobje
                    .manifest
                    .package
                    .metadata
                    .android
                    .clone()
                    .unwrap_or_default();

                // Add assets
                let assets_dir = format!("{}/{}", bobje.manifest_dir, android_metadata.assets_dir);
                if fs::metadata(&assets_dir).is_ok() {
                    for asset in index_files(&assets_dir) {
                        link_inputs.push(asset.clone());
                    }
                }
                if fs::metadata(&assets_dir).is_ok() {
                    link_command.push("-A".to_string());
                    link_command.push(assets_dir.clone());
                }

                // Add compiled resources
                let compiled_res_dir = format!(
                    "{}/res/{}",
                    bobje.out_dir(),
                    bobje
                        .manifest
                        .package
                        .id
                        .as_ref()
                        .expect("Should be some")
                        .replace('.', "/")
                );
                let res_dir = format!("{}/{}", bobje.manifest_dir, android_metadata.resources_dir);
                if !Path::new(&res_dir).exists() {
                    return;
                }
                for res_file in index_files(&res_dir) {
                    let mut compiled_res_file = format!(
                        "{}/{}.flat",
                        compiled_res_dir,
                        res_file
                            .trim_start_matches(&res_dir)
                            .trim_start_matches(['/', '\\'])
                            .replace(['/', '\\'], "_")
                    );
                    if compiled_res_file.contains("/values") {
                        compiled_res_file = compiled_res_file.replace(".xml", ".arsc");
                    }
                    link_inputs.push(compiled_res_file);
                }
                link_command.push(format!("{}/*.flat", compiled_res_dir));
            }
        }
        add_bobje_resources(bobje, &mut link_command, &mut link_inputs);

        link_command.extend(vec![
            "--manifest".to_string(),
            format!("{}/AndroidManifest.xml", bobje.manifest_dir),
            "--java".to_string(),
            format!("{}/src-gen", bobje.out_dir()),
            "--version-name".to_string(),
            bobje.version.clone(),
            "--version-code".to_string(),
            parse_version_to_code(&bobje.version).to_string(),
            "--min-sdk-version".to_string(),
            vars.android_metadata.min_sdk_version.to_string(),
            "--target-sdk-version".to_string(),
            vars.android_metadata.target_sdk_version.to_string(),
            "-I".to_string(),
            vars.platform_jar.clone(),
            "-o".to_string(),
            dest.to_string(),
        ]);
        executor.add_task_cmd(
            link_command.join(" "),
            link_inputs,
            vec![dest, r_java_path.clone()],
        );

        // Copy this bobje's R.java to every dependency R.java
        for dependency_bobje in bobje.dependencies.values() {
            if detect_android(dependency_bobje) {
                let src_r_java = format!(
                    "{}/src-gen/{}/R.java",
                    dependency_bobje.out_dir(),
                    dependency_bobje
                        .manifest
                        .package
                        .id
                        .as_ref()
                        .expect("Should be some")
                        .replace('.', "/")
                );
                executor.add_task_cmd(
                    format!(
                        "cp {} {} && sed -i{} 's/package {};/package {};/g' {}",
                        r_java_path,
                        src_r_java,
                        if cfg!(target_os = "macos") { " ''" } else { "" },
                        bobje.manifest.package.id.as_ref().expect("Should be some"),
                        dependency_bobje
                            .manifest
                            .package
                            .id
                            .as_ref()
                            .expect("Should be some"),
                        src_r_java
                    ),
                    vec![r_java_path.clone()],
                    vec![src_r_java],
                );
            }
        }
    }
}

// MARK: Link classpath
pub(crate) fn link_android_classpath(bobje: &mut Bobje) {
    let vars = AndroidVars::new(bobje);
    bobje.manifest.build.classpath.push(vars.platform_jar);
}

// MARK: Android classes.dex
pub(crate) fn generate_android_dex_tasks(bobje: &Bobje, executor: &mut Executor) {
    let vars = AndroidVars::new(bobje);
    let classes_dir = format!("{}/classes", bobje.out_dir());
    let modules = find_modules(bobje);

    let mut inputs = modules
        .iter()
        .map(|module| format!("{}/{}", classes_dir, module.name.replace('.', "/")))
        .collect::<Vec<_>>();
    for dependency_bobje in bobje.dependencies.values() {
        if dependency_bobje.r#type == crate::BobjeType::ExternalJar {
            let jar = dependency_bobje.jar.as_ref().expect("Should be some");
            inputs.push(format!("{}/{}", classes_dir, jar.package.replace('.', "/")));
        }
    }

    // Compile classes.dex with r8 task
    if bobje.profile == Profile::Release {
        // Write proguard config file
        let mut proguard_config = String::new();
        for module in &modules {
            for source_file in &module.source_files {
                if source_file.contains("Activity") {
                    proguard_config.push_str(&format!(
                        "-keep public class {} {{ protected void onCreate(android.os.Bundle); }}\n",
                        get_class_name(source_file)
                    ));
                }
            }
        }
        if let Some(android) = bobje.manifest.package.metadata.android.as_ref() {
            for keep in &android.proguard_keep {
                proguard_config.push_str(&format!("-keep {}\n", keep));
            }
        }
        let proguard_config_path = format!("{}/proguard.cfg", bobje.out_dir());
        write_file_when_different(&proguard_config_path, &proguard_config)
            .expect("Can't write proguard.cfg");

        // Add r8 task
        let r8_command = [
            format!("{}/r8", vars.command_line_tools_path),
            "--release".to_string(),
            "--dex".to_string(),
            format!("--min-api {}", vars.android_metadata.min_sdk_version),
            format!("--lib {}", vars.platform_jar),
            format!("--output {}", bobje.out_dir()),
            "--pg-compat".to_string(),
            format!("--pg-conf {}", proguard_config_path),
            format!(
                "$(find {} -name '*.class' | grep -v 'META-INF')",
                &classes_dir
            ),
        ];
        executor.add_task_cmd(
            format!(
                "{} && cd {} && zip {}-unaligned.apk classes.dex > /dev/null",
                r8_command.join(" "),
                bobje.out_dir(),
                bobje.name
            ),
            inputs,
            vec![format!("{}/classes.dex", bobje.out_dir())],
        );
    }
    // Compile classes.dex with d8 task
    else {
        let d8_command = [
            format!("{}/d8", vars.build_tools_path),
            "--debug".to_string(),
            format!("--min-api {}", vars.android_metadata.min_sdk_version),
            format!("--lib {}", vars.platform_jar),
            format!("--output {}", bobje.out_dir()),
            format!(
                "$(find {} -name '*.class' | grep -v 'META-INF')",
                &classes_dir
            ),
        ];
        executor.add_task_cmd(
            format!(
                "{} && cd {} && zip {}-unaligned.apk classes.dex > /dev/null",
                d8_command.join(" "),
                bobje.out_dir(),
                bobje.name
            ),
            inputs,
            vec![format!("{}/classes.dex", bobje.out_dir())],
        );
    }
}

// MARK: Android APK
pub(crate) fn generate_android_final_apk_tasks(bobje: &Bobje, executor: &mut Executor) {
    let vars = AndroidVars::new(bobje);

    // Generate dummy keystore if it doesn't exist
    let target_keystore = format!(
        "{}/{}",
        bobje.target_dir, vars.android_metadata.keystore_file
    );
    if fs::metadata(&vars.android_metadata.keystore_file).is_err()
        && fs::metadata(&target_keystore).is_err()
    {
        let mut cmd = Command::new("sh");
        let mut cmd_str = format!(
            "keytool -genkey -keystore {} -storetype JKS -keyalg RSA -keysize 4096 -validity 7120",
            &target_keystore
        );
        if !vars.android_metadata.key_alias.is_empty() {
            cmd_str.push_str(&format!(" -alias {}", &vars.android_metadata.key_alias));
        }
        cmd_str.push_str(&format!(
            " -storepass {} -keypass {} -dname \"CN=Unknown, OU=Unknown, O=Unknown, L=Unknown, S=Unknown, C=Unknown\"",
            &vars.android_metadata.keystore_password,
            &vars.android_metadata.key_password
        ));
        let status = cmd
            .arg("-c")
            .arg(format!("{} &> /dev/null", cmd_str))
            .status()
            .expect("Failed to execute keytool");
        if !status.success() {
            exit(status.code().unwrap_or(1));
        }
    }

    // zipalign
    let unaligned_apk = format!("{}/{}-unaligned.apk", bobje.out_dir(), bobje.name);
    let unsigned_apk = format!("{}/{}-unsigned.apk", bobje.out_dir(), bobje.name);
    executor.add_task_cmd(
        format!(
            "{}/zipalign -f -p 4 {} {}",
            vars.build_tools_path, unaligned_apk, unsigned_apk
        ),
        vec![
            unaligned_apk.clone(),
            format!("{}/classes.dex", bobje.out_dir()),
        ],
        vec![unsigned_apk.clone()],
    );

    // apksigner
    let signed_apk = format!("{}/{}-{}.apk", bobje.out_dir(), bobje.name, bobje.version);
    let mut apksigner_cmd = format!(
        "{}/apksigner sign --min-sdk-version {} --v4-signing-enabled false --ks {} ",
        vars.build_tools_path,
        vars.android_metadata.min_sdk_version,
        if fs::metadata(&vars.android_metadata.keystore_file).is_ok() {
            vars.android_metadata.keystore_file
        } else {
            target_keystore
        }
    );
    if !vars.android_metadata.key_alias.is_empty() {
        apksigner_cmd.push_str(&format!(
            "--ks-key-alias {} ",
            vars.android_metadata.key_alias
        ));
    }
    apksigner_cmd.push_str(&format!(
        "--ks-pass pass:{} --key-pass pass:{} --in {} --out {}",
        vars.android_metadata.keystore_password,
        vars.android_metadata.key_password,
        unsigned_apk,
        signed_apk
    ));
    executor.add_task_cmd(
        apksigner_cmd,
        vec![unsigned_apk.clone()],
        vec![signed_apk.clone()],
    );
}

pub(crate) fn run_android_apk(bobje: &Bobje) -> ! {
    let vars = AndroidVars::new(bobje);
    let adb_path = format!("{}/adb", vars.platform_tools_path);

    let status = Command::new(&adb_path)
        .arg("install")
        .arg("-r")
        .arg(format!(
            "{}/{}-{}.apk",
            bobje.out_dir(),
            bobje.name,
            bobje.version
        ))
        .status()
        .expect("Failed to execute adb");
    if !status.success() {
        exit(status.code().unwrap_or(1));
    }

    let status = Command::new(&adb_path)
        .arg("shell")
        .arg("am")
        .arg("start")
        .arg("-n")
        .arg(format!(
            "{}/{}",
            vars.id, vars.android_metadata.main_activity
        ))
        .status()
        .expect("Failed to execute adb");
    exit(status.code().unwrap_or(1))
}

// MARK: Utils
fn parse_version_to_code(version: &str) -> u32 {
    let version_parts: Vec<u32> = version
        .split('.')
        .map(|part| part.parse().expect("Version must be in semver format"))
        .collect();
    if version_parts.len() != 3 {
        panic!("Version must be in semver format");
    }
    version_parts[0] * 1_000_000 + version_parts[1] * 1_000 + version_parts[2]
}
