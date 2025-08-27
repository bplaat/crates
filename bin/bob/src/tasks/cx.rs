/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::Write;
use std::fs::{self};
use std::process::{Command, exit};

use regex::Regex;

use crate::executor::Executor;
use crate::manifest::{Dependency, LibraryType};
use crate::utils::write_file_when_different;
use crate::{Bobje, PackageType, Profile};

// MARK: Cx vars
struct CxVars {
    asflags: String,
    cflags: String,
    ldflags: String,
    libs: String,
    cc: String,
    cxx: String,
    ld: String,
    ar: String,
    strip: String,
}

impl CxVars {
    fn new(bobje: &Bobje) -> Self {
        let use_llvm = cfg!(any(target_os = "macos", windows));

        // Asflags
        let mut asflags = if !bobje.manifest.build.asflags.is_empty() {
            bobje.manifest.build.asflags.clone()
        } else {
            String::new()
        };
        if use_llvm && let Some(target) = &bobje.target {
            asflags.push_str(&format!(" --target={target}"));
        }

        // Cflags
        let mut cflags = match bobje.profile {
            Profile::Debug => "-g -DDEBUG".to_string(),
            Profile::Release => "-Os -DRELEASE".to_string(),
            Profile::Test => "-g -DDEBUG -DTEST".to_string(),
        };
        cflags.push_str(&format!(
            " -Wall -Wextra -Wpedantic -Werror -I{}/include",
            bobje.out_dir_with_target()
        ));
        if use_llvm && let Some(target) = &bobje.target {
            cflags.push_str(&format!(" --target={target}"));
        }
        if !bobje.manifest.build.cflags.is_empty() {
            cflags.push(' ');
            cflags.push_str(&bobje.manifest.build.cflags);
        }
        for dep in bobje.manifest.dependencies.values() {
            if let Dependency::PkgConfig { pkg_config } = &dep {
                cflags.push_str(&format!(" {}", pkg_config_cflags(pkg_config)));
            }
        }

        // Ldflags
        let mut ldflags = if !bobje.manifest.build.ldflags.is_empty() {
            bobje.manifest.build.ldflags.clone()
        } else {
            String::new()
        };
        if let Some(entry) = &bobje.manifest.build.entry {
            ldflags.push_str(&format!(" -e {entry}"));
            if !cfg!(target_os = "macos") {
                ldflags.push_str(" -nostartfiles");
            }
        }

        // Libs
        let mut libs = format!("-L{}", bobje.out_dir_with_target());
        if cfg!(target_os = "macos") {
            let sdk_path = Command::new("xcrun")
                .arg("--show-sdk-path")
                .output()
                .map(|output| {
                    if output.status.success() {
                        String::from_utf8_lossy(&output.stdout).trim().to_string()
                    } else {
                        panic!("Can't find macOS SDK path");
                    }
                })
                .unwrap_or_default();

            libs.push_str(&format!(" -L{sdk_path}/usr/lib"));

            if bobje
                .manifest
                .dependencies
                .values()
                .any(|dep| matches!(dep, Dependency::Framework { .. }))
            {
                libs.push_str(&format!(" -F{sdk_path}/System/Library/Frameworks"));
            }
            for dep in bobje.manifest.dependencies.values() {
                if let Dependency::Framework { framework } = &dep {
                    libs.push_str(&format!(" -framework {framework}"));
                }
            }
        }

        for dep in bobje.manifest.dependencies.values() {
            if let Dependency::Library { library } = &dep {
                libs.push_str(&format!(" -l{library}"));
            }
            if let Dependency::PkgConfig { pkg_config } = &dep {
                libs.push_str(&format!(" {}", pkg_config_libs(pkg_config)));
            }
        }

        // Find correct toolchain
        let (cc, cxx, ld, ar, strip) = if use_llvm {
            (
                "clang".to_string(),
                "clang++".to_string(),
                "ld".to_string(),
                "ar".to_string(),
                "strip".to_string(),
            )
        } else if let Some(target) = &bobje.target {
            let prefix = target.replace("-unknown-linux-gnu", "-linux-gnu");
            if cfg!(target_arch = "x86_64") && target == "x86_64-linux-gnu"
                || cfg!(target_arch = "aarch64") && target == "aarch64-linux-gnu"
            {
                (
                    "gcc".to_string(),
                    "g++".to_string(),
                    "ld".to_string(),
                    "ar".to_string(),
                    "strip".to_string(),
                )
            } else {
                (
                    format!("{prefix}-gcc"),
                    format!("{prefix}-g++"),
                    format!("{prefix}-ld"),
                    format!("{prefix}-ar"),
                    format!("{prefix}-strip"),
                )
            }
        } else {
            (
                "gcc".to_string(),
                "g++".to_string(),
                "ld".to_string(),
                "ar".to_string(),
                "strip".to_string(),
            )
        };

        Self {
            asflags,
            cflags,
            ldflags,
            libs,
            cc,
            cxx,
            ld,
            ar,
            strip,
        }
    }
}

