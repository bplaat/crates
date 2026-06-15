/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs::{self};
use std::path::PathBuf;
use std::process::{Command, exit};

use crate::Bobje;
use crate::args::Profile;
use crate::executor::ExecutorBuilder;
use crate::manifest::BundleMetadata;
use crate::utils::{index_files, write_bytes_when_different};

// MARK: Bundle tasks
pub(crate) fn detect_bundle(bobje: &Bobje) -> bool {
    bobje.manifest.package.metadata.bundle.is_some()
}

pub(crate) fn bundle_is_lipo(bobje: &Bobje) -> bool {
    bobje
        .manifest
        .package
        .metadata
        .bundle
        .as_ref()
        .is_some_and(|b| b.lipo.unwrap_or(bobje.profile == Profile::Release))
}

pub(crate) fn generate_bundle_tasks(bobje: &Bobje, executor: &mut ExecutorBuilder) {
    let bundle_metadata = &bobje
        .manifest
        .package
        .metadata
        .bundle
        .as_ref()
        .expect("Should be some");
    let contents_dir = format!("{}/{}.app/Contents", bobje.out_dir(), bobje.name);
    let mut bundle_files = Vec::new();

    // Copy resources
    if let Some(resources_dir) = &bundle_metadata.resources_dir {
        let resource_files = index_files(resources_dir);
        for resource_file in &resource_files {
            let dest = format!(
                "{}/Resources/{}",
                contents_dir,
                resource_file
                    .trim_start_matches(resources_dir.as_str())
                    .trim_start_matches(['/', '\\'])
            );
            executor.add_task_cp(resource_file.to_string(), dest.clone());
            bundle_files.push(dest);
        }
    }

    // Compile iconset
    if let Some(iconset) = &bundle_metadata.iconset {
        let iconset_path = PathBuf::from(iconset);
        let icon_name = iconset_path
            .file_stem()
            .expect("Invalid iconset path")
            .to_str()
            .expect("Invalid UTF-8 sequence");
        executor.add_task_cmd(
            format!(
                "iconutil -c icns {} -o {}/{}.icns",
                iconset,
                bobje.out_dir(),
                icon_name
            ),
            vec![iconset.clone()],
            vec![format!("{}/{}.icns", bobje.out_dir(), icon_name)],
        );

        // Copy .icns
        let dest = format!("{contents_dir}/Resources/{icon_name}.icns");
        executor.add_task_cp(
            format!("{}/{}.icns", bobje.out_dir(), icon_name),
            dest.clone(),
        );
        bundle_files.push(dest);
    }

    // Copy icns file
    if let Some(icns) = &bundle_metadata.icns {
        let icns_path = PathBuf::from(icns);
        let icns_name = icns_path
            .file_name()
            .expect("Invalid icns path")
            .to_str()
            .expect("Invalid UTF-8 sequence");
        let dest = format!("{contents_dir}/Resources/{icns_name}");
        executor.add_task_cp(icns.clone(), dest.clone());
        bundle_files.push(dest);
    }

    // Compile .icon file
    if let Some(icon) = &bundle_metadata.icon {
        let icon_path = PathBuf::from(icon);
        let icon_name = icon_path
            .file_stem()
            .expect("Invalid icon path")
            .to_str()
            .expect("Invalid UTF-8 sequence");
        // actool resolves paths relative to the icon file, not the process CWD, so use absolute paths
        let cwd = std::env::current_dir().expect("Failed to get CWD");
        let icon_abs = cwd.join(icon).to_string_lossy().into_owned();
        let out_abs = cwd.join(bobje.out_dir()).to_string_lossy().into_owned();
        executor.add_task_cmd(
            format!(
                "actool {icon_abs} --compile {out_abs} --platform macosx \
                --minimum-deployment-target {} \
                --target-device mac --app-icon {icon_name} --include-all-app-icons \
                --output-partial-info-plist {out_abs}/src-gen/partial.plist > /dev/null",
                bundle_metadata.minimal_os_version
            ),
            vec![icon.clone()],
            vec![
                format!("{}/Assets.car", bobje.out_dir()),
                format!("{}/{}.icns", bobje.out_dir(), icon_name),
                format!("{}/src-gen/partial.plist", bobje.out_dir()),
            ],
        );

        // Copy Assets.car
        let dest = format!("{contents_dir}/Resources/Assets.car");
        executor.add_task_cp(format!("{}/Assets.car", bobje.out_dir()), dest.clone());
        bundle_files.push(dest);

        // Copy .icns
        let dest = format!("{contents_dir}/Resources/{icon_name}.icns");
        executor.add_task_cp(
            format!("{}/{}.icns", bobje.out_dir(), icon_name),
            dest.clone(),
        );
        bundle_files.push(dest);
    }

    // Generate Info.plist
    let info_plist_file = bundle_metadata
        .info_plist
        .as_deref()
        .unwrap_or("Info.plist");
    let extra_dict = if fs::metadata(info_plist_file).is_ok() {
        match plist::Value::from_file(info_plist_file) {
            Ok(plist::Value::Dictionary(dict)) => Some(dict),
            _ => {
                eprintln!("Invalid Info.plist file: root value must be a dictionary");
                exit(1);
            }
        }
    } else {
        None
    };
    generate_info_plist(bobje, bundle_metadata, extra_dict);

    // Copy Info.plist
    let dest = format!("{contents_dir}/Info.plist");
    executor.add_task_cp(
        format!("{}/src-gen/Info.plist", bobje.out_dir()),
        dest.clone(),
    );
    bundle_files.push(dest);

    // Generate lipo binary
    let lipo = bundle_metadata
        .lipo
        .unwrap_or(bobje.profile == Profile::Release);
    if lipo {
        let x86_64 = format!(
            "{}/x86_64-apple-darwin/{}/{}",
            bobje.target_dir, bobje.profile, bobje.name
        );
        let aarch64 = format!(
            "{}/aarch64-apple-darwin/{}/{}",
            bobje.target_dir, bobje.profile, bobje.name,
        );
        executor.add_task_cmd(
            format!(
                "lipo -create {} {} -output {}/{}",
                x86_64,
                aarch64,
                bobje.out_dir(),
                bobje.name
            ),
            vec![x86_64, aarch64],
            vec![format!("{}/{}", bobje.out_dir(), bobje.name)],
        );
    }

    // Copy executable
    let dest = format!("{}/MacOS/{}", contents_dir, bobje.name);
    executor.add_task_cp(
        format!(
            "{}/{}",
            if lipo {
                bobje.out_dir()
            } else {
                bobje.out_dir_with_target()
            },
            bobje.name
        ),
        dest.clone(),
    );
    bundle_files.push(dest);

    // Create phony bundle task
    executor.add_task_phony(
        bundle_files,
        vec![format!("{}/{}.app", bobje.out_dir(), bobje.name)],
    );
}

