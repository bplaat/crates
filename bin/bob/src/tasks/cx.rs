/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, exit};

use indexmap::IndexMap;
use regex::Regex;

use crate::executor::Executor;
use crate::manifest::BundleMetadata;
use crate::utils::{index_files, write_file_when_different};
use crate::{Profile, Project};

// MARK: Cx tasks
struct CxVars {
    objects_dir: String,
    cflags: String,
    ldflags: String,
    cc: String,
    cxx: String,
    strip: String,
}

impl CxVars {
    fn new(project: &Project) -> Self {
        let objects_dir = format!("{}/{}/objects", project.target_dir, project.profile);

        // Cflags
        let mut cflags = match project.profile {
            Profile::Debug => "-g -DDEBUG".to_string(),
            Profile::Release => "-Os -DRELEASE".to_string(),
        };
        cflags.push_str(" -Wall -Wextra -Wpedantic -Werror");
        if project.is_test {
            cflags.push_str(&format!(" -DTEST {}", pkg_config_cflags("cunit")));
        }
        if !project.manifest.build.cflags.is_empty() {
            cflags.push(' ');
            cflags.push_str(&project.manifest.build.cflags);
        }

        // Ldflags
        let mut ldflags = String::new();
        if project.profile == Profile::Release {
            ldflags.push_str(" -Os");
        } else {
            ldflags.push_str(" -g");
        }
        if project
            .source_files
            .iter()
            .any(|p| p.ends_with(".m") || p.ends_with(".mm"))
        {
            ldflags.push_str(" -framework Foundation");
        }
        if project.is_test {
            ldflags.push(' ');
            ldflags.push_str(&pkg_config_libs("cunit"));
        }
        if !project.manifest.build.ldflags.is_empty() {
            ldflags.push(' ');
            ldflags.push_str(&project.manifest.build.ldflags);
        }

        // Use Clang on macOS and Windows, GCC elsewhere
        #[cfg(target_os = "macos")]
        let (cc, cxx, strip) = (
            "clang".to_string(),
            "clang++".to_string(),
            "strip".to_string(),
        );
        #[cfg(windows)]
        let (cc, cxx, strip) = (
            "clang".to_string(),
            "clang++".to_string(),
            "llvm-strip".to_string(),
        );
        #[cfg(not(any(target_os = "macos", windows)))]
        let (cc, cxx, strip) = ("gcc".to_string(), "g++".to_string(), "strip".to_string());

        Self {
            objects_dir,
            cflags,
            ldflags,
            cc,
            cxx,
            strip,
        }
    }
}

pub(crate) fn detect_c(project: &Project) -> bool {
    project.source_files.iter().any(|path| path.ends_with(".c"))
}

pub(crate) fn generate_c_tasks(project: &Project, executor: &mut Executor) {
    let vars = CxVars::new(project);
    let c_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".c"));
    for source_file in c_source_files {
        let object_file = format!(
            "{}/{}",
            vars.objects_dir,
            source_file.trim_start_matches("src/").replace(".c", ".o"),
        );
        // FIXME: Add support for depfiles -MD -MF $out.d
        executor.add_task(
            format!(
                "{} -c {} --std=c11 {} -o {}",
                vars.cc, vars.cflags, source_file, object_file
            ),
            vec![source_file.clone()],
            vec![object_file],
        );
    }
}

pub(crate) fn detect_cpp(project: &Project) -> bool {
    project
        .source_files
        .iter()
        .any(|path| path.ends_with(".cpp"))
}

pub(crate) fn generate_cpp_tasks(project: &Project, executor: &mut Executor) {
    let vars = CxVars::new(project);
    let cpp_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".cpp"));
    for source_file in cpp_source_files {
        let object_file = format!(
            "{}/{}",
            vars.objects_dir,
            source_file.trim_start_matches("src/").replace(".cpp", ".o"),
        );
        // FIXME: Add support for depfiles -MD -MF $out.d
        executor.add_task(
            format!(
                "{} -c {} --std=c++17 {} -o {}",
                vars.cxx, vars.cflags, source_file, object_file
            ),
            vec![source_file.clone()],
            vec![object_file],
        );
    }
}