// MARK: Copy headers
pub(crate) fn copy_cx_headers(bobje: &Bobje, _executor: &mut Executor) {
    for source_file in &bobje.source_files {
        if source_file.ends_with(".h")
            || source_file.ends_with(".hh")
            || source_file.ends_with(".hpp")
        {
            let dest = format!(
                "{}/include/{}/{}",
                bobje.out_dir_with_target(),
                bobje.name,
                source_file
                    .split("src/")
                    .nth(1)
                    .or_else(|| source_file.split("src-gen/").nth(1))
                    .expect("Should be some")
                    .replace('\\', "/")
            );
            fs::create_dir_all(dest.rsplit_once('/').expect("Should be some").0)
                .expect("Failed to create include directory");
            fs::copy(source_file, dest).expect("Failed to copy header file");
        }
    }
}

// MARK: Asm tasks
pub(crate) fn detect_asm(source_files: &[String]) -> bool {
    source_files
        .iter()
        .any(|path| path.ends_with(".s") || path.ends_with(".S"))
}

pub(crate) fn generate_asm_tasks(bobje: &Bobje, executor: &mut Executor) {
    let vars = CxVars::new(bobje);
    let asm_source_files = bobje
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".s") || source_file.ends_with(".S"));
    for source_file in asm_source_files {
        let object_file = get_object_path(bobje, source_file);
        executor.add_task_cmd(
            format!(
                "{} {} -c {} -o {}",
                vars.cc, vars.asflags, source_file, object_file
            ),
            vec![source_file.clone()],
            vec![object_file],
        );
    }
}

// MARK: C tasks
pub(crate) fn detect_c(source_files: &[String]) -> bool {
    source_files.iter().any(|path| path.ends_with(".c"))
}

pub(crate) fn generate_c_tasks(bobje: &Bobje, executor: &mut Executor) {
    let vars = CxVars::new(bobje);
    let c_source_files = bobje
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".c"));
    for source_file in c_source_files {
        let object_file = get_object_path(bobje, source_file);
        // FIXME: Add support for depfiles -MD -MF $out.d
        executor.add_task_cmd(
            format!(
                "{} -c {} --std=c11 {} -o {}",
                vars.cc, vars.cflags, source_file, object_file
            ),
            vec![source_file.clone()],
            vec![object_file],
        );
    }
}

// MARK: C++ tasks
pub(crate) fn detect_cpp(source_files: &[String]) -> bool {
    source_files.iter().any(|path| path.ends_with(".cpp"))
}

pub(crate) fn generate_cpp_tasks(bobje: &Bobje, executor: &mut Executor) {
    let vars = CxVars::new(bobje);
    let cpp_source_files = bobje
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".cpp"));
    for source_file in cpp_source_files {
        let object_file = get_object_path(bobje, source_file);
        // FIXME: Add support for depfiles -MD -MF $out.d
        executor.add_task_cmd(
            format!(
                "{} -c {} --std=c++17 {} -o {}",
                vars.cxx, vars.cflags, source_file, object_file
            ),
            vec![source_file.clone()],
            vec![object_file],
        );
    }
}