pub(crate) fn run_bundle(bobje: &Bobje) -> ! {
    let status = Command::new(format!(
        "{}/{}.app/Contents/MacOS/{}",
        bobje.out_dir(),
        bobje.name,
        bobje.name
    ))
    .status()
    .expect("Failed to execute executable");
    exit(status.code().unwrap_or(1))
}

#[cfg(target_os = "macos")]
pub(crate) fn sign_bundle(bobje: &Bobje) {
    let bundle_metadata = bobje
        .manifest
        .package
        .metadata
        .bundle
        .as_ref()
        .expect("Should be some");
    let bundle_path = format!("{}/{}.app", bobje.out_dir(), bobje.name);

    // Resolve entitlements: use manifest field if set, else fall back to Entitlements.plist
    let entitlements_path = bundle_metadata
        .entitlements
        .clone()
        .unwrap_or_else(|| "Entitlements.plist".to_string());
    let has_entitlements = fs::metadata(&entitlements_path).is_ok();

    // Hardened Runtime: explicit manifest field, or enabled by default when entitlements are present
    let use_hardened_runtime = bundle_metadata.hardened_runtime.unwrap_or(has_entitlements);

    let mut args = vec![
        "--force".to_string(),
        "--deep".to_string(),
        "--sign".to_string(),
        "-".to_string(),
    ];
    if let Some(id) = &bobje.manifest.package.id {
        args.push("--identifier".to_string());
        args.push(id.clone());
    }
    if use_hardened_runtime {
        args.push("--options".to_string());
        args.push("runtime".to_string());
    }
    if has_entitlements {
        args.push("--entitlements".to_string());
        args.push(entitlements_path);
    }
    args.push(bundle_path);

    let status = Command::new("codesign")
        .args(&args)
        .status()
        .expect("Failed to run codesign");
    if !status.success() {
        eprintln!("codesign failed");
        exit(1);
    }
}