pub(crate) fn detect_objc(project: &Project) -> bool {
    project.source_files.iter().any(|path| path.ends_with(".m"))
}

pub(crate) fn generate_objc_tasks(project: &Project, executor: &mut Executor) {
    let vars = CxVars::new(project);
    let m_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".m"));
    for source_file in m_source_files {
        let object_file = format!(
            "{}/{}",
            vars.objects_dir,
            source_file.trim_start_matches("src/").replace(".m", ".o"),
        );
        // FIXME: Add support for depfiles -MD -MF $out.d
        executor.add_task(
            format!(
                "{} -x objective-c -c {} --std=c11 {} -o {}",
                vars.cc, vars.cflags, source_file, object_file
            ),
            vec![source_file.clone()],
            vec![object_file],
        );
    }
}

pub(crate) fn detect_objcpp(project: &Project) -> bool {
    project
        .source_files
        .iter()
        .any(|path| path.ends_with(".mm"))
}

pub(crate) fn generate_objcpp_tasks(project: &Project, executor: &mut Executor) {
    let vars = CxVars::new(project);
    let mm_source_files = project
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".mm"));
    for source_file in mm_source_files {
        let object_file = format!(
            "{}/{}",
            vars.objects_dir,
            source_file.trim_start_matches("src/").replace(".mm", ".o"),
        );
        // FIXME: Add support for depfiles -MD -MF $out.d
        executor.add_task(
            format!(
                "{} -x objective-c++ -c {} --std=c++17 {} -o {}",
                vars.cxx, vars.cflags, source_file, object_file
            ),
            vec![source_file.clone()],
            vec![object_file],
        );
    }
}

pub(crate) fn detect_cx(project: &Project) -> bool {
    detect_c(project) || detect_cpp(project) || detect_objc(project) || detect_objcpp(project)
}

pub(crate) fn generate_ld_tasks(project: &Project, executor: &mut Executor) {
    let vars = CxVars::new(project);

    let mut object_files = Vec::new();
    let mut contains_cpp = false;
    if project.is_test {
        let test_functions = find_test_function(project);
        for source_file in test_functions.keys() {
            object_files.push(format!(
                "{}/{}",
                vars.objects_dir,
                source_file
                    .trim_start_matches("src/")
                    .replace(".mm", ".o")
                    .replace(".m", ".o")
                    .replace(".cpp", ".o")
                    .replace(".c", ".o"),
            ));
        }
    } else {
        for source_file in &project.source_files {
            let source_file = source_file.trim_start_matches("src/");
            if source_file.ends_with(".c") {
                object_files.push(format!(
                    "{}/{}",
                    vars.objects_dir,
                    source_file.replace(".c", ".o")
                ));
            }
            if source_file.ends_with(".cpp") {
                object_files.push(format!(
                    "{}/{}",
                    vars.objects_dir,
                    source_file.replace(".cpp", ".o")
                ));
                contains_cpp = true;
            }
            if source_file.ends_with(".m") {
                object_files.push(format!(
                    "{}/{}",
                    vars.objects_dir,
                    source_file.replace(".m", ".o")
                ));
            }
            if source_file.ends_with(".mm") {
                object_files.push(format!(
                    "{}/{}",
                    vars.objects_dir,
                    source_file.replace(".mm", ".o")
                ));
                contains_cpp = true;
            }
        }
    }

    let executable_file = if project.is_test {
        format!(
            "{}/{}/test_{}",
            project.target_dir, project.profile, project.manifest.package.name
        )
    } else {
        format!(
            "{}/{}/{}",
            project.target_dir, project.profile, project.manifest.package.name
        )
    };
    #[cfg(windows)]
    let ext = ".exe";
    #[cfg(not(windows))]
    let ext = "";
    if project.profile == Profile::Release {
        let unstripped_path = format!("{}-unstripped{}", executable_file, ext);
        let stripped_path = format!("{}{}", executable_file, ext);
        executor.add_task(
            format!(
                "{} {} {} -o {}",
                if contains_cpp { vars.cxx } else { vars.cc },
                vars.ldflags,
                object_files.join(" "),
                unstripped_path,
            ),
            object_files.clone(),
            vec![unstripped_path.clone()],
        );
        executor.add_task(
            format!("{} {} -o {}", vars.strip, unstripped_path, stripped_path),
            vec![unstripped_path.clone()],
            vec![stripped_path.clone()],
        );
    } else {
        let out_path = format!("{}{}", executable_file, ext);
        executor.add_task(
            format!(
                "{} {} {} -o {}",
                if contains_cpp { vars.cxx } else { vars.cc },
                vars.ldflags,
                object_files.join(" "),
                out_path,
            ),
            object_files.clone(),
            vec![out_path.clone()],
        );
    }
}

