/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple Cargo plugin the builds macOS app bundles

use std::fs;
use std::path::Path;
use std::process::{Command, exit};

use copy_dir::copy_dir;

use crate::manifest::{BundleMetadata, Manifest, Package, PluginMetadata};

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

fn generate_resources(path: &str, target_dir: &str, manifest: &Manifest) {
    let bundle = &manifest.package.metadata.bundle;

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

    // Compile .icon dir to Assets.car and icon.icns if needed
    if let Some(icon) = &bundle.icon {
        // actool resolves paths relative to the icon file, not the process CWD, so use absolute paths
        let icon_abs = fs::canonicalize(format!("{path}/{icon}"))
            .expect("Failed to resolve icon path")
            .to_string_lossy()
            .into_owned();
        let target_abs = fs::canonicalize(target_dir)
            .expect("Failed to resolve target dir")
            .to_string_lossy()
            .into_owned();
        let partial_plist_abs = format!("{target_abs}/partial.plist");
        let actool_output = Command::new("actool")
            .args([
                &icon_abs,
                "--compile",
                &target_abs,
                "--platform",
                "macosx",
                "--minimum-deployment-target",
                bundle.minimal_os_version.as_deref().unwrap_or("11.0"),
                "--target-device",
                "mac",
                "--app-icon",
                "icon",
                "--include-all-app-icons",
                "--output-partial-info-plist",
                &partial_plist_abs,
            ])
            .output()
            .expect("Failed to run actool");
        if !actool_output.status.success() {
            let stdout = String::from_utf8_lossy(&actool_output.stdout);
            let stderr = String::from_utf8_lossy(&actool_output.stderr);
            eprintln!("actool failed:\n{stdout}{stderr}");
            exit(1);
        }
    }

    // Copy icns if provided
    if let Some(icns) = &bundle.icns {
        fs::copy(format!("{path}/{icns}"), format!("{target_dir}/icon.icns"))
            .expect("Failed to copy icon.icns");
    }

    // Create Info.plist as binary plist
    let mut dict = plist::Dictionary::new();
    dict.insert("CFBundleInfoDictionaryVersion".into(), "6.0".into());
    dict.insert("CFBundlePackageType".into(), "APPL".into());
    dict.insert("CFBundleName".into(), bundle.name.clone().into());
    dict.insert("CFBundleDisplayName".into(), bundle.name.clone().into());
    dict.insert(
        "CFBundleIdentifier".into(),
        bundle.identifier.clone().into(),
    );
    dict.insert(
        "CFBundleVersion".into(),
        manifest.package.version.clone().into(),
    );
    dict.insert(
        "CFBundleShortVersionString".into(),
        manifest.package.version.clone().into(),
    );
    dict.insert("CFBundleExecutable".into(), bundle.name.clone().into());
    dict.insert(
        "LSMinimumSystemVersion".into(),
        bundle
            .minimal_os_version
            .clone()
            .unwrap_or_else(|| "11.0".to_string())
            .into(),
    );
    if let Some(copyright) = &bundle.copyright {
        dict.insert("NSHumanReadableCopyright".into(), copyright.clone().into());
    }
    if bundle.iconset.is_some() || bundle.icns.is_some() || bundle.icon.is_some() {
        dict.insert("CFBundleIconFile".into(), "icon.icns".into());
        if bundle.icon.is_some() {
            dict.insert("CFBundleIconName".into(), "icon".into());
        }
    }
    dict.insert("NSHighResolutionCapable".into(), true.into());

    // Merge extra keys from project-local Info.plist
    let info_plist_path = bundle
        .info_plist
        .as_deref()
        .map(|p| format!("{path}/{p}"))
        .unwrap_or_else(|| format!("{path}/Info.plist"));
    if Path::new(&info_plist_path).exists() {
        match plist::Value::from_file(&info_plist_path) {
            Ok(plist::Value::Dictionary(extra)) => {
                for (key, value) in extra {
                    dict.insert(key, value);
                }
            }
            _ => {
                eprintln!("Invalid Info.plist file: root value must be a dictionary");
                exit(1);
            }
        }
    }

    plist::Value::Dictionary(dict)
        .to_file_binary(format!("{target_dir}/Info.plist"))
        .expect("Failed to write binary Info.plist");
}

// Compile the binary and return its path.
fn compile_binary(
    path: &str,
    target_dir: &str,
    package: &Package,
    bundle: &BundleMetadata,
) -> String {
    let lipo = bundle.lipo.unwrap_or(true);
    if lipo {
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
        let output = format!("{target_dir}/{}", bundle.name);
        let lipo_status = Command::new("lipo")
            .args([
                "-create",
                &format!("target/x86_64-apple-darwin/release/{}", package.name),
                &format!("target/aarch64-apple-darwin/release/{}", package.name),
                "-output",
                &output,
            ])
            .status()
            .expect("Failed to run lipo");
        assert!(lipo_status.success(), "lipo failed");
        output
    } else {
        let status = Command::new("cargo")
            .args([
                "build",
                "--release",
                "--manifest-path",
                &format!("{path}/Cargo.toml"),
            ])
            .status()
            .expect("Failed to run cargo build");
        assert!(status.success(), "cargo build failed");
        format!("target/release/{}", package.name)
    }
}