// MARK: Utils
fn generate_info_plist(
    bobje: &Bobje,
    bundle: &BundleMetadata,
    extra_dict: Option<plist::Dictionary>,
) {
    let id = bobje.manifest.package.id.as_ref().unwrap_or_else(|| {
        eprintln!("Manifest package id is required");
        exit(1);
    });

    let mut dict = plist::Dictionary::new();
    dict.insert("CFBundlePackageType".into(), "APPL".into());
    dict.insert("CFBundleName".into(), bobje.name.clone().into());
    dict.insert("CFBundleDisplayName".into(), bobje.name.clone().into());
    dict.insert("CFBundleIdentifier".into(), id.clone().into());
    dict.insert("CFBundleVersion".into(), bobje.version.clone().into());
    dict.insert(
        "CFBundleShortVersionString".into(),
        bobje.version.clone().into(),
    );
    dict.insert("CFBundleExecutable".into(), bobje.name.clone().into());
    dict.insert(
        "LSMinimumSystemVersion".into(),
        bundle.minimal_os_version.clone().into(),
    );

    if let Some(copyright) = &bundle.copyright {
        dict.insert("NSHumanReadableCopyright".into(), copyright.clone().into());
    }
    if let Some(iconset) = &bundle.iconset {
        let icon_name = PathBuf::from(iconset)
            .file_stem()
            .expect("Invalid iconset path")
            .to_str()
            .expect("Invalid UTF-8 sequence")
            .to_string();
        dict.insert(
            "CFBundleIconFile".into(),
            format!("{icon_name}.icns").into(),
        );
    }
    if let Some(icns) = &bundle.icns {
        let icns_name = PathBuf::from(icns)
            .file_name()
            .expect("Invalid icns path")
            .to_str()
            .expect("Invalid UTF-8 sequence")
            .to_string();
        dict.insert("CFBundleIconFile".into(), icns_name.into());
    }
    if let Some(icon) = &bundle.icon {
        let icon_path = PathBuf::from(icon);
        let icon_name = icon_path
            .file_stem()
            .expect("Invalid icon path")
            .to_str()
            .expect("Invalid UTF-8 sequence")
            .to_string();
        dict.insert(
            "CFBundleIconFile".into(),
            format!("{icon_name}.icns").into(),
        );
        dict.insert("CFBundleIconName".into(), icon_name.into());
    }
    if let Some(extra) = extra_dict {
        for (key, value) in extra {
            dict.insert(key, value);
        }
    }

    let mut bytes = Vec::new();
    plist::to_writer_binary(&mut bytes, &plist::Value::Dictionary(dict))
        .expect("Can't serialize Info.plist");
    write_bytes_when_different(&format!("{}/src-gen/Info.plist", bobje.out_dir()), &bytes)
        .expect("Can't write src-gen/Info.plist");
}