// MARK: Objective-C tasks
pub(crate) fn detect_objc(source_files: &[String]) -> bool {
    source_files.iter().any(|path| path.ends_with(".m"))
}

pub(crate) fn generate_objc_tasks(bobje: &Bobje, executor: &mut Executor) {
    let vars = CxVars::new(bobje);
    let m_source_files = bobje
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".m"));
    for source_file in m_source_files {
        let object_file = get_object_path(bobje, source_file);
        // FIXME: Add support for depfiles -MD -MF $out.d
        executor.add_task_cmd(
            format!(
                "{} -x objective-c -c {} --std=c11 {} -o {}",
                vars.cc, vars.cflags, source_file, object_file
            ),
            vec![source_file.clone()],
            vec![object_file],
        );
    }
}

// MARK: Objective-C++ tasks
pub(crate) fn detect_objcpp(source_files: &[String]) -> bool {
    source_files.iter().any(|path| path.ends_with(".mm"))
}

pub(crate) fn generate_objcpp_tasks(bobje: &Bobje, executor: &mut Executor) {
    let vars = CxVars::new(bobje);
    let mm_source_files = bobje
        .source_files
        .iter()
        .filter(|source_file| source_file.ends_with(".mm"));
    for source_file in mm_source_files {
        let object_file = get_object_path(bobje, source_file);
        // FIXME: Add support for depfiles -MD -MF $out.d
        executor.add_task_cmd(
            format!(
                "{} -x objective-c++ -c {} --std=c++17 {} -o {}",
                vars.cxx, vars.cflags, source_file, object_file
            ),
            vec![source_file.clone()],
            vec![object_file],
        );
    }
}

// MARK: Linker tasks
pub(crate) fn detect_cx(source_files: &[String]) -> bool {
    detect_asm(source_files)
        || detect_c(source_files)
        || detect_cpp(source_files)
        || detect_objc(source_files)
        || detect_objcpp(source_files)
}

