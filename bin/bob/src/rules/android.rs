/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::io::Write;
use std::process::{Command, exit};
use std::{env, fs};

use crate::rules::java;
use crate::{Profile, Project, index_files};

// MARK: Rules
pub(crate) fn generate_android_vars(f: &mut dyn Write, project: &Project) {
    let android_metadata = project
        .manifest
        .package
        .metadata
        .android
        .as_ref()
        .unwrap_or_else(|| {
            eprintln!("Android metadata is required");
            exit(1);
        });

    _ = writeln!(f, "\n# Android variables");
    _ = writeln!(f, "min_sdk_version = {}", android_metadata.min_sdk_version);
    _ = writeln!(
        f,
        "target_sdk_version = {}",
        android_metadata.target_sdk_version
    );

    _ = writeln!(
        f,
        "android_home = {}",
        env::var("ANDROID_HOME").expect("$ANDROID_HOME env var must be set")
    );
    _ = writeln!(
        f,
        "platform_jar = $android_home/platforms/android-$target_sdk_version/android.jar"
    );
    _ = writeln!(f, "classpath = $classpath:$platform_jar");
    _ = writeln!(
        f,
        "build_tools_path = $android_home/build-tools/$target_sdk_version.0.0"
    );
    _ = writeln!(f, "platform_tools_path = $android_home/platform-tools");
}

pub(crate) fn generate_android_res(f: &mut dyn Write, project: &mut Project) {
    let identifier = project
        .manifest
        .package
        .identifier
        .as_ref()
        .unwrap_or_else(|| {
            eprintln!("Identifier is required");
            exit(1);
        });

    _ = writeln!(f, "\n# Compile Android resources");
    _ = writeln!(
        f,
        "rule aapt2_compile\n  command = $build_tools_path/aapt2 compile --no-crunch $in -o $target_dir/$profile/res\n  description = Compiling $in\n"
    );
    let assets_dir = format!("{}/assets", project.manifest_dir);
    _ = writeln!(
        f,
        "rule aapt2_link\n  command = $build_tools_path/aapt2 link $target_dir/$profile/res/*.flat {} --manifest $manifest_dir/AndroidManifest.xml \
        --java $target_dir/$profile/src-gen --version-name $version --version-code {} --min-sdk-version $min_sdk_version --target-sdk-version $target_sdk_version \
        -I $platform_jar -o $target_dir/$profile/${{name}}-unaligned.apk\n  description = Linking $in\n",
        if fs::metadata(&assets_dir).is_ok() {
            "-A $manifest_dir/assets"
        } else {
            ""
        },
        parse_version_to_code(&project.manifest.package.version)
    );

    let mut compiled_res_files = Vec::new();
    let res_dir = format!("{}/res/", project.manifest_dir);
    for res_file in index_files(&res_dir) {
        let mut compiled_res_file = format!(
            "$target_dir/$profile/res/{}.flat",
            res_file.trim_start_matches(&res_dir).replace('/', "_")
        );
        if compiled_res_file.contains("/values") {
            compiled_res_file = compiled_res_file.replace(".xml", ".arsc");
        }
        _ = writeln!(
            f,
            "build {}: aapt2_compile {}",
            &compiled_res_file,
            res_file.replace(&res_dir, "$manifest_dir/res/")
        );
        compiled_res_files.push(compiled_res_file);
    }

    let r_java_path = format!("$source_gen_dir/{}/R.java", identifier.replace('.', "/"));
    _ = writeln!(
        f,
        "build $target_dir/$profile/${{name}}-unaligned.apk {}: aapt2_link {} {}",
        r_java_path,
        compiled_res_files.join(" "),
        if fs::metadata(&assets_dir).is_ok() {
            index_files(&assets_dir)
                .iter()
                .map(|p| p.replace(&assets_dir, "$manifest_dir/assets"))
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            "".to_string()
        }
    );
    project.source_files.push(r_java_path);
}

