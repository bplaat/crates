/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::io::Write;

use crate::Manifest;

pub(crate) fn generate_ninja(
    f: &mut impl Write,
    _manifest_dir: &str,
    _manifest: &Manifest,
    source_files: &[String],
) {
    // Build objects
    _ = writeln!(f, "objects_dir = $target_dir/objects");
    let mut object_files = Vec::new();

    // Build C objects
    let c_source_files = source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".c"))
        .cloned()
        .collect::<Vec<String>>();
    if !c_source_files.is_empty() {
        _ = writeln!(f, "# Build C objects");
        _ = writeln!(f, "objects_dir = $target_dir/objects");
        _ = writeln!(f, "cflags = -Wall -Wextra -Wpedantic -Werror --std=c11\n");
        _ = writeln!(
        f,
        "rule cc\n  command = gcc -c $cflags -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = cc $in\n"
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
            "cxxflags = -Wall -Wextra -Wpedantic -Werror --std=c++17\n"
        );
        _ = writeln!(
            f,
            "rule cxx\n  command = g++ -c $cxxflags -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = cxx $in\n"
        );
        for source_file in &cpp_source_files {
            let object_file = format!("$objects_dir/{}", source_file.replace(".cpp", ".o"));
            _ = writeln!(f, "build {}: cxx $source_dir/{}", object_file, source_file);
            object_files.push(object_file);
        }
    }

    // Link executable
    _ = writeln!(f, "\n# Link executable");
    _ = writeln!(f, "ldflags =\n");
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
    let executable_file = "$target_dir/${name}-$version.exe";
    #[cfg(not(windows))]
    let executable_file = "$target_dir/${name}-$version";
    _ = writeln!(
        f,
        "build {}: ld {}",
        executable_file,
        object_files.join(" ")
    );
}

pub(crate) fn run(manifest_dir: &str, manifest: &Manifest, _source_files: &[String]) {
    let status = std::process::Command::new(
        #[cfg(windows)]
        format!(
            "{}/target/{}-{}.exe",
            manifest_dir, manifest.package.name, manifest.package.version
        ),
        #[cfg(not(windows))]
        format!(
            "{}/target/{}-{}",
            manifest_dir, manifest.package.name, manifest.package.version
        ),
    )
    .status()
    .expect("Failed to execute executable");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}
