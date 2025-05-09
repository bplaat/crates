/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, exit};

use indexmap::IndexMap;
use regex::Regex;

use crate::manifest::BundleMetadata;
use crate::utils::{resolve_source_file_path, write_file_when_different};
use crate::{Profile, Project, index_files};

pub(crate) fn generate_cx_vars(f: &mut dyn Write, project: &mut Project) {
    if project.is_test {
        generate_test_main(project);
    }

    _ = writeln!(f, "\n# Cx variables");
    _ = writeln!(f, "objects_dir = $target_dir/$profile/objects");

    // Cflags
    _ = write!(
        f,
        "cflags = {} -Wall -Wextra -Wpedantic -Werror",
        match project.profile {
            Profile::Debug => "-g -DDEBUG".to_string(),
            Profile::Release => "-Os -DRELEASE".to_string(),
        }
    );
    if project.is_test {
        _ = write!(f, " -DTEST {}", pkg_config_cflags("cunit"));
    }
    if let Some(build) = &project.manifest.build {
        if let Some(cflags) = &build.cflags {
            _ = write!(f, " {}", cflags);
        }
    }
    _ = writeln!(f);

    // Ldflags
    _ = write!(f, "ldflags =");
    if project.profile == Profile::Release {
        _ = write!(f, " -Os");
    } else {
        _ = write!(f, " -g");
    }
    if project
        .source_files
        .iter()
        .any(|p| p.ends_with(".m") || p.ends_with(".mm"))
    {
        _ = write!(f, " -framework Foundation");
    }
    if project.is_test {
        _ = write!(f, " {}", pkg_config_libs("cunit"));
    }
    if let Some(build) = &project.manifest.build {
        if let Some(extra_ldflags) = &build.ldflags {
            _ = write!(f, " {}", extra_ldflags);
        }
    }
    _ = writeln!(f);

    // Use Clang on macOS and Windows
    #[cfg(target_os = "macos")]
    {
        _ = writeln!(f, "cc = clang");
        _ = writeln!(f, "cxx = clang++");
        _ = writeln!(f, "strip = strip");
    }
    #[cfg(windows)]
    {
        _ = writeln!(f, "cc = clang");
        _ = writeln!(f, "cxx = clang++");
        _ = writeln!(f, "strip = llvm-strip");
    }
    // Use GCC on Linux and other Unix-like systems
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        _ = writeln!(f, "cc = gcc");
        _ = writeln!(f, "cxx = g++");
        _ = writeln!(f, "strip = strip");
    }
}

pub(crate) fn generate_c(f: &mut dyn Write, project: &Project) {
    let c_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".c"))
        .cloned()
        .collect::<Vec<String>>();
    _ = writeln!(f, "\n# Compile C objects");
    _ = writeln!(
        f,
        "rule cc\n  command = $cc -c $cflags --std=c11 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = Compiling $in\n"
    );
    for source_file in &c_source_files {
        let object_file = source_file
            .replace("$source_dir/", "$objects_dir/")
            .replace("$source_gen_dir/", "$objects_dir/")
            .replace(".c", ".o");
        _ = writeln!(f, "build {}: cc {}", object_file, source_file);
    }
}

pub(crate) fn generate_cpp(f: &mut dyn Write, project: &Project) {
    let cpp_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".cpp"))
        .cloned()
        .collect::<Vec<String>>();
    _ = writeln!(f, "\n# Compile C++ objects");
    _ = writeln!(
        f,
        "rule cpp\n  command = $cxx -c $cflags --std=c++17 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = Compiling $in\n"
    );
    for source_file in &cpp_source_files {
        let object_file = source_file
            .replace("$source_dir/", "$objects_dir/")
            .replace("$source_gen_dir/", "$objects_dir/")
            .replace(".cpp", ".o");
        _ = writeln!(f, "build {}: cpp {}", object_file, source_file);
    }
}

pub(crate) fn generate_objc(f: &mut dyn Write, project: &Project) {
    let m_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".m"))
        .cloned()
        .collect::<Vec<String>>();
    _ = writeln!(f, "\n# Compile Objective-C objects");
    _ = writeln!(
        f,
        "rule objc\n  command = $cc -x objective-c -c $cflags --std=c11 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = Compiling $in\n"
    );
    for source_file in &m_source_files {
        let object_file = source_file
            .replace("$source_dir/", "$objects_dir/")
            .replace("$source_gen_dir/", "$objects_dir/")
            .replace(".m", ".o");
        _ = writeln!(f, "build {}: objc {}", object_file, source_file);
    }
}

