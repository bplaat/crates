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
use crate::utils::write_file_when_different;
use crate::{Bobje, Profile};

// MARK: Cx vars
struct CxVars {
    cflags: String,
    ldflags: String,
    cc: String,
    cxx: String,
    ar: String,
    strip: String,
}

impl CxVars {
    fn new(bobje: &Bobje) -> Self {
        // Cflags
        let include_path = format!("{}/include", bobje.out_dir_with_target());
        let mut cflags = match bobje.profile {
            Profile::Debug => "-g -DDEBUG".to_string(),
            Profile::Release => "-Os -DRELEASE".to_string(),
        };
        cflags.push_str(&format!(
            " -Wall -Wextra -Wpedantic -Werror -I{include_path}"
        ));
        if let Some(target) = &bobje.target {
            cflags.push_str(&format!(" --target={target}"));
        }
        if bobje.is_test {
            cflags.push_str(&format!(" -DTEST {}", pkg_config_cflags("cunit")));
        }
        if !bobje.manifest.build.cflags.is_empty() {
            cflags.push(' ');
            cflags.push_str(&bobje.manifest.build.cflags);
        }

        // Ldflags
        let mut ldflags = String::new();
        if bobje.profile == Profile::Release {
            ldflags.push_str(" -Os");
        } else {
            ldflags.push_str(" -g");
        }
        if bobje
            .source_files
            .iter()
            .any(|p| p.ends_with(".m") || p.ends_with(".mm"))
        {
            ldflags.push_str(" -framework Foundation");
        }
        if let Some(target) = &bobje.target {
            ldflags.push_str(&format!(" --target={target}"));
        }
        if !bobje.manifest.build.ldflags.is_empty() {
            ldflags.push(' ');
            ldflags.push_str(&bobje.manifest.build.ldflags);
        }

        // Use Clang on macOS and Windows, GCC elsewhere
        #[cfg(target_os = "macos")]
        let (cc, cxx, ar, strip) = (
            "clang".to_string(),
            "clang++".to_string(),
            "ar".to_string(),
            "strip".to_string(),
        );
        #[cfg(windows)]
        let (cc, cxx, ar, strip) = (
            "clang".to_string(),
            "clang++".to_string(),
            "llvm-ar".to_string(),
            "llvm-strip".to_string(),
        );
        #[cfg(not(any(target_os = "macos", windows)))]
        let (cc, cxx, ar, strip) = (
            "gcc".to_string(),
            "g++".to_string(),
            "ar".to_string(),
            "strip".to_string(),
        );

        Self {
            cflags,
            ldflags,
            cc,
            cxx,
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
                    .expect("Should be some")
                    .replace('\\', "/")
            );
            fs::create_dir_all(dest.rsplit_once('/').expect("Should be some").0)
                .expect("Failed to create include directory");
            fs::copy(source_file, dest).expect("Failed to copy header file");
        }
    }
}