pub(crate) fn run_ld(project: &Project) {
    #[cfg(windows)]
    let ext = ".exe";
    #[cfg(not(windows))]
    let ext = "";
    let status = Command::new(format!(
        "{}/{}/{}{}",
        project.target_dir, project.profile, project.manifest.package.name, ext
    ))
    .status()
    .expect("Failed to execute executable");
    exit(status.code().unwrap_or(1));
}

pub(crate) fn run_ld_tests(project: &Project) {
    #[cfg(windows)]
    let ext = ".exe";
    #[cfg(not(windows))]
    let ext = "";
    let status = Command::new(format!(
        "{}/{}/test_{}{}",
        project.target_dir, project.profile, project.manifest.package.name, ext
    ))
    .status()
    .expect("Failed to execute executable");
    exit(status.code().unwrap_or(1));
}

// MARK: Bundle tasks
pub(crate) fn detect_bundle(project: &Project) -> bool {
    project.manifest.package.metadata.bundle.is_some()
}

pub(crate) fn generate_bundle_tasks(project: &Project, executor: &mut Executor) {
    let bundle = &project
        .manifest
        .package
        .metadata
        .bundle
        .as_ref()
        .expect("Should be some");

    // Write Info.plist
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
    generate_info_plist(project, bundle, extra_keys.as_deref());

    // Copy Info.plist
    executor.add_task(
        format!(
            "cp {}/{}/src-gen/Info.plist {}/{}/{}.app/Contents/Info.plist",
            project.target_dir,
            project.profile,
            project.target_dir,
            project.profile,
            project.manifest.package.name
        ),
        vec![format!(
            "{}/{}/src-gen/Info.plist",
            project.target_dir, project.profile
        )],
        vec![format!(
            "{}/{}/{}.app/Contents/Info.plist",
            project.target_dir, project.profile, project.manifest.package.name
        )],
    );

    // Copy resources
    let resources_dir = "Resources/";
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
            executor.add_task(
                format!(
                    "cp Resources/{} {}/{}/{}.app/Contents/Resources/{}",
                    resource_file,
                    project.target_dir,
                    project.profile,
                    project.manifest.package.name,
                    resource_file
                ),
                vec![format!("Resources/{}", resource_file)],
                vec![format!(
                    "{}/{}/{}.app/Contents/Resources/{}",
                    project.target_dir,
                    project.profile,
                    project.manifest.package.name,
                    resource_file
                )],
            );
        }
    }

    // Compile iconset
    if let Some(iconset) = &bundle.iconset {
        let icon_path = PathBuf::from(iconset);
        let icon_name = icon_path
            .file_stem()
            .expect("Invalid iconset path")
            .to_str()
            .expect("Invalid UTF-8 sequence");
        executor.add_task(
            format!(
                "iconutil -c icns {} -o {}/{}/{}.app/Contents/Resources/{}.icns",
                iconset,
                project.target_dir,
                project.profile,
                project.manifest.package.name,
                icon_name
            ),
            vec![format!("{}/{}", project.target_dir, iconset)],
            vec![format!(
                "{}/{}/{}.app/Contents/Resources/{}.icns",
                project.target_dir, project.profile, project.manifest.package.name, icon_name
            )],
        );
    }

    // Copy executable
    executor.add_task(
        format!(
            "cp {}/{}/{} {}/{}/{}.app/Contents/MacOS/{}",
            project.target_dir,
            project.profile,
            project.manifest.package.name,
            project.target_dir,
            project.profile,
            project.manifest.package.name,
            project.manifest.package.name
        ),
        vec![format!(
            "{}/{}/{}",
            project.target_dir, project.profile, project.manifest.package.name
        )],
        vec![format!(
            "{}/{}/{}.app/Contents/MacOS/{}",
            project.target_dir,
            project.profile,
            project.manifest.package.name,
            project.manifest.package.name
        )],
    );
}