pub(crate) fn generate_objcpp(f: &mut dyn Write, project: &Project) {
    let mm_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".mm"))
        .cloned()
        .collect::<Vec<String>>();
    _ = writeln!(f, "\n# Compile Objective-C++ objects");
    _ = writeln!(
        f,
        "rule objcpp\n  command = $cxx -x objective-c++ -c $cflags --std=c++17 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = Compiling $in\n"
    );
    for source_file in &mm_source_files {
        let object_file = source_file
            .replace("$source_dir/", "$objects_dir/")
            .replace("$source_gen_dir/", "$objects_dir/")
            .replace(".mm", ".o");
        _ = writeln!(f, "build {}: objcpp {}", object_file, source_file);
    }
}

pub(crate) fn generate_ld(f: &mut dyn Write, project: &Project) {
    let mut object_files = Vec::new();
    let mut contains_cpp = false;
    if project.is_test {
        let test_functions = find_test_function(project);
        for source_file in test_functions.keys() {
            object_files.push(
                source_file
                    .replace("$source_dir/", "$objects_dir/")
                    .replace("$source_gen_dir/", "$objects_dir/")
                    .replace(".mm", ".o")
                    .replace(".m", ".o")
                    .replace(".cpp", ".o")
                    .replace(".c", ".o"),
            );
        }
    } else {
        for source_file in &project.source_files {
            let source_file = source_file
                .replace("$source_dir/", "$objects_dir/")
                .replace("$source_gen_dir/", "$objects_dir/");
            if source_file.ends_with(".c") {
                object_files.push(source_file.replace(".c", ".o"));
            }
            if source_file.ends_with(".cpp") {
                object_files.push(source_file.replace(".cpp", ".o"));
                contains_cpp = true;
            }
            if source_file.ends_with(".m") {
                object_files.push(source_file.replace(".m", ".o"));
            }
            if source_file.ends_with(".mm") {
                object_files.push(source_file.replace(".mm", ".o"));
                contains_cpp = true;
            }
        }
    }

    _ = writeln!(f, "\n# Link executable");
    #[cfg(windows)]
    let shell = "cmd.exe /c";
    #[cfg(not(windows))]
    let shell = "";
    _ = writeln!(
        f,
        "rule ld\n  command = {} {} $ldflags $in -o $out{}\n  description = Linking $out\n",
        shell,
        if contains_cpp { "$cxx" } else { "$cc" },
        match project.profile {
            Profile::Release => " && $strip $out",
            _ => "",
        }
    );
    #[cfg(windows)]
    let executable_file = if project.is_test {
        "$target_dir/$profile/test_$name.exe"
    } else {
        "$target_dir/$profile/$name.exe"
    };
    #[cfg(not(windows))]
    let executable_file = if project.is_test {
        "$target_dir/$profile/test_$name"
    } else {
        "$target_dir/$profile/$name"
    };
    _ = writeln!(
        f,
        "build {}: ld {}",
        executable_file,
        object_files.join(" ")
    );
}

