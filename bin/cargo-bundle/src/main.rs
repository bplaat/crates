/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = "A simple Cargo plugin the builds macOS app bundles"]
#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;
use std::process::{Command, exit};

use copy_dir::copy_dir;

use crate::manifest::Manifest;

mod args;
mod manifest;

fn read_manifest(path: &str) -> Manifest {
    let manifest_path = format!("{path}/Cargo.toml");
    basic_toml::from_str(&fs::read_to_string(&manifest_path).unwrap_or_else(|err| {
        eprintln!("Can't read {manifest_path} file: {err}");
        exit(1);
    }))
    .unwrap_or_else(|err| {
        eprintln!("Can't parse {manifest_path} file: {err}");
        exit(1);
    })
}

fn generate_resources(path: &str, target_dir: &str, bundle: &manifest::BundleMetadata) {
    // Generate resources for macOS bundle
    fs::create_dir_all(target_dir).expect("Failed to create target directory");

    // Compile iconset to icns if needed
    if let Some(iconset) = &bundle.iconset {
        Command::new("iconutil")
            .args([
                "-c",
                "icns",
                &format!("{path}/{iconset}"),
                "-o",
                &format!("{target_dir}/icon.icns"),
            ])
            .output()
            .expect("Failed to create icon.icns");
    }

    // Copy icns if provided
    if let Some(icns) = &bundle.icns {
        fs::copy(format!("{path}/{icns}"), format!("{target_dir}/icon.icns"))
            .expect("Failed to copy icon.icns");
    }

    // Create Info.plist
    let mut plist = vec![
        ("CFBundlePackageType", "APPL".to_string()),
        ("CFBundleName", bundle.name.clone()),
        ("CFBundleDisplayName", bundle.name.clone()),
        ("CFBundleIdentifier", bundle.identifier.clone()),
        ("CFBundleVersion", env!("CARGO_PKG_VERSION").to_string()),
        (
            "CFBundleShortVersionString",
            env!("CARGO_PKG_VERSION").to_string(),
        ),
        ("CFBundleExecutable", bundle.name.clone()),
        ("LSMinimumSystemVersion", "11.0".to_string()),
    ];
    if bundle.iconset.is_some() || bundle.icns.is_some() {
        plist.push(("CFBundleIconFile", "icon".to_string()));
    }
    if let Some(copyright) = &bundle.copyright {
        plist.push(("NSHumanReadableCopyright", copyright.clone()));
    }

    // Write Info.plist using the Vec
    let mut info_plist = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
"#,
    );
    for (key, value) in &plist {
        info_plist.push_str(&format!("\t<key>{key}</key>\n\t<string>{value}</string>\n"));
    }
    info_plist.push_str("</dict>\n</plist>\n");

    fs::write(format!("{target_dir}/Info.plist"), info_plist).expect("Failed to write Info.plist");
}

fn compile_lipo(
    path: &str,
    target_dir: &str,
    package: &manifest::Package,
    bundle: &manifest::BundleMetadata,
) {
    for target in ["x86_64-apple-darwin", "aarch64-apple-darwin"] {
        let status = Command::new("cargo")
            .args([
                "build",
                "--release",
                "--manifest-path",
                &format!("{path}/Cargo.toml"),
                "--target",
                target,
            ])
            .status()
            .expect("Failed to run cargo build");
        assert!(status.success(), "cargo build failed for {target}");
    }
    let lipo_status = Command::new("lipo")
        .args([
            "-create",
            &format!("target/x86_64-apple-darwin/release/{}", package.name),
            &format!("target/aarch64-apple-darwin/release/{}", package.name),
            "-output",
            &format!("{target_dir}/{}", bundle.name),
        ])
        .status()
        .expect("Failed to run lipo");
    assert!(lipo_status.success(), "lipo failed");
}