pub(crate) fn generate_ld_tasks(bobje: &Bobje, executor: &mut Executor) {
    let vars = CxVars::new(bobje);

    let dylib_ext = if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(windows) {
        "dll"
    } else {
        "so"
    };

    // Gather inputs
    let mut inputs = Vec::new();
    let mut contains_cpp = false;
    for source_file in &bobje.source_files {
        if source_file.ends_with(".s")
            || source_file.ends_with(".S")
            || source_file.ends_with(".c")
            || source_file.ends_with(".cpp")
            || source_file.ends_with(".m")
            || source_file.ends_with(".mm")
        {
            inputs.push(get_object_path(bobje, source_file));
        }
        if source_file.ends_with(".cpp") || source_file.ends_with(".mm") {
            contains_cpp = true;
        }
    }

    // Add dependencies
    fn add_dependency_inputs(
        bobje: &Bobje,
        dylib_ext: &str,
        inputs: &mut Vec<String>,
        contains_cpp: &mut bool,
    ) {
        for dependency_bobje in bobje.dependencies.values() {
            add_dependency_inputs(dependency_bobje, dylib_ext, inputs, contains_cpp);
        }
        for source_file in &bobje.source_files {
            if source_file.ends_with(".cpp") || source_file.ends_with(".mm") {
                *contains_cpp = true;
            }
        }

        if let PackageType::Library { r#type } = bobje.r#type {
            inputs.push(format!(
                "{}/lib{}.{}",
                bobje.out_dir_with_target(),
                bobje.name,
                match r#type {
                    LibraryType::Static => "a",
                    LibraryType::Dynamic => dylib_ext,
                }
            ));
        }
    }
    for dependency_bobje in bobje.dependencies.values() {
        add_dependency_inputs(dependency_bobje, dylib_ext, &mut inputs, &mut contains_cpp);
    }

    // Link library
    let linker = if cfg!(target_os = "macos") {
        vars.ld
    } else if contains_cpp {
        vars.cxx
    } else {
        vars.cc
    };
    if let PackageType::Library { r#type } = bobje.r#type {
        let input_objects = inputs
            .iter()
            .filter(|f| f.ends_with(".o"))
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        match r#type {
            LibraryType::Static => {
                let staticlib_path = format!("{}/lib{}.a", bobje.out_dir_with_target(), bobje.name);
                executor.add_task_cmd(
                    format!("{} rc {} {}", vars.ar, staticlib_path, input_objects),
                    inputs.clone(),
                    vec![staticlib_path],
                );
            }
            LibraryType::Dynamic => {
                let dylib_path = format!(
                    "{}/lib{}.{}",
                    bobje.out_dir_with_target(),
                    bobje.name,
                    dylib_ext
                );
                executor.add_task_cmd(
                    format!(
                        "{} {} {} {} {} {} -o {}",
                        linker,
                        if cfg!(target_os = "macos") {
                            "-dylib"
                        } else {
                            "-shared"
                        },
                        vars.ldflags,
                        input_objects,
                        vars.libs,
                        if cfg!(target_os = "macos") {
                            format!("-install_name @rpath/lib{}.{}", bobje.name, dylib_ext)
                        } else {
                            String::new()
                        },
                        dylib_path,
                    ),
                    inputs.clone(),
                    vec![dylib_path],
                );
            }
        }
    }

    // Link executable
    if bobje.r#type.is_binary() {
        let executable_file = format!("{}/{}", bobje.out_dir_with_target(), bobje.name);
        let ext = if cfg!(windows) { ".exe" } else { "" };

        let input_objects = inputs
            .iter()
            .filter(|f| f.ends_with(".o") || f.ends_with(".a"))
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        let mut libs = vars.libs.clone();
        for input in &inputs {
            if input.ends_with(dylib_ext) {
                libs.push_str(&format!(
                    " -l{}",
                    input
                        .rsplit_once('/')
                        .expect("Failed to split")
                        .1
                        .strip_prefix("lib")
                        .expect("Failed to strip prefix")
                        .strip_suffix(&format!(".{dylib_ext}"))
                        .expect("Failed to strip suffix")
                ));
            }
        }

        if bobje.profile == Profile::Release {
            let unstripped_path = format!("{executable_file}-unstripped{ext}");
            let stripped_path = format!("{executable_file}{ext}");
            executor.add_task_cmd(
                format!(
                    "{} {} {} {} -rpath @executable_path -o {}",
                    linker, vars.ldflags, input_objects, libs, unstripped_path
                ),
                inputs.clone(),
                vec![unstripped_path.clone()],
            );
            executor.add_task_cmd(
                format!("{} {} -o {}", vars.strip, unstripped_path, stripped_path),
                vec![unstripped_path],
                vec![stripped_path],
            );
        } else {
            let out_path = format!("{executable_file}{ext}");
            executor.add_task_cmd(
                format!(
                    "{} {} {} {} -rpath @executable_path -o {}",
                    linker, vars.ldflags, input_objects, libs, out_path
                ),
                inputs.clone(),
                vec![out_path],
            );
        }
    }
}

pub(crate) fn generate_ld_cunit_tests(bobje: &Bobje, executor: &mut Executor) {
    let vars = CxVars::new(bobje);

    // Gather inputs
    let mut inputs = Vec::new();
    let mut contains_cpp = false;
    let test_functions = find_test_functions(bobje);
    for test_function in &test_functions {
        inputs.push(get_object_path(bobje, &test_function.source_file));
        if test_function.source_file.ends_with(".cpp") || test_function.source_file.ends_with(".mm")
        {
            contains_cpp = true;
        }
    }

    // Link test executable
    let executable_file = format!("{}/test_{}", bobje.out_dir_with_target(), bobje.name);
    let ext = if cfg!(windows) { ".exe" } else { "" };
    let out_path = format!("{executable_file}{ext}");
    executor.add_task_cmd(
        format!(
            "{} {} {} {} -o {}",
            if cfg!(target_os = "macos") {
                vars.ld
            } else if contains_cpp {
                vars.cxx
            } else {
                vars.cc
            },
            vars.ldflags,
            inputs.join(" "),
            vars.libs,
            out_path,
        ),
        inputs,
        vec![out_path],
    );
}