pub(crate) fn run_bundle(project: &Project) {
    let status = Command::new(format!(
        "{}/{}/{}.app/Contents/MacOS/{}",
        project.target_dir,
        project.profile,
        project.manifest.package.name,
        project.manifest.package.name
    ))
    .status()
    .expect("Failed to execute executable");
    exit(status.code().unwrap_or(1));
}

// MARK: Utils
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
        if let Ok(contents) = fs::read_to_string(source_file) {
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

pub(crate) fn generate_test_main(project: &mut Project) {
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
            "{}/{}/src-gen/test_main.c",
            project.target_dir, project.profile
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
    _ = writeln!(s, r#"	<key>CFBundlePackageType</key>"#);
    _ = writeln!(s, r#"	<string>APPL</string>"#);
    _ = writeln!(s, r#"	<key>CFBundleName</key>"#);
    _ = writeln!(s, r#"	<string>{}</string>"#, project.manifest.package.name);
    _ = writeln!(s, r#"	<key>CFBundleDisplayName</key>"#);
    _ = writeln!(s, r#"	<string>{}</string>"#, project.manifest.package.name);
    _ = writeln!(s, r#"	<key>CFBundleIdentifier</key>"#);
    _ = writeln!(s, r#"	<string>{}</string>"#, identifier);
    _ = writeln!(s, r#"	<key>CFBundleVersion</key>"#);
    _ = writeln!(
        s,
        r#"	<string>{}</string>"#,
        project.manifest.package.version
    );
    _ = writeln!(s, r#"	<key>CFBundleShortVersionString</key>"#);
    _ = writeln!(
        s,
        r#"	<string>{}</string>"#,
        project.manifest.package.version
    );
    _ = writeln!(s, r#"	<key>CFBundleExecutable</key>"#);
    _ = writeln!(s, r#"	<string>{}</string>"#, project.manifest.package.name);
    _ = writeln!(s, r#"	<key>LSMinimumSystemVersion</key>"#);
    _ = writeln!(s, r#"	<string>11.0</string>"#,);
    if let Some(copyright) = &bundle.copyright {
        _ = writeln!(s, r#"	<key>NSHumanReadableCopyright</key>"#);
        _ = writeln!(s, r#"	<string>{}</string>"#, copyright);
    }
    if let Some(extra_keys) = extra_keys {
        _ = writeln!(s, "{}", extra_keys);
    }
    _ = writeln!(s, r#"</dict>"#);
    _ = writeln!(s, r#"</plist>"#);

    write_file_when_different(
        &format!(
            "{}/{}/src-gen/Info.plist",
            project.target_dir, project.profile
        ),
        &s,
    )
    .expect("Can't write src-gen/Info.plist");
}