fn create_bundle(
    path: &str,
    target_dir: &str,
    bundle: &BundleMetadata,
    binary_path: &str,
) {
    // Create bundle folder structure
    let bundle_dir = format!("{target_dir}/{}.app/Contents", bundle.name);
    fs::create_dir_all(format!("{bundle_dir}/MacOS")).expect("Can't create directory");
    fs::create_dir_all(format!("{bundle_dir}/Resources")).expect("Can't create directory");

    // Copy files
    if let Some(resources_dir) = &bundle.resources_dir {
        copy_dir(
            format!("{path}/{resources_dir}"),
            format!("{bundle_dir}/Resources"),
        )
        .expect("Failed to copy resources directory");
    }

    if bundle.iconset.is_some() || bundle.icns.is_some() || bundle.icon.is_some() {
        fs::copy(
            format!("{target_dir}/icon.icns"),
            format!("{bundle_dir}/Resources/icon.icns"),
        )
        .expect("Failed to copy icon.icns");
    }

    if bundle.icon.is_some() {
        fs::copy(
            format!("{target_dir}/Assets.car"),
            format!("{bundle_dir}/Resources/Assets.car"),
        )
        .expect("Failed to copy Assets.car");
    }

    fs::copy(binary_path, format!("{bundle_dir}/MacOS/{}", bundle.name))
        .expect("Failed to copy executable");

    fs::copy(
        format!("{target_dir}/Info.plist"),
        format!("{bundle_dir}/Info.plist"),
    )
    .expect("Failed to copy Info.plist");
}

fn build_plugin(
    path: &str,
    target_dir: &str,
    plugins_dir: &str,
    plugin: &PluginMetadata,
    bundle: &BundleMetadata,
    package: &Package,
) {
    let lipo = bundle.lipo.unwrap_or(true);

    // Compile the plugin binary for each target architecture.
    let binary_path = if lipo {
        for target in ["x86_64-apple-darwin", "aarch64-apple-darwin"] {
            let status = Command::new("cargo")
                .args([
                    "build",
                    "--release",
                    "--manifest-path",
                    &format!("{path}/Cargo.toml"),
                    "--target",
                    target,
                    "--bin",
                    &plugin.binary,
                ])
                .status()
                .expect("Failed to run cargo build for plugin");
            assert!(
                status.success(),
                "cargo build failed for plugin {} on {target}",
                plugin.name
            );
        }
        let lipo_out = format!("{target_dir}/{}", plugin.binary);
        let lipo_status = Command::new("lipo")
            .args([
                "-create",
                &format!("target/x86_64-apple-darwin/release/{}", plugin.binary),
                &format!("target/aarch64-apple-darwin/release/{}", plugin.binary),
                "-output",
                &lipo_out,
            ])
            .status()
            .expect("Failed to run lipo for plugin");
        assert!(lipo_status.success(), "lipo failed for plugin {}", plugin.name);
        lipo_out
    } else {
        let status = Command::new("cargo")
            .args([
                "build",
                "--release",
                "--manifest-path",
                &format!("{path}/Cargo.toml"),
                "--bin",
                &plugin.binary,
            ])
            .status()
            .expect("Failed to run cargo build for plugin");
        assert!(status.success(), "cargo build failed for plugin {}", plugin.name);
        format!("target/release/{}", plugin.binary)
    };

    // Create the .appex bundle directory layout.
    let appex_contents = format!("{plugins_dir}/{}.appex/Contents", plugin.name);
    fs::create_dir_all(format!("{appex_contents}/MacOS"))
        .expect("Failed to create appex MacOS directory");

    fs::copy(&binary_path, format!("{appex_contents}/MacOS/{}", plugin.name))
        .expect("Failed to copy plugin binary");

    if lipo {
        fs::remove_file(&binary_path).expect("Failed to remove temporary lipo plugin binary");
    }

    // Assemble the plugin's Info.plist.
    let mut dict = plist::Dictionary::new();
    dict.insert("CFBundleInfoDictionaryVersion".into(), "6.0".into());
    dict.insert("CFBundlePackageType".into(), "XPC!".into());
    dict.insert("CFBundleName".into(), plugin.name.clone().into());
    dict.insert("CFBundleDisplayName".into(), plugin.name.clone().into());
    dict.insert("CFBundleIdentifier".into(), plugin.identifier.clone().into());
    dict.insert("CFBundleVersion".into(), package.version.clone().into());
    dict.insert(
        "CFBundleShortVersionString".into(),
        package.version.clone().into(),
    );
    dict.insert("CFBundleExecutable".into(), plugin.name.clone().into());
    dict.insert(
        "LSMinimumSystemVersion".into(),
        bundle
            .minimal_os_version
            .clone()
            .unwrap_or_else(|| "11.0".to_string())
            .into(),
    );

    // Merge extension-specific Info.plist keys (NSExtension dict, QLSupportedContentTypes, etc.).
    let info_plist_path = format!("{path}/{}", plugin.info_plist);
    if Path::new(&info_plist_path).exists() {
        match plist::Value::from_file(&info_plist_path) {
            Ok(plist::Value::Dictionary(extra)) => {
                for (key, value) in extra {
                    dict.insert(key, value);
                }
            }
            _ => {
                eprintln!("Invalid plugin Info.plist: {}", plugin.info_plist);
                exit(1);
            }
        }
    }

    plist::Value::Dictionary(dict)
        .to_file_binary(format!("{appex_contents}/Info.plist"))
        .expect("Failed to write plugin Info.plist");

    // Sign the plugin with its own entitlements before the parent app is signed.
    let appex_bundle = format!("{plugins_dir}/{}.appex", plugin.name);
    let entitlements_path = plugin
        .entitlements
        .as_deref()
        .map(|e| format!("{path}/{e}"));
    let mut cmd = Command::new("codesign");
    cmd.args(["--force", "--sign", "-"]);
    if let Some(ref ent) = entitlements_path
        && Path::new(ent).exists()
    {
        cmd.args(["--entitlements", ent]);
    }
    cmd.arg(&appex_bundle);
    let status = cmd.status().expect("Failed to run codesign for plugin");
    assert!(status.success(), "codesign failed for plugin {}", plugin.name);
}

