/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs;
use std::io::Write;
use std::process::{Command, exit};

use indexmap::IndexMap;

use crate::utils::resolve_source_file_path;
use crate::{Profile, Project};

// MARK: Rules
pub(crate) fn generate_java_vars(f: &mut dyn Write, project: &Project) {
    _ = writeln!(f, "\n# Java variables");
    _ = writeln!(f, "classes_dir = $target_dir/$profile/classes");

    // Javac flags
    _ = writeln!(
        f,
        "javac_flags = -Xlint -Werror {} {}",
        if project.profile == Profile::Release {
            "-g:none"
        } else {
            "-g"
        },
        &project.manifest.build.javac_flags
    );

    // Javac classpath
    _ = write!(f, "classpath = $classes_dir");
    if !project.manifest.build.classpath.is_empty() {
        _ = write!(f, " {}", project.manifest.build.classpath.join(":"));
    }

    // Main class
    if let Some(jar_metadata) = project.manifest.package.metadata.jar.as_ref() {
        _ = writeln!(
            f,
            "main_class = {}",
            if let Some(main_class) = &jar_metadata.main_class {
                main_class.clone()
            } else {
                find_main_class(project).unwrap_or_else(|| {
                    eprintln!("Can't find main class");
                    exit(1);
                })
            }
        );
    }
}

pub(crate) fn generate_java(f: &mut dyn Write, project: &Project) {
    let modules = find_modules(&project.source_files);
    let module_dependencies = find_dependencies(project, &modules);

    _ = writeln!(f, "\n# Compile Java modules");
    if cfg!(windows) {
        _ = writeln!(
            f,
            "rule javac\n  command = cmd.exe /c javac $javac_flags -cp $classpath $in -d $classes_dir && echo.> $stamp\n  description = Compiling $in\n"
        );
    } else {
        _ = writeln!(
            f,
            "rule javac\n  command = javac $javac_flags -cp $classpath $in -d $classes_dir && touch $stamp\n  description = Compiling $in\n"
        );
    }

    for (module, source_files) in &modules {
        _ = write!(
            f,
            "build $classes_dir/{}/.stamp: javac {}",
            module.replace('.', "/"),
            source_files.join(" ")
        );
        if let Some(dependencies) = module_dependencies.get(module) {
            _ = write!(f, " |");
            for dependency in dependencies {
                _ = write!(f, " $classes_dir/{}/.stamp", dependency.replace('.', "/"));
            }
        }
        _ = writeln!(
            f,
            "\n  stamp = $classes_dir/{}/.stamp",
            module.replace('.', "/")
        );
    }
}

pub(crate) fn generate_java_jar(f: &mut dyn Write, project: &Project) {
    let modules = find_modules(&project.source_files);

    _ = writeln!(f, "\n# Link jar");
    _ = writeln!(
        f,
        "rule jar\n  command = jar cfe $out $main_class -C $classes_dir .\n  description = Packaging $out\n"
    );
    _ = writeln!(
        f,
        "build $target_dir/$profile/${{name}}-$version.jar: jar {}",
        modules
            .keys()
            .map(|module| format!("$classes_dir/{}/.stamp", module.replace('.', "/")))
            .collect::<Vec<_>>()
            .join(" "),
    );
}

// MARK: Runners
pub(crate) fn run_java(project: &Project) {
    let status = Command::new("java")
        .arg("-cp")
        .arg(format!("{}/target/classes", project.manifest_dir))
        .arg(find_main_class(project).unwrap_or_else(|| {
            eprintln!("Can't find main class");
            exit(1);
        }))
        .status()
        .expect("Failed to execute java");
    exit(status.code().unwrap_or(1));
}

pub(crate) fn run_java_jar(project: &Project) {
    let status = Command::new("java")
        .arg("-jar")
        .arg(format!(
            "{}/target/{}/{}-{}.jar",
            project.manifest_dir,
            project.profile,
            project.manifest.package.name,
            project.manifest.package.version
        ))
        .status()
        .expect("Failed to execute java");
    exit(status.code().unwrap_or(1));
}

// MARK: Utils
pub(crate) fn find_modules(source_files: &[String]) -> IndexMap<String, Vec<String>> {
    let mut modules = IndexMap::new();
    for source_file in source_files {
        if source_file.ends_with(".java") {
            let parts = source_file.split('/').collect::<Vec<&str>>();
            modules
                .entry(parts[1..parts.len() - 1].join("."))
                .or_insert_with(Vec::new)
                .push(source_file.clone());
        }
    }
    modules
}

fn find_dependencies(
    project: &Project,
    modules: &IndexMap<String, Vec<String>>,
) -> IndexMap<String, Vec<String>> {
    let mut module_deps = IndexMap::new();
    for (module, source_files) in modules {
        for source_file in source_files {
            if let Ok(contents) = fs::read_to_string(resolve_source_file_path(source_file, project))
            {
                for other_module in modules.keys() {
                    if other_module == module {
                        continue;
                    }
                    let re = regex::Regex::new(&format!(r"import {}.\w+;", other_module))
                        .expect("Failed to compile regex");
                    if re.is_match(&contents)
                        && !module_deps
                            .entry(module.clone())
                            .or_insert_with(Vec::new)
                            .iter()
                            .any(|m| m == other_module)
                    {
                        if let Some(deps) = module_deps.get_mut(module) {
                            deps.push(other_module.clone());
                        }
                    }
                }
            }
        }
    }
    module_deps
}

fn find_main_class(project: &Project) -> Option<String> {
    let re = regex::Regex::new(r"(public\s+)?static\s+void\s+main\s*\(")
        .expect("Failed to compile regex");
    for source_file in &project.source_files {
        if let Ok(contents) = fs::read_to_string(resolve_source_file_path(source_file, project)) {
            if re.is_match(&contents) {
                return Some(
                    source_file
                        .trim_start_matches("$source_dir/")
                        .trim_end_matches(".java")
                        .replace(['/', '\\'], "."),
                );
            }
        }
    }
    None
}