pub(crate) fn generate_bundle(f: &mut dyn Write, project: &Project) {
    let bundle = &project
        .manifest
        .package
        .metadata
        .as_ref()
        .and_then(|m| m.bundle.as_ref())
        .expect("Should be some");

    // Write Info.plist
    let info_plist_file = format!("{}/Info.plist", project.manifest_dir);
    let extra_keys = if fs::metadata(&info_plist_file).is_ok() {
        let contents = fs::read_to_string(&info_plist_file).expect("Can't create Info.plist");
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
    generate_info_plist(project, bundle, extra_keys.as_deref());

    // Rules
    _ = writeln!(f, "\n# Build macOS bundle");
    _ = writeln!(
        f,
        "rule cp\n  command = cp $in $out\n  description = Copying $in"
    );
    if bundle.iconset.is_some() {
        _ = writeln!(
            f,
            "rule iconutil\n  command = iconutil -c icns $in -o $out\n  description = Converting $in"
        );
    }

    // Copy Info.plist and resources
    _ = writeln!(
        f,
        "\nbuild $target_dir/$profile/$name.app/Contents/Info.plist: cp $target_dir/$profile/src-gen/Info.plist"
    );
    let resources_dir = format!("{}/Resources/", project.manifest_dir);
    if fs::metadata(&resources_dir).is_ok() {
        let resource_files = index_files(&resources_dir)
            .into_iter()
            .map(|file| {
                file.strip_prefix(&resources_dir)
                    .expect("Should be some")
                    .to_string()
            })
            .collect::<Vec<_>>();
        for resource_file in &resource_files {
            _ = writeln!(
                f,
                "build $target_dir/$profile/$name.app/Contents/Resources/{}: cp $manifest_dir/Resources/{}",
                resource_file, resource_file
            );
        }
    }

    // Compile iconset
    if let Some(iconset) = &bundle.iconset {
        _ = writeln!(
            f,
            "build $target_dir/$profile/$name.app/Contents/Resources/{}.icns: iconutil $manifest_dir/{}",
            PathBuf::from(iconset)
                .file_stem()
                .expect("Invalid iconset path")
                .to_str()
                .expect("Invalid UTF-8 sequence"),
            iconset
        );
    }

    // Copy executable
    let executable_file = "$target_dir/$profile/$name";
    _ = writeln!(
        f,
        "build $target_dir/$profile/$name.app/Contents/MacOS/$name: cp {}",
        executable_file
    );
}

pub(crate) fn run_ld(project: &Project) {
    let status = Command::new(
        #[cfg(windows)]
        format!(
            "{}/target/{}/{}.exe",
            project.manifest_dir, project.profile, project.manifest.package.name
        ),
        #[cfg(not(windows))]
        format!(
            "{}/target/{}/{}",
            project.manifest_dir, project.profile, project.manifest.package.name
        ),
    )
    .status()
    .expect("Failed to execute executable");
    exit(status.code().unwrap_or(1));
}

pub(crate) fn run_tests(project: &Project) {
    let status = Command::new(
        #[cfg(windows)]
        format!(
            "{}/target/{}/test_{}.exe",
            project.manifest_dir, project.profile, project.manifest.package.name
        ),
        #[cfg(not(windows))]
        format!(
            "{}/target/{}/test_{}",
            project.manifest_dir, project.profile, project.manifest.package.name
        ),
    )
    .status()
    .expect("Failed to execute executable");
    exit(status.code().unwrap_or(1));
}

pub(crate) fn run_bundle(project: &Project) {
    let status = Command::new(format!(
        "{}/target/{}/{}.app/Contents/MacOS/{}",
        project.manifest_dir,
        project.profile,
        project.manifest.package.name,
        project.manifest.package.name
    ))
    .status()
    .expect("Failed to execute executable");
    exit(status.code().unwrap_or(1));
}

fn pkg_config_cflags(package: &str) -> String {
    let output = Command::new("pkg-config")
        .arg("--cflags")
        .arg(package)
        .output()
        .expect("Failed to execute pkg-config");
    if output.status.success() {
        String::from_utf8(output.stdout)
            .expect("Invalid UTF-8 sequence")
            .replace('\n', " ")
    } else {
        eprintln!(
            "pkg-config failed with error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        exit(1);
    }
}

fn pkg_config_libs(package: &str) -> String {
    let output = Command::new("pkg-config")
        .arg("--libs")
        .arg(package)
        .output()
        .expect("Failed to execute pkg-config");
    if output.status.success() {
        String::from_utf8(output.stdout)
            .expect("Invalid UTF-8 sequence")
            .replace('\n', " ")
    } else {
        eprintln!(
            "pkg-config failed with error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        exit(1);
    }
}

fn find_test_function(project: &Project) -> IndexMap<String, Vec<String>> {
    let mut test_functions = IndexMap::new();
    let re = Regex::new(r"void\s+(test_[^\(]+)").expect("Can't compile regex");
    for source_file in &project.source_files {
        if let Ok(contents) = fs::read_to_string(resolve_source_file_path(source_file, project)) {
            let mut functions = Vec::new();
            for cap in re.captures_iter(&contents) {
                functions.push(cap[1].to_string());
            }
            if !functions.is_empty() {
                test_functions.insert(source_file.clone(), functions);
            }
        }
    }
    test_functions
}

fn generate_test_main(project: &mut Project) {
    let test_functions = find_test_function(project);

    let mut s = String::new();
    _ = writeln!(s, "// This file is generated by bob, do not edit!");
    _ = writeln!(s, "#include <stdint.h>");
    _ = writeln!(s, "#include <CUnit/Basic.h>\n");
    for functions in test_functions.values() {
        for function in functions {
            _ = writeln!(s, "extern void {}(void);", function);
        }
    }
    _ = writeln!(s, "\nint main(void) {{");
    _ = writeln!(s, "    CU_initialize_registry();\n");
    for (source_file, functions) in &test_functions {
        let module_name_suite = format!(
            "{}_suite",
            source_file
                .trim_start_matches("$source_dir/")
                .trim_end_matches(".c")
        );
        _ = writeln!(
            s,
            "    CU_pSuite {} = CU_add_suite(\"{}\", 0, 0);",
            module_name_suite, source_file
        );
        for function in functions {
            _ = writeln!(
                s,
                "    CU_add_test({}, \"{}\", {});",
                module_name_suite, function, function
            );
        }
        _ = writeln!(s);
    }
    _ = writeln!(
        s,
        r#"    CU_basic_set_mode(CU_BRM_VERBOSE);
    CU_basic_run_tests();

    #define ANSI_COLOR_RED "\x1b[31m"
    #define ANSI_COLOR_GREEN "\x1b[32m"
    #define ANSI_COLOR_RESET "\x1b[0m"
    uint32_t number_of_failures = CU_get_number_of_failures();
    if (number_of_failures == 0) {{
        printf(ANSI_COLOR_GREEN "All tests passed!" ANSI_COLOR_RESET "\n");
    }} else {{
        printf(ANSI_COLOR_RED "%d tests failed!" ANSI_COLOR_RESET "\n", number_of_failures);
    }}

    CU_cleanup_registry();
    return CU_get_error();
}}"#
    );

    write_file_when_different(
        &format!(
            "{}/target/{}/src-gen/test_main.c",
            project.manifest_dir, project.profile
        ),
        &s,
    )
    .expect("Can't write src-gen/test_main.c");

    project
        .source_files
        .push("$source_gen_dir/test_main.c".to_string());
}

fn generate_info_plist(project: &Project, bundle: &BundleMetadata, extra_keys: Option<&str>) {
    let identifier = project
        .manifest
        .package
        .identifier
        .as_ref()
        .unwrap_or_else(|| {
            eprintln!("Identifier is required");
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
    _ = writeln!(s, r#"    <key>CFBundlePackageType</key>"#);
    _ = writeln!(s, r#"    <string>APPL</string>"#);
    _ = writeln!(s, r#"    <key>CFBundleName</key>"#);
    _ = writeln!(
        s,
        r#"    <string>{}</string>"#,
        project.manifest.package.name
    );
    _ = writeln!(s, r#"    <key>CFBundleDisplayName</key>"#);
    _ = writeln!(
        s,
        r#"    <string>{}</string>"#,
        project.manifest.package.name
    );
    _ = writeln!(s, r#"    <key>CFBundleIdentifier</key>"#);
    _ = writeln!(s, r#"    <string>{}</string>"#, identifier);
    _ = writeln!(s, r#"    <key>CFBundleVersion</key>"#);
    _ = writeln!(
        s,
        r#"    <string>{}</string>"#,
        project.manifest.package.version
    );
    _ = writeln!(s, r#"    <key>CFBundleShortVersionString</key>"#);
    _ = writeln!(
        s,
        r#"    <string>{}</string>"#,
        project.manifest.package.version
    );
    _ = writeln!(s, r#"    <key>CFBundleExecutable</key>"#);
    _ = writeln!(
        s,
        r#"    <string>{}</string>"#,
        project.manifest.package.name
    );
    _ = writeln!(s, r#"    <key>LSMinimumSystemVersion</key>"#);
    _ = writeln!(s, r#"    <string>11.0</string>"#,);
    if let Some(copyright) = &bundle.copyright {
        _ = writeln!(s, r#"    <key>NSHumanReadableCopyright</key>"#);
        _ = writeln!(s, r#"    <string>{}</string>"#, copyright);
    }
    if let Some(extra_keys) = extra_keys {
        _ = writeln!(s, "    {}", extra_keys);
    }
    _ = writeln!(s, r#"</dict>"#);
    _ = writeln!(s, r#"</plist>"#);

    write_file_when_different(
        &format!(
            "{}/target/{}/src-gen/Info.plist",
            project.manifest_dir, project.profile
        ),
        &s,
    )
    .expect("Can't write src-gen/Info.plist");
}
