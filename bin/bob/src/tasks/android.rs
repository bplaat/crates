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
use crate::tasks::java::find_modules;
use crate::utils::index_files;
use crate::{Bobje, Profile};

// MARK: Android vars
struct AndroidVars {
    identifier: String,
    android_metadata: AndroidMetadata,
    platform_jar: String,
    build_tools_path: String,
    platform_tools_path: String,
}

impl AndroidVars {
    fn new(bobje: &Bobje) -> Self {
        let identifier = bobje
            .manifest
            .package
            .identifier
            .as_ref()
            .unwrap_or_else(|| {
                eprintln!("Manifest package identifier is required");
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
        let build_tools_path = format!(
            "{}/build-tools/{}.0.0",
            android_home, android_metadata.target_sdk_version
        );
        let platform_tools_path = format!("{}/platform-tools", android_home);
        Self {
            identifier: identifier.clone(),
            android_metadata: android_metadata.clone(),
            platform_jar,
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
    let compiled_res_dir = format!(
        "{}/{}/res/{}",
        bobje.target_dir,
        bobje.profile,
        vars.identifier.replace('.', "/")
    );
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
        "{}/{}/src-gen/{}/R.java",
        bobje.target_dir,
        bobje.profile,
        vars.identifier.replace('.', "/")
    );
    bobje.source_files.push(r_java_path.clone());

    if bobje.r#type == crate::BobjeType::Binary {
        let dest = format!(
            "{}/{}/{}-unaligned.apk",
            bobje.target_dir, bobje.profile, bobje.manifest.package.name
        );

        let mut link_inputs = Vec::new();
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
                    "{}/{}/res/{}",
                    bobje.target_dir,
                    bobje.profile,
                    bobje
                        .manifest
                        .package
                        .identifier
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
            format!("{}/{}/src-gen", bobje.target_dir, bobje.profile),
            "--version-name".to_string(),
            bobje.manifest.package.version.clone(),
            "--version-code".to_string(),
            parse_version_to_code(&bobje.manifest.package.version).to_string(),
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
                    "{}/{}/src-gen/{}/R.java",
                    dependency_bobje.target_dir,
                    dependency_bobje.profile,
                    dependency_bobje
                        .manifest
                        .package
                        .identifier
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
                        bobje
                            .manifest
                            .package
                            .identifier
                            .as_ref()
                            .expect("Should be some"),
                        dependency_bobje
                            .manifest
                            .package
                            .identifier
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
    let classes_dir = format!("{}/{}/classes", bobje.target_dir, bobje.profile);
    let modules = find_modules(bobje);

    // Compile classes.dex with d8 task
    let mut d8_command = vec![
        format!("{}/d8", vars.build_tools_path),
        if bobje.profile == Profile::Release {
            "--release"
        } else {
            "--debug"
        }
        .to_string(),
        format!("--lib {}", vars.platform_jar),
        format!("--min-api {}", vars.android_metadata.min_sdk_version),
        format!("--output {}/{}/", bobje.target_dir, bobje.profile),
    ];
    for module in &modules {
        d8_command.push(format!(
            "{}/{}/*.class",
            classes_dir,
            module.name.replace('.', "/")
        ));
    }
    executor.add_task_cmd(
        format!(
            "{} && cd {}/{} && zip {}-unaligned.apk classes.dex",
            d8_command.join(" "),
            bobje.target_dir,
            bobje.profile,
            bobje.manifest.package.name
        ),
        modules
            .iter()
            .map(|module| format!("{}/{}", classes_dir, module.name.replace('.', "/")))
            .collect(),
        vec![format!(
            "{}/{}/classes.dex",
            bobje.target_dir, bobje.profile
        )],
    );
}

// MARK: Android APK
pub(crate) fn generate_android_final_apk_tasks(bobje: &Bobje, executor: &mut Executor) {
    let mut vars = AndroidVars::new(bobje);

    // Generate dummy keystore if it doesn't exist
    let target_keystore = format!(
        "{}/{}",
        bobje.target_dir, vars.android_metadata.keystore_file
    );
    if fs::metadata(&vars.android_metadata.keystore_file).is_err()
        && fs::metadata(&target_keystore).is_err()
    {
        println!("Android signing keystore not found, generating dummy one...");
        let mut cmd = Command::new("keytool");
        vars.android_metadata.keystore_file = target_keystore;
        cmd.arg("-genkey")
            .arg("-keystore")
            .arg(&vars.android_metadata.keystore_file)
            .arg("-storetype")
            .arg("JKS")
            .arg("-keyalg")
            .arg("RSA")
            .arg("-keysize")
            .arg("4096")
            .arg("-validity")
            .arg("7120");
        if !vars.android_metadata.key_alias.is_empty() {
            cmd.arg("-alias").arg(&vars.android_metadata.key_alias);
        }
        let status = cmd
            .arg("-storepass")
            .arg(&vars.android_metadata.keystore_password)
            .arg("-keypass")
            .arg(&vars.android_metadata.key_password)
            .arg("-dname")
            .arg("CN=Unknown, OU=Unknown, O=Unknown, L=Unknown, S=Unknown, C=Unknown")
            .status()
            .expect("Failed to execute keytool");
        if !status.success() {
            exit(status.code().unwrap_or(1));
        }
    }

    // zipalign
    let unaligned_apk = format!(
        "{}/{}/{}-unaligned.apk",
        bobje.target_dir, bobje.profile, bobje.manifest.package.name
    );
    let unsigned_apk = format!(
        "{}/{}/{}-unsigned.apk",
        bobje.target_dir, bobje.profile, bobje.manifest.package.name
    );
    executor.add_task_cmd(
        format!(
            "{}/zipalign -f -p 4 {} {}",
            vars.build_tools_path, unaligned_apk, unsigned_apk
        ),
        vec![
            unaligned_apk.clone(),
            format!("{}/{}/classes.dex", bobje.target_dir, bobje.profile),
        ],
        vec![unsigned_apk.clone()],
    );

    // apksigner
    let signed_apk = format!(
        "{}/{}/{}-{}.apk",
        bobje.target_dir,
        bobje.profile,
        bobje.manifest.package.name,
        bobje.manifest.package.version
    );
    let mut apksigner_cmd = format!(
        "{}/apksigner sign --min-sdk-version {} --v4-signing-enabled false --ks {} ",
        vars.build_tools_path,
        vars.android_metadata.min_sdk_version,
        vars.android_metadata.keystore_file
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

pub(crate) fn run_android_apk(bobje: &Bobje) {
    let vars = AndroidVars::new(bobje);
    let adb_path = format!("{}/adb", vars.platform_tools_path);

    let status = Command::new(&adb_path)
        .arg("install")
        .arg("-r")
        .arg(format!(
            "{}/{}/{}-{}.apk",
            bobje.target_dir,
            bobje.profile,
            bobje.manifest.package.name,
            bobje.manifest.package.version
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
            vars.identifier, vars.android_metadata.main_activity
        ))
        .status()
        .expect("Failed to execute adb");
    exit(status.code().unwrap_or(1));
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