pub(crate) fn run_ld(bobje: &Bobje) -> ! {
    let ext = if cfg!(windows) { ".exe" } else { "" };
    let status = Command::new(format!(
        "{}/{}{}",
        bobje.out_dir_with_target(),
        bobje.name,
        ext
    ))
    .status()
    .expect("Failed to execute executable");
    exit(status.code().unwrap_or(1))
}

pub(crate) fn run_ld_cunit_tests(bobje: &Bobje) -> ! {
    let ext = if cfg!(windows) { ".exe" } else { "" };
    let status = Command::new(format!(
        "{}/test_{}{}",
        bobje.out_dir_with_target(),
        bobje.name,
        ext
    ))
    .status()
    .expect("Failed to execute executable");
    exit(status.code().unwrap_or(1))
}

// MARK: Utils
fn get_object_path(bobje: &Bobje, source_file: &str) -> String {
    format!(
        "{}/objects/{}/{}",
        bobje.out_dir_with_target(),
        bobje.name,
        source_file
            .split("src/")
            .nth(1)
            .or_else(|| source_file.split("src-gen/").nth(1))
            .expect("Should be some")
            .replace(".s", ".o")
            .replace(".S", ".o")
            .replace(".cpp", ".o")
            .replace(".c", ".o")
            .replace(".mm", ".o")
            .replace(".m", ".o")
    )
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

struct TestFunction {
    source_file: String,
    functions: Vec<String>,
}

fn find_test_functions(bobje: &Bobje) -> Vec<TestFunction> {
    let mut test_functions = Vec::new();
    let re = Regex::new(r"void\s+(test_[^\(]+)").expect("Can't compile regex");
    for source_file in &bobje.source_files {
        if let Ok(contents) = fs::read_to_string(source_file) {
            let mut functions = Vec::new();
            for cap in re.captures_iter(&contents) {
                functions.push(cap[1].to_string());
            }
            if !functions.is_empty() {
                test_functions.push(TestFunction {
                    source_file: source_file.to_string(),
                    functions,
                });
            }
        }
    }
    test_functions
}

pub(crate) fn generate_cx_test_main(bobje: &mut Bobje) {
    let test_functions = find_test_functions(bobje);

    let mut s = String::new();
    _ = writeln!(s, "// This file is generated by bob, do not edit!");
    _ = writeln!(s, "\n#include <CUnit/Basic.h>\n");
    for test_function in &test_functions {
        for function in &test_function.functions {
            _ = writeln!(s, "extern void {function}(void);");
        }
    }
    _ = writeln!(s, "\nint main(void) {{");
    _ = writeln!(s, "    CU_initialize_registry();\n");
    for test_function in &test_functions {
        let module_name_suite = format!(
            "{}_suite",
            test_function
                .source_file
                .split("src/")
                .nth(1)
                .expect("Should be some")
                .trim_end_matches(".s")
                .trim_end_matches(".S")
                .trim_end_matches(".c")
                .trim_end_matches(".cpp")
                .trim_end_matches(".m")
                .trim_end_matches(".mm")
                .replace(['/', '\\'], "_")
        );
        _ = writeln!(
            s,
            "    CU_pSuite {} = CU_add_suite(\"{}\", 0, 0);",
            module_name_suite, test_function.source_file
        );
        for function in &test_function.functions {
            _ = writeln!(
                s,
                "    CU_add_test({module_name_suite}, \"{function}\", {function});"
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
    int number_of_failures = CU_get_number_of_failures();
    if (number_of_failures == 0) {{
        printf(ANSI_COLOR_GREEN "All tests passed!" ANSI_COLOR_RESET "\n");
    }} else {{
        printf(ANSI_COLOR_RED "%d tests failed!" ANSI_COLOR_RESET "\n", number_of_failures);
    }}

    CU_cleanup_registry();
    return CU_get_error();
}}"#
    );

    let dest = format!("{}/src-gen/test_main.c", bobje.out_dir_with_target());
    write_file_when_different(&dest, &s).expect("Can't write src-gen/test_main.c");
    bobje.source_files.push(dest);
}