// MARK: C tasks
pub(crate) fn detect_c(bobje: &Bobje) -> bool {
    bobje.source_files.iter().any(|path| path.ends_with(".c"))
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
pub(crate) fn detect_cpp(bobje: &Bobje) -> bool {
    bobje.source_files.iter().any(|path| path.ends_with(".cpp"))
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
pub(crate) fn detect_objc(bobje: &Bobje) -> bool {
    bobje.source_files.iter().any(|path| path.ends_with(".m"))
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
pub(crate) fn detect_objcpp(bobje: &Bobje) -> bool {
    bobje.source_files.iter().any(|path| path.ends_with(".mm"))
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
pub(crate) fn detect_cx(bobje: &Bobje) -> bool {
    detect_c(bobje) || detect_cpp(bobje) || detect_objc(bobje) || detect_objcpp(bobje)
}

pub(crate) fn generate_ld_tasks(bobje: &Bobje, executor: &mut Executor) {
    let vars = CxVars::new(bobje);

    // Gather inputs
    let mut inputs = Vec::new();
    let mut contains_cpp = false;
    if bobje.is_test {
        let test_functions = find_test_function(bobje);
        for test_function in &test_functions {
            inputs.push(get_object_path(bobje, &test_function.source_file));
        }
    } else {
        for source_file in &bobje.source_files {
            if source_file.ends_with(".c")
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
        fn add_dependency_inputs(bobje: &Bobje, inputs: &mut Vec<String>, contains_cpp: &mut bool) {
            for dependency_bobje in bobje.dependencies.values() {
                add_dependency_inputs(dependency_bobje, inputs, contains_cpp);
            }
            for source_file in &bobje.source_files {
                if source_file.ends_with(".cpp") || source_file.ends_with(".mm") {
                    *contains_cpp = true;
                }
            }
            inputs.push(format!(
                "{}/lib{}.a",
                bobje.out_dir_with_target(),
                bobje.name
            ));
        }
        for dependency_bobje in bobje.dependencies.values() {
            add_dependency_inputs(dependency_bobje, &mut inputs, &mut contains_cpp);
        }
    }

    if bobje.r#type == crate::BobjeType::Library {
        let static_library_file = format!("{}/lib{}.a", bobje.out_dir_with_target(), bobje.name);
        executor.add_task_cmd(
            format!(
                "{} rc {} {}",
                vars.ar,
                static_library_file,
                inputs.join(" "),
            ),
            inputs.clone(),
            vec![static_library_file.clone()],
        );
    }

    if bobje.r#type == crate::BobjeType::Binary {
        let executable_file = if bobje.is_test {
            format!("{}/test_{}", bobje.out_dir_with_target(), bobje.name)
        } else {
            format!("{}/{}", bobje.out_dir_with_target(), bobje.name)
        };
        let ext = if cfg!(windows) { ".exe" } else { "" };
        if bobje.profile == Profile::Release {
            let unstripped_path = format!("{executable_file}-unstripped{ext}");
            let stripped_path = format!("{executable_file}{ext}");
            executor.add_task_cmd(
                format!(
                    "{} {} {} {} -o {}",
                    if contains_cpp { vars.cxx } else { vars.cc },
                    vars.ldflags,
                    inputs.join(" "),
                    if bobje.is_test {
                        pkg_config_libs("cunit")
                    } else {
                        "".to_string()
                    },
                    unstripped_path,
                ),
                inputs.clone(),
                vec![unstripped_path.clone()],
            );
            executor.add_task_cmd(
                format!("{} {} -o {}", vars.strip, unstripped_path, stripped_path),
                vec![unstripped_path.clone()],
                vec![stripped_path.clone()],
            );
        } else {
            let out_path = format!("{executable_file}{ext}");
            executor.add_task_cmd(
                format!(
                    "{} {} {} {} -o {}",
                    if contains_cpp { vars.cxx } else { vars.cc },
                    vars.ldflags,
                    inputs.join(" "),
                    if bobje.is_test {
                        pkg_config_libs("cunit")
                    } else {
                        "".to_string()
                    },
                    out_path,
                ),
                inputs.clone(),
                vec![out_path.clone()],
            );
        }
    }
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

pub(crate) fn run_ld_tests(bobje: &Bobje) -> ! {
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

fn find_test_function(bobje: &Bobje) -> Vec<TestFunction> {
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
    let test_functions = find_test_function(bobje);

    let mut s = String::new();
    _ = writeln!(s, "// This file is generated by bob, do not edit!");
    _ = writeln!(s, "#include <stdint.h>");
    _ = writeln!(s, "#include <CUnit/Basic.h>\n");
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

    let dest = format!("{}/src-gen/test_main.c", bobje.out_dir_with_target());
    write_file_when_different(&dest, &s).expect("Can't write src-gen/test_main.c");
    bobje.source_files.push(dest);
}