fn embed_plugins(path: &str, target_dir: &str, manifest: &Manifest) {
    let bundle = &manifest.package.metadata.bundle;
    let Some(plugins) = &bundle.plugins else {
        return;
    };

    let bundle_dir = format!("{target_dir}/{}.app/Contents", bundle.name);
    let plugins_dir = format!("{bundle_dir}/PlugIns");
    fs::create_dir_all(&plugins_dir).expect("Failed to create PlugIns directory");

    for plugin in plugins {
        println!("  Embedding plugin {} ({})", plugin.name, plugin.binary);
        build_plugin(path, target_dir, &plugins_dir, plugin, bundle, &manifest.package);
    }
}

fn sign_bundle(path: &str, target_dir: &str, bundle: &BundleMetadata) {
    let app_bundle = format!("{target_dir}/{}.app", bundle.name);

    // Resolve entitlements: use manifest field if set, else fall back to root Entitlements.plist
    let entitlements_path = bundle
        .entitlements
        .as_deref()
        .map(|e| format!("{path}/{e}"))
        .unwrap_or_else(|| format!("{path}/Entitlements.plist"));
    let has_entitlements = Path::new(&entitlements_path).exists();

    // Hardened Runtime: explicit manifest field, or enabled by default when entitlements are present
    let use_hardened_runtime = bundle.hardened_runtime.unwrap_or(has_entitlements);

    let mut cmd = Command::new("codesign");
    cmd.args(["--force", "--deep", "--sign", "-"]);
    if use_hardened_runtime {
        cmd.args(["--options", "runtime"]);
    }
    if has_entitlements {
        cmd.args(["--entitlements", &entitlements_path]);
    }
    cmd.arg(&app_bundle);

    let status = cmd.status().expect("Failed to run codesign");
    assert!(status.success(), "codesign failed");
}

fn create_zip(target_dir: &str, bundle: &BundleMetadata) {
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

fn create_dmg(target_dir: &str, bundle: &BundleMetadata) {
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
    #[cfg(unix)]
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
    if !cfg!(target_os = "macos") {
        eprintln!("cargo-bundle can only be run on macOS");
        exit(1);
    }

    // Subcommands
    let args = args::parse_args();
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

    println!(
        "Bundling {} v{} ({})",
        bundle.name, manifest.package.version, args.path
    );

    // Generate resources
    let target_dir = format!("target/bundle/{}", manifest.package.name);
    generate_resources(&args.path, &target_dir, &manifest);

    // Compile binary
    let binary_path = compile_binary(&args.path, &target_dir, &manifest.package, bundle);

    // Create bundle folder structure
    create_bundle(&args.path, &target_dir, bundle, &binary_path);

    // Build and embed app extension plugins into Contents/PlugIns/
    embed_plugins(&args.path, &target_dir, &manifest);

    // Ad-hoc code sign the bundle
    sign_bundle(&args.path, &target_dir, bundle);

    // Create zip
    if args.zip {
        create_zip(&target_dir, bundle);
    }

    // Create dmg installer
    if args.dmg {
        create_dmg(&target_dir, bundle);
    }
}
