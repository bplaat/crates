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

// Run cargo build with --message-format json and return the OUT_DIR for the given package.
// Falls back to globbing target/ when build-script-executed is absent (cached incremental builds).
fn cargo_build_get_out_dir(cargo_args: &[&str], package_name: &str) -> Option<String> {
    let output = Command::new("cargo")
        .args(cargo_args)
        .args(["--message-format", "json-render-diagnostics"])
        .output()
        .expect("Failed to run cargo build");
    assert!(output.status.success(), "cargo build failed");
    // Parse each JSON line looking for build-script-executed for our package.
    // package_id format is "name version (source)", so match on "package_id":"name
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if line.contains("\"reason\":\"build-script-executed\"")
            && line.contains(&format!("\"package_id\":\"{}", package_name))
        {
            if let Some(start) = line.find("\"out_dir\":\"") {
                let rest = &line[start + 11..];
                if let Some(end) = rest.find('"') {
                    return Some(rest[..end].replace("\\\\", "\\").replace("\\/", "/"));
                }
            }
        }
    }
    // Fallback: find the most recently modified build output dir in target/.
    // Cargo uses target/{target}/release/build/{name}-{hash}/out for each target.
    find_out_dir_in_target(cargo_args, package_name)
}

// Scan target/ for the most recently modified build output dir matching {name}-*.
fn find_out_dir_in_target(cargo_args: &[&str], package_name: &str) -> Option<String> {
    // Determine the target triple from the cargo args (--target <triple>)
    let target_triple = cargo_args
        .windows(2)
        .find(|w| w[0] == "--target")
        .map(|w| w[1]);
    let build_dir = if let Some(triple) = target_triple {
        format!("target/{triple}/release/build")
    } else {
        "target/release/build".to_string()
    };
    let prefix = format!("{package_name}-");
    let mut best: Option<(std::time::SystemTime, String)> = None;
    let Ok(entries) = fs::read_dir(&build_dir) else {
        return None;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with(&prefix) {
            continue;
        }
        let out = entry.path().join("out");
        if !out.is_dir() {
            continue;
        }
        if let Ok(meta) = out.metadata() {
            if let Ok(modified) = meta.modified() {
                if best.as_ref().is_none_or(|(t, _)| modified > *t) {
                    // canonicalize to absolute path so resolve_folder_path doesn't
                    // mistake it for a manifest-relative path
                    let abs = fs::canonicalize(&out)
                        .unwrap_or(out)
                        .to_string_lossy()
                        .into_owned();
                    best = Some((modified, abs));
                }
            }
        }
    }
    best.map(|(_, path)| path)
}

// Compile the binary and return (binary_path, Option<out_dir>).
fn compile_binary(
    path: &str,
    target_dir: &str,
    package: &manifest::Package,
    bundle: &manifest::BundleMetadata,
    has_macos_bundle: bool,
) -> (String, Option<String>) {
    let lipo = bundle.lipo.unwrap_or(true);
    let manifest_path = format!("{path}/Cargo.toml");
    if lipo {
        let mut out_dir = None;
        for target in ["x86_64-apple-darwin", "aarch64-apple-darwin"] {
            let mut args = vec!["build", "--release", "--manifest-path", &manifest_path, "--target", target];
            if has_macos_bundle {
                args.extend(["--features", "macos-bundle"]);
            }
            let detected = cargo_build_get_out_dir(&args, &package.name);
            if out_dir.is_none() {
                out_dir = detected;
            }
        }
        let binary = format!("{target_dir}/{}", bundle.name);
        let lipo_status = Command::new("lipo")
            .args([
                "-create",
                &format!("target/x86_64-apple-darwin/release/{}", package.name),
                &format!("target/aarch64-apple-darwin/release/{}", package.name),
                "-output",
                &binary,
            ])
            .status()
            .expect("Failed to run lipo");
        assert!(lipo_status.success(), "lipo failed");
        (binary, out_dir)
    } else {
        let mut args = vec!["build", "--release", "--manifest-path", &manifest_path];
        if has_macos_bundle {
            args.extend(["--features", "macos-bundle"]);
        }
        let out_dir = cargo_build_get_out_dir(&args, &package.name);
        (format!("target/release/{}", package.name), out_dir)
    }
}

fn scan_rs_for_folders(dir: &str, folders: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_rs_for_folders(&path.to_string_lossy(), folders);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            for line in content.lines() {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix("#[folder = \"") {
                    if let Some(folder) = rest.strip_suffix("\"]") {
                        folders.push(folder.to_string());
                    }
                }
            }
        }
    }
}

fn find_embed_folders(manifest_dir: &str) -> Vec<String> {
    let mut folders = Vec::new();
    scan_rs_for_folders(&format!("{manifest_dir}/src"), &mut folders);
    folders.sort();
    folders.dedup();
    folders
}

fn resolve_folder_path(folder: &str, manifest_dir: &str, out_dir: Option<&str>) -> String {
    let mut resolved = folder.to_string();
    if let Some(dir) = out_dir {
        resolved = resolved.replace("$OUT_DIR", dir);
    }
    for (key, value) in std::env::vars() {
        resolved = resolved.replace(&format!("${key}"), &value);
    }
    if Path::new(&resolved).is_relative() {
        format!("{manifest_dir}/{resolved}")
    } else {
        resolved
    }
}

fn create_bundle(
    path: &str,
    target_dir: &str,
    bundle: &manifest::BundleMetadata,
    binary_path: &str,
    out_dir: Option<&str>,
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

    // Discover and copy embed asset dirs from source #[folder = "..."] attributes
    for folder in find_embed_folders(path) {
        let src = resolve_folder_path(&folder, path, out_dir);
        let src_path = Path::new(&src);
        if src_path.exists() {
            let folder_name = src_path
                .file_name()
                .expect("folder has no name")
                .to_string_lossy();
            let dest = format!("{bundle_dir}/Resources/{folder_name}");
            fs::create_dir_all(&dest).expect("Can't create directory");
            copy_dir(&src, &dest).expect("Failed to copy embed resources dir");
        }
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

fn sign_bundle(path: &str, target_dir: &str, bundle: &manifest::BundleMetadata) {
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
    let has_macos_bundle = manifest.features.contains_key("macos-bundle");
    let (binary_path, out_dir) =
        compile_binary(&args.path, &target_dir, &manifest.package, bundle, has_macos_bundle);

    // Create bundle folder structure
    create_bundle(
        &args.path,
        &target_dir,
        bundle,
        &binary_path,
        out_dir.as_deref(),
    );

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
