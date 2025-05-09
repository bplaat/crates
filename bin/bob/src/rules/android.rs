/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::io::Write;
use std::process::{Command, exit};
use std::{env, fs};

use crate::rules::java;
use crate::{Profile, Project};

// FIXME: Add dummy key generation: keytool -genkey -keystore keystore.jks -storetype JKS -keyalg RSA -keysize 4096 -validity 7120 -alias android -storepass android -keypass android

// MARK: Rules
pub(crate) fn generate_android_vars(f: &mut dyn Write, project: &Project) {
    _ = writeln!(f, "\n# Android variables");

    let android_metadata = project
        .manifest
        .package
        .metadata
        .as_ref()
        .and_then(|m| m.android.clone())
        .unwrap_or_default();

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
    // FIXME: Compile every resources in separate build steps
    _ = writeln!(
        f,
        "rule aapt2_compile\n  command = mkdir -p $target_dir/$profile/res && $build_tools_path/aapt2 compile --no-crunch --dir $in -o $out\n  description = Compiling $in\n"
    );
    _ = writeln!(
        f,
        "rule aapt2_link\n  command = $build_tools_path/aapt2 link $target_dir/$profile/res/*.flat {}--manifest $manifest_dir/AndroidManifest.xml \
        --java $target_dir/$profile/src-gen --version-name $version --version-code {} --min-sdk-version $min_sdk_version --target-sdk-version $target_sdk_version \
        -I $platform_jar -o $target_dir/$profile/${{name}}-unaligned.apk\n  description = Linking $in\n",
        if fs::metadata(format!("{}/assets", project.manifest_dir)).is_ok() {
            "-A $manifest_dir/assets "
        } else {
            ""
        },
        parse_version_to_code(&project.manifest.package.version)
    );
    _ = writeln!(
        f,
        "build $target_dir/$profile/res: aapt2_compile $manifest_dir/res",
    );

    let r_java_path = format!("$source_gen_dir/{}/R.java", identifier.replace('.', "/"));
    _ = writeln!(
        f,
        "build $target_dir/$profile/${{name}}-unaligned.apk.0 {}: aapt2_link $target_dir/$profile/res",
        r_java_path
    );
    project.source_files.push(r_java_path);
}

pub(crate) fn generate_android_dex(f: &mut dyn Write, project: &Project) {
    let modules = java::find_modules(&project.source_files);

    _ = writeln!(f, "\n# Compile Android dex");
    _ = writeln!(
        f,
        "rule d8\n  command = $build_tools_path/d8 {} --lib $platform_jar --min-api $min_sdk_version --output $target_dir/$profile {}\n  description = Compiling $out\n",
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
        .as_ref()
        .and_then(|m| m.android.clone())
        .unwrap_or_default();

    _ = writeln!(f, "\n# Build Android apk");
    _ = writeln!(
        f,
        "rule zipcp\n  command = zip -j $out $in > /dev/null\n  description = Copying $in"
    );
    _ = writeln!(
        f,
        "rule zipalign\n  command = $build_tools_path/zipalign -f -p 4 $in $out\n  description = Aligning $out"
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
        "build $target_dir/$profile/${{name}}-unaligned.apk: zipcp $target_dir/$profile/classes.dex | $target_dir/$profile/${{name}}-unaligned.apk.0"
    );
    _ = writeln!(
        f,
        "build $target_dir/$profile/${{name}}-unsigned.apk: zipalign $target_dir/$profile/${{name}}-unaligned.apk"
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
        .as_ref()
        .and_then(|m| m.android.clone())
        .unwrap_or_default();

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
        return;
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
