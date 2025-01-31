/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::process::{exit, Command};

use indexmap::IndexMap;
use regex::Regex;

use crate::manifest::BundleMetadata;
use crate::{create_file_with_dirs, Profile, Project};

pub(crate) fn generate_cx_common(f: &mut impl Write, project: &Project) {
    if project.is_test {
        generate_test_main(project);
    }

    _ = writeln!(f, "objects_dir = $target_dir/$profile/objects");
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
}

pub(crate) fn generate_c(f: &mut impl Write, project: &Project) {
    let c_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".c"))
        .cloned()
        .collect::<Vec<String>>();
    _ = writeln!(f, "\n# Build C objects");
    _ = writeln!(
        f,
        "rule cc\n  command = gcc -c $cflags --std=c11 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = cc $in\n"
    );
    for source_file in &c_source_files {
        let object_file = format!("$objects_dir/{}", source_file.replace(".c", ".o"));
        _ = writeln!(f, "build {}: cc $source_dir/{}", object_file, source_file);
    }
    if project.is_test {
        _ = writeln!(
            f,
            "build $objects_dir/test_main.o: cc $target_dir/$profile/src_gen/test_main.c"
        );
    }
}

pub(crate) fn generate_cpp(f: &mut impl Write, project: &Project) {
    let cpp_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".cpp"))
        .cloned()
        .collect::<Vec<String>>();
    _ = writeln!(f, "\n# Build C++ objects");
    _ = writeln!(
            f,
            "rule cpp\n  command = g++ -c $cflags --std=c++17 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = cpp $in\n"
        );
    for source_file in &cpp_source_files {
        let object_file = format!("$objects_dir/{}", source_file.replace(".cpp", ".o"));
        _ = writeln!(f, "build {}: cpp $source_dir/{}", object_file, source_file);
    }
}

pub(crate) fn generate_objc(f: &mut impl Write, project: &Project) {
    let m_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".m"))
        .cloned()
        .collect::<Vec<String>>();
    _ = writeln!(f, "\n# Build Objective-C objects");
    _ = writeln!(
            f,
            "rule objc\n  command = gcc -x objective-c -c $cflags --std=c11 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = objc $in\n"
        );
    for source_file in &m_source_files {
        let object_file = format!("$objects_dir/{}", source_file.replace(".m", ".o"));
        _ = writeln!(f, "build {}: objc $source_dir/{}", object_file, source_file);
    }
}

