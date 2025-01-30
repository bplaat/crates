/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs::File;
use std::io::Write;
use std::process::Command;

use crate::Project;

fn generate_common(f: &mut impl Write, project: &Project) {
    _ = writeln!(f, "objects_dir = $target_dir/objects");
    _ = write!(f, "cflags = -Wall -Wextra -Wpedantic -Werror");
    if let Some(build) = &project.manifest.build {
        if let Some(cflags) = &build.cflags {
            _ = write!(f, " {}", cflags);
        }
    }
    _ = writeln!(f);
}

pub(crate) fn generate_c(f: &mut impl Write, project: &Project) {
    let c_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".c"))
        .cloned()
        .collect::<Vec<String>>();
    if !c_source_files.is_empty() {
        _ = writeln!(f, "\n# Build C objects");
        generate_common(f, project);
        _ = writeln!(
        f,
        "\nrule cc\n  command = gcc -c $cflags --std=c11 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = cc $in\n"
    );
        for source_file in &c_source_files {
            let object_file = format!("$objects_dir/{}", source_file.replace(".c", ".o"));
            _ = writeln!(f, "build {}: cc $source_dir/{}", object_file, source_file);
        }
    }
}

pub(crate) fn generate_cpp(f: &mut impl Write, project: &Project) {
    let cpp_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".cpp"))
        .cloned()
        .collect::<Vec<String>>();
    if !cpp_source_files.is_empty() {
        _ = writeln!(f, "\n# Build C++ objects");
        generate_common(f, project);
        _ = writeln!(
            f,
            "\nrule cpp\n  command = g++ -c $cflags --std=c++17 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = cpp $in\n"
        );
        for source_file in &cpp_source_files {
            let object_file = format!("$objects_dir/{}", source_file.replace(".cpp", ".o"));
            _ = writeln!(f, "build {}: cpp $source_dir/{}", object_file, source_file);
        }
    }
}

pub(crate) fn generate_objc(f: &mut impl Write, project: &Project) {
    let m_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".m"))
        .cloned()
        .collect::<Vec<String>>();
    if !m_source_files.is_empty() {
        _ = writeln!(f, "\n# Build Objective-C objects");
        generate_common(f, project);
        _ = writeln!(
            f,
            "\nrule objc\n  command = gcc -x objective-c -c $cflags --std=c11 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = objc $in\n"
        );
        for source_file in &m_source_files {
            let object_file = format!("$objects_dir/{}", source_file.replace(".m", ".o"));
            _ = writeln!(f, "build {}: objc $source_dir/{}", object_file, source_file);
        }
    }
}

pub(crate) fn generate_objcpp(f: &mut impl Write, project: &Project) {
    let mm_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".mm"))
        .cloned()
        .collect::<Vec<String>>();
    if !mm_source_files.is_empty() {
        _ = writeln!(f, "\n# Build Objective-C++ objects");
        generate_common(f, project);
        _ = writeln!(
            f,
            "\nrule objcpp\n  command = g++ -x objective-c++ -c $cflags --std=c++17 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = objcpp $in\n"
        );
        for source_file in &mm_source_files {
            let object_file = format!("$objects_dir/{}", source_file.replace(".mm", ".o"));
            _ = writeln!(
                f,
                "build {}: objcpp $source_dir/{}",
                object_file, source_file
            );
        }
    }
}

pub(crate) fn generate_ld(f: &mut impl Write, project: &Project) {
    let mut object_files = Vec::new();
    let mut contains_cpp = false;
    let mut contains_objc = false;
    for source_file in &project.source_files {
        if source_file.ends_with(".c") {
            object_files.push(format!("$objects_dir/{}", source_file.replace(".c", ".o")));
        }
        if source_file.ends_with(".cpp") {
            object_files.push(format!(
                "$objects_dir/{}",
                source_file.replace(".cpp", ".o")
            ));
            contains_cpp = true;
        }
        if source_file.ends_with(".m") {
            object_files.push(format!("$objects_dir/{}", source_file.replace(".m", ".o")));
            contains_objc = true;
        }
        if source_file.ends_with(".mm") {
            object_files.push(format!("$objects_dir/{}", source_file.replace(".mm", ".o")));
            contains_cpp = true;
            contains_objc = true;
        }
    }

    let mut ldflags = "".to_string();
    if contains_objc {
        ldflags.push_str("-framework Foundation");
    }
    if let Some(build) = &project.manifest.build {
        if let Some(ldflags_extra) = &build.ldflags {
            ldflags.push(' ');
            ldflags.push_str(ldflags_extra);
        }
    }

    _ = writeln!(f, "\n# Link executable");
    _ = writeln!(f, "ldflags = {}\n", ldflags);
    _ = writeln!(
        f,
        "rule ld\n  command = {} $ldflags $in -o $out\n  description = ld $out\n",
        if contains_cpp { "g++" } else { "gcc" }
    );
    #[cfg(windows)]
    let executable_file = "$target_dir/$name.exe";
    #[cfg(not(windows))]
    let executable_file = "$target_dir/$name";
    _ = writeln!(
        f,
        "build {}: ld {}",
        executable_file,
        object_files.join(" ")
    );
}

pub(crate) fn generate_bundle(f: &mut impl Write, project: &Project) {
    let bundle = &project
        .manifest
        .package
        .metadata
        .as_ref()
        .and_then(|m| m.bundle.as_ref())
        .expect("Should be some");

    // Write Info.plist
    let mut i = File::create(format!("{}/target/Info.plist", project.manifest_dir))
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
    _ = writeln!(
        i,
        r#"    <string>{}</string>"#,
        project.manifest.package.name
    );
    _ = writeln!(i, r#"    <key>CFBundleDisplayName</key>"#);
    _ = writeln!(
        i,
        r#"    <string>{}</string>"#,
        project.manifest.package.name
    );
    _ = writeln!(i, r#"    <key>CFBundleIdentifier</key>"#);
    _ = writeln!(
        i,
        r#"    <string>{}</string>"#,
        project
            .manifest
            .package
            .identifier
            .as_ref()
            .expect("Identifier is required")
    );
    _ = writeln!(i, r#"    <key>CFBundleVersion</key>"#);
    _ = writeln!(
        i,
        r#"    <string>{}</string>"#,
        project.manifest.package.version
    );
    _ = writeln!(i, r#"    <key>CFBundleShortVersionString</key>"#);
    _ = writeln!(
        i,
        r#"    <string>{}</string>"#,
        project.manifest.package.version
    );
    _ = writeln!(i, r#"    <key>CFBundleExecutable</key>"#);
    _ = writeln!(
        i,
        r#"    <string>{}</string>"#,
        project.manifest.package.name
    );
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
    #[cfg(windows)]
    let executable_file = "$target_dir/$name.exe";
    #[cfg(not(windows))]
    let executable_file = "$target_dir/$name";
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

pub(crate) fn run_ld(project: &Project) {
    let status = Command::new(
        #[cfg(windows)]
        format!(
            "{}/target/{}.exe",
            project.manifest_dir, project.manifest.package.name
        ),
        #[cfg(not(windows))]
        format!(
            "{}/target/{}",
            project.manifest_dir, project.manifest.package.name
        ),
    )
    .status()
    .expect("Failed to execute executable");
    std::process::exit(status.code().unwrap_or(1));
}

pub(crate) fn run_bundle(project: &Project) {
    let status = Command::new("open")
        .arg(format!(
            "{}/target/{}.app",
            project.manifest_dir, project.manifest.package.name
        ))
        .status()
        .expect("Failed to execute executable");
    std::process::exit(status.code().unwrap_or(1));
}
