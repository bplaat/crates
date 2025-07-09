/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::Write;
use std::fs::{self};
use std::path::PathBuf;
use std::process::{Command, exit};

use regex::Regex;

use crate::Bobje;
use crate::executor::Executor;
use crate::manifest::BundleMetadata;
use crate::utils::{index_files, write_file_when_different};

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
        .is_some_and(|b| b.lipo)
}

pub(crate) fn generate_bundle_tasks(bobje: &Bobje, executor: &mut Executor) {
    let bundle_metadata = &bobje
        .manifest
        .package
        .metadata
        .bundle
        .as_ref()
        .expect("Should be some");
    let contents_dir = format!(
        "{}/{}.app/Contents",
        bobje.out_dir(),
        bobje.manifest.package.name
    );
    let mut bundle_files = Vec::new();

    // Copy resources
    if fs::metadata(&bundle_metadata.resources_dir).is_ok() {
        let resource_files = index_files(&bundle_metadata.resources_dir);
        for resource_file in &resource_files {
            let dest = format!(
                "{}/Resources/{}",
                contents_dir,
                resource_file
                    .trim_start_matches(&bundle_metadata.resources_dir)
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
        let dest = format!("{}/Resources/{}.icns", contents_dir, icon_name);
        executor.add_task_cp(
            format!("{}/{}.icns", bobje.out_dir(), icon_name),
            dest.clone(),
        );
        bundle_files.push(dest);
    }

    // Generate Info.plist
    let info_plist_file = "Info.plist";
    let extra_keys = if fs::metadata(info_plist_file).is_ok() {
        let contents = fs::read_to_string(info_plist_file).expect("Can't create Info.plist");
        let re = Regex::new(r"<dict>([\s\S]*?)<\/dict>").expect("Can't compile regex");
        if let Some(captures) = re.captures(&contents) {
            Some(
                captures
                    .get(1)
                    .map_or("", |m| m.as_str())
                    .trim()
                    .to_string(),
            )
        } else {
            eprintln!("Invalid Info.plist file place extra keys inside the <dict> tag");
            exit(1);
        }
    } else {
        None
    };
    generate_info_plist(bobje, bundle_metadata, extra_keys.as_deref());

    // Copy Info.plist
    let dest = format!("{}/Info.plist", contents_dir);
    executor.add_task_cp(
        format!("{}/src-gen/Info.plist", bobje.out_dir()),
        dest.clone(),
    );
    bundle_files.push(dest);

    // Generate lipo binary
    if bundle_metadata.lipo {
        let x86_64 = format!(
            "{}/x86_64-apple-darwin/{}/{}",
            bobje.target_dir, bobje.profile, bobje.manifest.package.name
        );
        let aarch64 = format!(
            "{}/aarch64-apple-darwin/{}/{}",
            bobje.target_dir, bobje.profile, bobje.manifest.package.name,
        );
        executor.add_task_cmd(
            format!(
                "lipo -create {} {} -output {}/{}",
                x86_64,
                aarch64,
                bobje.out_dir(),
                bobje.manifest.package.name
            ),
            vec![x86_64, aarch64],
            vec![format!(
                "{}/{}",
                bobje.out_dir(),
                bobje.manifest.package.name
            )],
        );
    }

    // Copy executable
    let dest = format!("{}/MacOS/{}", contents_dir, bobje.manifest.package.name);
    executor.add_task_cp(
        format!(
            "{}/{}",
            if bundle_metadata.lipo {
                bobje.out_dir()
            } else {
                bobje.out_dir_with_target()
            },
            bobje.manifest.package.name
        ),
        dest.clone(),
    );
    bundle_files.push(dest);

    // Create phony bundle task
    executor.add_task_phony(
        bundle_files,
        vec![format!(
            "{}/{}.app",
            bobje.out_dir(),
            bobje.manifest.package.name
        )],
    );
}

pub(crate) fn run_bundle(bobje: &Bobje) -> ! {
    let status = Command::new(format!(
        "{}/{}.app/Contents/MacOS/{}",
        bobje.out_dir(),
        bobje.manifest.package.name,
        bobje.manifest.package.name
    ))
    .status()
    .expect("Failed to execute executable");
    exit(status.code().unwrap_or(1))
}

// MARK: Utils
fn generate_info_plist(bobje: &Bobje, bundle: &BundleMetadata, extra_keys: Option<&str>) {
    let id = bobje.manifest.package.id.as_ref().unwrap_or_else(|| {
        eprintln!("Manifest package id is required");
        exit(1);
    });

    let mut s = String::new();
    _ = writeln!(s, "<!-- This file is generated by bob, do not edit! -->");
    _ = writeln!(s, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    _ = writeln!(
        s,
        r#"<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">"#
    );
    _ = writeln!(s, r#"<plist version="1.0">"#);
    _ = writeln!(s, r#"<dict>"#);
    _ = writeln!(s, r#"	<key>CFBundlePackageType</key>"#);
    _ = writeln!(s, r#"	<string>APPL</string>"#);
    _ = writeln!(s, r#"	<key>CFBundleName</key>"#);
    _ = writeln!(s, r#"	<string>{}</string>"#, bobje.manifest.package.name);
    _ = writeln!(s, r#"	<key>CFBundleDisplayName</key>"#);
    _ = writeln!(s, r#"	<string>{}</string>"#, bobje.manifest.package.name);
    _ = writeln!(s, r#"	<key>CFBundleIdentifier</key>"#);
    _ = writeln!(s, r#"	<string>{}</string>"#, id);
    _ = writeln!(s, r#"	<key>CFBundleVersion</key>"#);
    _ = writeln!(s, r#"	<string>{}</string>"#, bobje.manifest.package.version);
    _ = writeln!(s, r#"	<key>CFBundleShortVersionString</key>"#);
    _ = writeln!(s, r#"	<string>{}</string>"#, bobje.manifest.package.version);
    _ = writeln!(s, r#"	<key>CFBundleExecutable</key>"#);
    _ = writeln!(s, r#"	<string>{}</string>"#, bobje.manifest.package.name);
    _ = writeln!(s, r#"	<key>LSMinimumSystemVersion</key>"#);
    _ = writeln!(s, r#"	<string>11.0</string>"#,);
    if let Some(copyright) = &bundle.copyright {
        _ = writeln!(s, r#"	<key>NSHumanReadableCopyright</key>"#);
        _ = writeln!(s, r#"	<string>{}</string>"#, copyright);
    }
    if let Some(iconset) = &bundle.iconset {
        let iconset_path = PathBuf::from(iconset);
        let icon_name = iconset_path
            .file_stem()
            .expect("Invalid iconset path")
            .to_str()
            .expect("Invalid UTF-8 sequence");
        _ = writeln!(s, r#"	<key>CFBundleIconFile</key>"#);
        _ = writeln!(s, r#"	<string>{}.icns</string>"#, icon_name);
    }
    if let Some(extra_keys) = extra_keys {
        _ = writeln!(s, "{}", extra_keys);
    }
    _ = writeln!(s, r#"</dict>"#);
    _ = writeln!(s, r#"</plist>"#);

    write_file_when_different(&format!("{}/src-gen/Info.plist", bobje.out_dir()), &s)
        .expect("Can't write src-gen/Info.plist");
}