pub(crate) fn generate_objcpp(f: &mut impl Write, project: &Project) {
    let mm_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".mm"))
        .cloned()
        .collect::<Vec<String>>();
    _ = writeln!(f, "\n# Build Objective-C++ objects");
    _ = writeln!(
            f,
            "rule objcpp\n  command = g++ -x objective-c++ -c $cflags --std=c++17 -MD -MF $out.d $in -o $out\n  depfile = $out.d\n  description = objcpp $in\n"
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

pub(crate) fn generate_ld(f: &mut impl Write, project: &Project) {
    let mut object_files = Vec::new();
    let mut contains_cpp = false;
    let mut contains_objc = false;
    if project.is_test {
        let test_functions = index_test_function(project);
        object_files.push("$objects_dir/test_main.o".to_string());
        for source_file in test_functions.keys() {
            object_files.push(format!(
                "$objects_dir/{}",
                source_file
                    .replace(".m", ".o")
                    .replace(".mm", ".o")
                    .replace(".cpp", ".o")
                    .replace(".c", ".o")
            ));
        }
    } else {
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
    }

    let mut ldflags = "".to_string();
    if project.profile == Profile::Release {
        ldflags.push_str(" -Os");
    }
    if contains_objc {
        ldflags.push_str(" -framework Foundation");
    }
    if project.is_test {
        ldflags.push(' ');
        ldflags.push_str(&pkg_config_libs("cunit"));
    }
    if let Some(build) = &project.manifest.build {
        if let Some(ldflags_extra) = &build.ldflags {
            ldflags.push(' ');
            ldflags.push_str(ldflags_extra);
        }
    }

    _ = writeln!(f, "\n# Link executable");
    _ = writeln!(f, "ldflags ={}\n", ldflags);
    _ = writeln!(
        f,
        "rule ld\n  command = {} {} $ldflags $in -o $out{}\n  description = ld $out\n",
        if contains_cpp { "g++" } else { "gcc" },
        match project.profile {
            Profile::Release => "-Os",
            _ => "-g",
        },
        match project.profile {
            Profile::Release => " && strip $out",
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

pub(crate) fn generate_bundle(f: &mut impl Write, project: &Project) {
    let bundle = &project
        .manifest
        .package
        .metadata
        .as_ref()
        .and_then(|m| m.bundle.as_ref())
        .expect("Should be some");

    // Write Info.plist
    generate_info_plist(project, bundle);

    // Copy Info.plist and executable
    _ = writeln!(f, "\n# Build macOS bundle");
    _ = writeln!(
        f,
        "rule cp\n  command = cp $in $out\n  description = cp $in\n"
    );
    #[cfg(windows)]
    let executable_file = "$target_dir/$profile/$name.exe";
    #[cfg(not(windows))]
    let executable_file = "$target_dir/$profile/$name";
    _ = writeln!(
        f,
        "build $target_dir/$profile/$name.app/Contents/MacOS/$name: cp {}",
        executable_file
    );
    _ = writeln!(
        f,
        "build $target_dir/$profile/$name.app/Contents/Info.plist: cp $target_dir/$profile/src_gen/Info.plist"
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
    let status = Command::new("open")
        .arg(format!(
            "{}/target/{}/{}.app",
            project.manifest_dir, project.profile, project.manifest.package.name
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

fn index_test_function(project: &Project) -> IndexMap<String, Vec<String>> {
    let mut test_functions = IndexMap::new();
    let re = Regex::new(r"void\s+(test_[^\(]+)").expect("Can't compile regex");
    for source_file in &project.source_files {
        let contents = fs::read_to_string(format!("{}/src/{}", project.manifest_dir, source_file))
            .unwrap_or_else(|_| panic!("Can't read source file: {}", source_file));
        let mut functions = Vec::new();
        for cap in re.captures_iter(&contents) {
            functions.push(cap[1].to_string());
        }
        if !functions.is_empty() {
            test_functions.insert(source_file.clone(), functions);
        }
    }
    test_functions
}

fn generate_test_main(project: &Project) {
    let test_functions = index_test_function(project);

    let mut s = String::new();
    _ = writeln!(s, "// This file is generated by bob, do not edit!");
    _ = writeln!(s, "#include <CUnit/Basic.h>\n");
    for functions in test_functions.values() {
        for function in functions {
            _ = writeln!(s, "extern void {}(void);", function);
        }
    }
    _ = writeln!(s, "\nint main(void) {{");
    _ = writeln!(s, "    CU_initialize_registry();\n");
    for (source_file, functions) in &test_functions {
        let module_name_suite = format!("{}_suite", source_file.replace(".c", ""));
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
    unsigned int number_of_failures = CU_get_number_of_failures();
    if (number_of_failures == 0) {{
        printf(ANSI_COLOR_GREEN "All tests passed!" ANSI_COLOR_RESET "\n");
    }} else {{
        printf(ANSI_COLOR_RED "%d tests failed!" ANSI_COLOR_RESET "\n", number_of_failures);
    }}

    CU_cleanup_registry();
    return CU_get_error();
}}"#
    );

    let file_path = format!(
        "{}/target/{}/src_gen/test_main.c",
        project.manifest_dir, project.profile
    );
    if let Ok(existing_contents) = fs::read_to_string(&file_path) {
        if existing_contents == s {
            return;
        }
    }
    let mut f = create_file_with_dirs(file_path).expect("Can't create test_main.c");
    f.write_all(s.as_bytes()).expect("Write test_main.c failed");
}

fn generate_info_plist(project: &Project, bundle: &BundleMetadata) {
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
    _ = writeln!(
        s,
        r#"    <string>{}</string>"#,
        project
            .manifest
            .package
            .identifier
            .as_ref()
            .unwrap_or_else(|| {
                eprintln!("Identifier is required");
                exit(1);
            })
    );
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
    _ = writeln!(s, r#"    <key>NSHumanReadableCopyright</key>"#);
    _ = writeln!(s, r#"    <string>{}</string>"#, bundle.copyright);
    _ = writeln!(s, r#"</dict>"#);
    _ = writeln!(s, r#"</plist>"#);

    let file_path = format!(
        "{}/target/{}/src_gen/Info.plist",
        project.manifest_dir, project.profile
    );
    if let Ok(existing_contents) = fs::read_to_string(&file_path) {
        if existing_contents == s {
            return;
        }
    }
    let mut f = create_file_with_dirs(file_path).expect("Can't create Info.plist");
    f.write_all(s.as_bytes()).expect("Write Info.plist failed");
}