pub(crate) fn generate_android_dex(f: &mut dyn Write, project: &Project) {
    let modules = java::find_modules(&project.source_files);

    _ = writeln!(f, "\n# Compile Android dex");
    _ = writeln!(
        f,
        "rule d8\n  command = $build_tools_path/d8 {} --lib $platform_jar --min-api $min_sdk_version --output $target_dir/$profile {} && zip -j $target_dir/$profile/${{name}}-unaligned.apk $target_dir/$profile/classes.dex > /dev/null\n  description = Compiling $out\n",
        if project.profile == Profile::Release {
            "--release"
        } else {
            "--debug"
        },
        modules
            .keys()
            .map(|module| format!("$classes_dir/{}/*.class", module.replace('.', "/")))
            .collect::<Vec<_>>()
            .join(" ")
    );
    _ = writeln!(
        f,
        "build $target_dir/$profile/classes.dex: d8 {}",
        modules
            .keys()
            .map(|module| format!("$classes_dir/{}/.stamp", module.replace('.', "/")))
            .collect::<Vec<_>>()
            .join(" ")
    );
}

pub(crate) fn generate_android_apk(f: &mut dyn Write, project: &Project) {
    let android_metadata = project
        .manifest
        .package
        .metadata
        .android
        .as_ref()
        .unwrap_or_else(|| {
            eprintln!("Android metadata is required");
            exit(1);
        });

    // Generate dummy keystore if it doesn't exist
    let keystore_path = format!(
        "{}/{}",
        project.manifest_dir, android_metadata.keystore_file
    );
    if fs::metadata(&keystore_path).is_err() {
        println!("Android signing keystore not found, generating dummy one...");
        let status = Command::new("keytool")
            .arg("-genkey")
            .arg("-keystore")
            .arg(&keystore_path)
            .arg("-storetype")
            .arg("JKS")
            .arg("-keyalg")
            .arg("RSA")
            .arg("-keysize")
            .arg("4096")
            .arg("-validity")
            .arg("7120")
            .arg("-alias")
            .arg("android")
            .arg("-storepass")
            .arg("android")
            .arg("-keypass")
            .arg("android")
            .arg("-dname")
            .arg("CN=Unknown, OU=Unknown, O=Unknown, L=Unknown, S=Unknown, C=Unknown")
            .status()
            .expect("Failed to execute keytool");
        if !status.success() {
            exit(status.code().unwrap_or(1));
        }
    }

    _ = writeln!(f, "\n# Build Android apk");
    _ = writeln!(
        f,
        "rule zipalign\n  command = $build_tools_path/zipalign -f -p 4 $in $out\n  description = Aligning $out\n"
    );
    _ = writeln!(
        f,
        "rule apksigner\n  command = $build_tools_path/apksigner sign --v4-signing-enabled false --ks $manifest_dir/{} \
        {}--ks-pass pass:{} --ks-pass pass:{} --in $in --out $out\n  description = Signing $in\n",
        android_metadata.keystore_file,
        if !android_metadata.key_alias.is_empty() {
            format!("--ks-key-alias {} ", android_metadata.key_alias)
        } else {
            "".to_string()
        },
        android_metadata.keystore_password,
        android_metadata.key_password
    );

    _ = writeln!(
        f,
        "build $target_dir/$profile/${{name}}-unsigned.apk: zipalign $target_dir/$profile/${{name}}-unaligned.apk | $target_dir/$profile/classes.dex"
    );
    _ = writeln!(
        f,
        "build $target_dir/$profile/${{name}}-$version.apk: apksigner $target_dir/$profile/${{name}}-unsigned.apk"
    );
}

// MARK: Runners
pub(crate) fn run_android_apk(project: &Project) {
    let identifier = project
        .manifest
        .package
        .identifier
        .as_ref()
        .unwrap_or_else(|| {
            eprintln!("Identifier is required");
            exit(1);
        });
    let android_metadata = project
        .manifest
        .package
        .metadata
        .android
        .as_ref()
        .unwrap_or_else(|| {
            eprintln!("Android metadata is required");
            exit(1);
        });

    let adb_path = format!(
        "{}/platform-tools/adb",
        env::var("ANDROID_HOME").expect("$ANDROID_HOME env var must be set")
    );
    let status = Command::new(&adb_path)
        .arg("install")
        .arg("-r")
        .arg(format!(
            "{}/target/{}/{}-{}.apk",
            project.manifest_dir,
            project.profile,
            project.manifest.package.name,
            project.manifest.package.version
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
        .arg(format!("{}/{}", identifier, android_metadata.main_activity))
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
    version_parts[0] * 10_000 + version_parts[1] * 100 + version_parts[2]
}