fn create_bundle(path: &str, target_dir: &str, bundle: &manifest::BundleMetadata) {
    // Create bundle folder structure
    let bundle_dir = format!("{target_dir}/{}.app/Contents", bundle.name);
    fs::create_dir_all(format!("{bundle_dir}/MacOS")).expect("Can't create directory");
    fs::create_dir_all(format!("{bundle_dir}/Resources")).expect("Can't create directory");

    // Copy files
    fs::copy(
        format!("{target_dir}/icon.icns"),
        format!("{bundle_dir}/Resources/icon.icns"),
    )
    .expect("Failed to copy icon.icns");

    if let Some(resources_dir) = &bundle.resources_dir {
        copy_dir(
            format!("{path}/{resources_dir}"),
            format!("{bundle_dir}/Resources"),
        )
        .expect("Failed to copy resources directory");
    }

    fs::copy(
        format!("{target_dir}/{}", bundle.name),
        format!("{bundle_dir}/MacOS/{}", bundle.name),
    )
    .expect("Failed to copy executable");

    fs::copy(
        format!("{target_dir}/Info.plist"),
        format!("{bundle_dir}/Info.plist"),
    )
    .expect("Failed to copy Info.plist");
}

fn create_zip(target_dir: &str, bundle: &manifest::BundleMetadata) {
    let zip_name = format!("{}/{}.zip", target_dir, bundle.name);
    if Path::new(&zip_name).exists() {
        fs::remove_file(&zip_name).expect("Failed to remove existing zip");
    }
    let status = Command::new("zip")
        .args([
            "-r",
            &format!("{}.zip", bundle.name),
            &format!("{}.app", bundle.name),
        ])
        .current_dir(target_dir)
        .status()
        .expect("Failed to run zip");
    assert!(status.success(), "zip command failed");
}

#[cfg(target_os = "macos")]
fn create_dmg(target_dir: &str, bundle: &manifest::BundleMetadata) {
    let disk_dir = format!("{target_dir}/disk");
    let app_name = format!("{}.app", bundle.name);

    // Create disk directory
    fs::create_dir_all(&disk_dir).expect("Failed to create disk directory");

    // Copy .app bundle into disk directory
    let src_app = format!("{target_dir}/{app_name}");
    let dst_app = format!("{disk_dir}/{app_name}");
    copy_dir(&src_app, &dst_app).expect("Failed to copy .app bundle to disk");

    // Create Applications symlink
    let applications_link = format!("{disk_dir}/Applications");
    if Path::new(&applications_link).exists() {
        fs::remove_file(&applications_link)
            .expect("Failed to remove existing Applications symlink");
    }
    std::os::unix::fs::symlink("/Applications", &applications_link)
        .expect("Failed to create Applications symlink");

    // Create DMG using hdiutil
    let dmg_name = format!("{}/{}.dmg", target_dir, bundle.name);
    if Path::new(&dmg_name).exists() {
        fs::remove_file(&dmg_name).expect("Failed to remove existing DMG");
    }
    let status = Command::new("hdiutil")
        .args([
            "create",
            "-srcfolder",
            &disk_dir,
            "-volname",
            &bundle.name,
            "-fs",
            "HFS+",
            "-format",
            "UDZO",
            &dmg_name,
        ])
        .status()
        .expect("Failed to run hdiutil");
    assert!(status.success(), "hdiutil failed to create DMG");
}

fn main() {
    let args = args::parse_args();

    // Subcommands
    if args.help {
        args::help();
        return;
    }
    if args.version {
        println!("cargo-bundle {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Read Cargo.toml manifest
    let manifest = read_manifest(&args.path);
    let bundle = &manifest.package.metadata.bundle;

    // Generate resource
    let target_dir = format!("target/bundle/{}", manifest.package.name);
    generate_resources(&args.path, &target_dir, bundle);

    // Compile lipo executable
    compile_lipo(&args.path, &target_dir, &manifest.package, bundle);

    // Create bundle folder structure
    create_bundle(&args.path, &target_dir, bundle);

    // Create zip
    create_zip(&target_dir, bundle);

    // Create dmg installer
    #[cfg(target_os = "macos")]
    {
        create_dmg(&target_dir, bundle);
    }
}
