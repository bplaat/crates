/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs;
use std::process::{Command, exit};

use crate::manifest::{AppexMetadata, BundleMetadata, Manifest};
use crate::support::{assert_file_name, compile_universal, remove_empty_directory};

pub(crate) fn build_all(path: &str, target_dir: &str, manifest: &Manifest) {
    let bundle = &manifest.package.metadata.bundle;
    let Some(extensions) = &bundle.appex else {
        return;
    };
    for extension in extensions {
        assert_file_name(&extension.name, "app extension name");
        write_info_plist(path, target_dir, manifest, extension);
        let binary = compile(path, target_dir, bundle, extension);
        let appex = create(target_dir, bundle, extension, &binary);
        sign(path, extension, &appex);
    }
}

fn write_info_plist(path: &str, target_dir: &str, manifest: &Manifest, extension: &AppexMetadata) {
    let mut info = plist::Dictionary::new();
    info.insert("CFBundleInfoDictionaryVersion".into(), "6.0".into());
    info.insert("CFBundlePackageType".into(), "XPC!".into());
    info.insert("CFBundleName".into(), extension.name.clone().into());
    info.insert("CFBundleDisplayName".into(), extension.name.clone().into());
    info.insert(
        "CFBundleIdentifier".into(),
        extension.identifier.clone().into(),
    );
    info.insert(
        "CFBundleVersion".into(),
        manifest.package.version.clone().into(),
    );
    info.insert(
        "CFBundleShortVersionString".into(),
        manifest.package.version.clone().into(),
    );
    info.insert("CFBundleExecutable".into(), extension.name.clone().into());
    info.insert(
        "LSMinimumSystemVersion".into(),
        extension
            .minimal_os_version
            .clone()
            .unwrap_or_else(|| String::from("12.0"))
            .into(),
    );

    let custom_info = format!("{path}/{}", extension.info_plist);
    let plist::Value::Dictionary(custom) =
        plist::Value::from_file(&custom_info).unwrap_or_else(|error| {
            eprintln!("Invalid app extension Info.plist {custom_info}: {error}");
            exit(1);
        })
    else {
        eprintln!("Invalid app extension Info.plist: root value must be a dictionary");
        exit(1);
    };
    info.extend(custom);

    let output_dir = metadata_dir(target_dir, extension);
    fs::create_dir_all(&output_dir).expect("Failed to create app extension metadata directory");
    plist::Value::Dictionary(info)
        .to_file_binary(format!("{output_dir}/Info.plist"))
        .expect("Failed to write app extension Info.plist");
}

fn compile(
    path: &str,
    target_dir: &str,
    bundle: &BundleMetadata,
    extension: &AppexMetadata,
) -> String {
    let package = format!("{path}/{}", extension.package);
    let output = format!("{}/{}", metadata_dir(target_dir, extension), extension.name);
    if bundle.lipo.unwrap_or(true) {
        compile_universal(&package, &extension.binary, &output);
    } else {
        let status = Command::new("cargo")
            .args([
                "build",
                "--release",
                "--manifest-path",
                &format!("{package}/Cargo.toml"),
            ])
            .status()
            .expect("Failed to build app extension");
        assert!(status.success(), "cargo build failed for app extension");
        fs::copy(format!("target/release/{}", extension.binary), &output)
            .expect("Failed to copy app extension executable");
    }
    output
}

fn create(
    target_dir: &str,
    bundle: &BundleMetadata,
    extension: &AppexMetadata,
    binary: &str,
) -> String {
    let contents = contents_dir(target_dir, bundle, extension);
    fs::create_dir_all(format!("{contents}/MacOS"))
        .expect("Failed to create app extension executable directory");
    remove_empty_directory(&format!("{contents}/Resources"));
    fs::copy(binary, format!("{contents}/MacOS/{}", extension.name))
        .expect("Failed to copy app extension executable");
    fs::copy(
        format!("{}/Info.plist", metadata_dir(target_dir, extension)),
        format!("{contents}/Info.plist"),
    )
    .expect("Failed to copy app extension Info.plist");
    contents
        .strip_suffix("/Contents")
        .expect("app extension contents path has suffix")
        .to_string()
}

fn sign(path: &str, extension: &AppexMetadata, appex: &str) {
    let mut command = Command::new("codesign");
    command.args(["--force", "--sign", "-", "--options", "runtime"]);
    if let Some(entitlements) = &extension.entitlements {
        command.args(["--entitlements", &format!("{path}/{entitlements}")]);
    }
    let status = command
        .arg(appex)
        .status()
        .expect("Failed to run codesign for app extension");
    assert!(status.success(), "codesign failed for app extension");
}

fn metadata_dir(target_dir: &str, extension: &AppexMetadata) -> String {
    format!("{target_dir}/appex/{}", extension.name)
}

fn contents_dir(target_dir: &str, bundle: &BundleMetadata, extension: &AppexMetadata) -> String {
    format!(
        "{target_dir}/{}.app/Contents/PlugIns/{}.appex/Contents",
        bundle.name, extension.name
    )
}
