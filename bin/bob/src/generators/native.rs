/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs::File;
use std::io::Write;
use std::process::Command;

use crate::Manifest;

pub(crate) fn generate_ninja(
    f: &mut impl Write,
    manifest_dir: &str,
    manifest: &Manifest,
    source_files: &[String],
) {
    // Build objects
    _ = writeln!(f, "objects_dir = $target_dir/objects");
    _ = write!(f, "cflags = -Wall -Wextra -Wpedantic -Werror");
    if let Some(build) = &manifest.build {
        if let Some(cflags) = &build.cflags {
            _ = writeln!(f, " {}", cflags);
        } else {
            _ = writeln!(f);
        }
    } else {
        _ = writeln!(f);
    }
    let mut object_files = Vec::new();

    // Build C objects
    let c_source_files = source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".c"))
        .cloned()
        .collect::<Vec<String>>();
    if !c_source_files.is_empty() {
        _ = writeln!(f, "# Build C objects");
        _ = writeln!(
        f,
        "rule cc\n  command = gcc -c $cflags --std=c11 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = cc $in\n"
    );
        for source_file in &c_source_files {
            let object_file = format!("$objects_dir/{}", source_file.replace(".c", ".o"));
            _ = writeln!(f, "build {}: cc $source_dir/{}", object_file, source_file);
            object_files.push(object_file);
        }
    }

    // Build C++ objects
    let cpp_source_files = source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".cpp"))
        .cloned()
        .collect::<Vec<String>>();
    if !cpp_source_files.is_empty() {
        _ = writeln!(f, "\n# Build C++ objects");
        _ = writeln!(
            f,
            "rule cxx\n  command = g++ -c $cflags --std=c++17 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = cxx $in\n"
        );
        for source_file in &cpp_source_files {
            let object_file = format!("$objects_dir/{}", source_file.replace(".cpp", ".o"));
            _ = writeln!(f, "build {}: cxx $source_dir/{}", object_file, source_file);
            object_files.push(object_file);
        }
    }

    // Build Objective-C objects
    let m_source_files = source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".m"))
        .cloned()
        .collect::<Vec<String>>();
    if !m_source_files.is_empty() {
        _ = writeln!(f, "\n# Build Objective-C objects");
        _ = writeln!(
            f,
            "rule objc\n  command = gcc -x objective-c -c $cflags --std=c11 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = objc $in\n"
        );
        for source_file in &m_source_files {
            let object_file = format!("$objects_dir/{}", source_file.replace(".m", ".o"));
            _ = writeln!(f, "build {}: objc $source_dir/{}", object_file, source_file);
            object_files.push(object_file);
        }
    }

    // Build Objective-C++ objects
    let mm_source_files = source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".mm"))
        .cloned()
        .collect::<Vec<String>>();
    if !mm_source_files.is_empty() {
        _ = writeln!(f, "\n# Build Objective-C++ objects");
        _ = writeln!(
            f,
            "rule objcxx\n  command = g++ -x objective-c++ -c $cflags --std=c++17 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = objcxx $in\n"
        );
        for source_file in &mm_source_files {
            let object_file = format!("$objects_dir/{}", source_file.replace(".mm", ".o"));
            _ = writeln!(
                f,
                "build {}: objcxx $source_dir/{}",
                object_file, source_file
            );
            object_files.push(object_file);
        }
    }

    // Link executable
    _ = writeln!(f, "\n# Link executable");
    if !m_source_files.is_empty() || !mm_source_files.is_empty() {
        _ = write!(f, "ldflags = -framework Foundation");
    } else {
        _ = write!(f, "ldflags =");
    }
    if let Some(build) = &manifest.build {
        if let Some(ldflags) = &build.ldflags {
            _ = writeln!(f, " {}", ldflags);
        } else {
            _ = writeln!(f);
        }
    } else {
        _ = writeln!(f);
    }

    _ = writeln!(
        f,
        "rule ld\n  command = {} $ldflags $in -o $out\n  description = ld $out\n",
        if !cpp_source_files.is_empty() {
            "g++"
        } else {
            "gcc"
        }
    );
    #[cfg(windows)]
    let executable_file = "$target_dir/${name}.exe";
    #[cfg(not(windows))]
    let executable_file = "$target_dir/${name}";
    _ = writeln!(
        f,
        "build {}: ld {}",
        executable_file,
        object_files.join(" ")
    );

    // Build macOS bundle
    if let Some(bundle) = &manifest
        .package
        .metadata
        .as_ref()
        .and_then(|m| m.bundle.as_ref())
    {
        // Write Info.plist
        let mut i = File::create(format!("{}/target/Info.plist", manifest_dir))
            .expect("Can't create Info.plist");
        _ = writeln!(i, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        _ = writeln!(
            i,
            r#"<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">"#
        );
        _ = writeln!(i, r#"<plist version="1.0">"#);
        _ = writeln!(i, r#"<dict>"#);
        _ = writeln!(i, r#"    <key>CFBundlePackageType</key>"#);
        _ = writeln!(i, r#"    <string>APPL</string>"#);
        _ = writeln!(i, r#"    <key>CFBundleName</key>"#);
        _ = writeln!(i, r#"    <string>{}</string>"#, manifest.package.name);
        _ = writeln!(i, r#"    <key>CFBundleDisplayName</key>"#);
        _ = writeln!(i, r#"    <string>{}</string>"#, manifest.package.name);
        _ = writeln!(i, r#"    <key>CFBundleIdentifier</key>"#);
        _ = writeln!(
            i,
            r#"    <string>{}</string>"#,
            manifest
                .package
                .identifier
                .as_ref()
                .expect("Identifier is required")
        );
        _ = writeln!(i, r#"    <key>CFBundleVersion</key>"#);
        _ = writeln!(i, r#"    <string>{}</string>"#, manifest.package.version);
        _ = writeln!(i, r#"    <key>CFBundleShortVersionString</key>"#);
        _ = writeln!(i, r#"    <string>{}</string>"#, manifest.package.version);
        _ = writeln!(i, r#"    <key>CFBundleExecutable</key>"#);
        _ = writeln!(i, r#"    <string>{}</string>"#, manifest.package.name);
        _ = writeln!(i, r#"    <key>LSMinimumSystemVersion</key>"#);
        _ = writeln!(i, r#"    <string>11.0</string>"#,);
        _ = writeln!(i, r#"    <key>NSHumanReadableCopyright</key>"#);
        _ = writeln!(i, r#"    <string>{}</string>"#, bundle.copyright);
        _ = writeln!(i, r#"</dict>"#);
        _ = writeln!(i, r#"</plist>"#);
        drop(i);

        // Copy Info.plist
        _ = writeln!(f, "\n# Build macOS bundle");
        _ = writeln!(
            f,
            "rule cp\n  command = cp $in $out\n  description = cp $in\n"
        );
        _ = writeln!(
            f,
            "build $target_dir/$name.app/Contents/MacOS/$name: cp {}",
            executable_file
        );
        _ = writeln!(
            f,
            "build $target_dir/$name.app/Contents/Info.plist: cp $target_dir/Info.plist"
        );
    }
}

pub(crate) fn run(manifest_dir: &str, manifest: &Manifest, _source_files: &[String]) {
    let mut cmd = if manifest
        .package
        .metadata
        .as_ref()
        .and_then(|m| m.bundle.as_ref())
        .is_some()
    {
        let mut cmd = Command::new("open");
        cmd.arg(format!(
            "{}/target/{}.app",
            manifest_dir, manifest.package.name
        ));
        cmd
    } else {
        Command::new(
            #[cfg(windows)]
            format!("{}/target/{}.exe", manifest_dir, manifest.package.name),
            #[cfg(not(windows))]
            format!("{}/target/{}", manifest_dir, manifest.package.name),
        )
    };
    let status = cmd.status().expect("Failed to execute executable");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}
