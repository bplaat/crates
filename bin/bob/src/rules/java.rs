/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::process::{self, Command};

use indexmap::IndexMap;

use crate::Project;

pub(crate) fn generate_java(f: &mut impl Write, project: &Project) {
    let modules = find_modules(&project.source_files);

    // Find module dependencies
    let mut module_deps = HashMap::new();
    for (module, files) in &modules {
        for file in files {
            let source_file = format!("{}/src/{}", project.manifest_dir, file);
            let contents = std::fs::read_to_string(&source_file)
                .unwrap_or_else(|_| panic!("Can't read file: {}", source_file));
            for other_module in modules.keys() {
                let re =
                    regex::Regex::new(&format!(r"import {}.\w+;", other_module.replace("/", ".")))
                        .expect("Failed to compile regex");
                if re.is_match(&contents) {
                    module_deps
                        .entry(module.clone())
                        .or_insert_with(Vec::new)
                        .push(other_module.clone());
                }
            }
        }
    }

    _ = writeln!(f, "\n# Build Java modules");
    _ = writeln!(f, "classes_dir = $target_dir/classes\n");
    _ = writeln!(
        f,
        "rule javac\n  command = javac -Xlint -cp $classes_dir $in -d $classes_dir && touch $stamp\n  description = javac $in\n"
    );
    for (module, source_files) in &modules {
        _ = write!(
            f,
            "build $classes_dir/{}/.stamp: javac {}",
            module,
            source_files
                .iter()
                .map(|source_file| format!("$source_dir/{}", source_file))
                .collect::<Vec<_>>()
                .join(" ")
        );
        if let Some(dependencies) = module_deps.get(module) {
            _ = write!(
                f,
                " | {}",
                dependencies
                    .iter()
                    .map(|source_file| format!("$classes_dir/{}/.stamp", source_file))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        _ = writeln!(f, "\n  stamp = $classes_dir/{}/.stamp", module);
    }
}

pub(crate) fn generate_jar(f: &mut impl Write, project: &Project) {
    let modules = find_modules(&project.source_files);
    let jar_metadata = &project
        .manifest
        .package
        .metadata
        .as_ref()
        .and_then(|m| m.jar.as_ref())
        .expect("Should be some");

    _ = writeln!(f, "\n# Link jar");
    _ = writeln!(
        f,
        "main_class = {}\n",
        if let Some(main_class) = &jar_metadata.main_class {
            main_class.clone()
        } else {
            find_main_class(project).expect("Can't find main class")
        }
    );
    _ = writeln!(
        f,
        "rule jar\n  command = jar cfe $out $main_class -C $in .\n  description = jar $out\n"
    );
    _ = writeln!(
        f,
        "build $classes_dir: phony {}",
        modules
            .keys()
            .map(|module| format!("$classes_dir/{}/.stamp", module))
            .collect::<Vec<_>>()
            .join(" ")
    );
    _ = writeln!(
        f,
        "build $target_dir/${{name}}-$version.jar: jar $classes_dir",
    );
}

pub(crate) fn run_java(project: &Project) {
    let status = Command::new("java")
        .arg("-cp")
        .arg(format!("{}/target/classes", project.manifest_dir))
        .arg(find_main_class(project).expect("Can't find main class"))
        .status()
        .expect("Failed to execute java");
    process::exit(status.code().unwrap_or(1));
}

pub(crate) fn run_jar(project: &Project) {
    let status = Command::new("java")
        .arg("-jar")
        .arg(format!(
            "{}/target/{}-{}.jar",
            project.manifest_dir, project.manifest.package.name, project.manifest.package.version
        ))
        .status()
        .expect("Failed to execute java");
    process::exit(status.code().unwrap_or(1));
}

fn find_modules(source_files: &[String]) -> IndexMap<String, Vec<String>> {
    let mut modules = IndexMap::new();
    for file in source_files {
        if let Some(parent) = Path::new(file).parent() {
            if let Some(parent_str) = parent.to_str() {
                modules
                    .entry(parent_str.to_string())
                    .or_insert_with(Vec::new)
                    .push(file.clone());
            }
        }
    }
    modules
}

fn find_main_class(project: &Project) -> Option<String> {
    let re =
        regex::Regex::new(r"public\s+static\s+void\s+main\s*\(\s*String\s*\[\s*\]\s*args\s*\)")
            .expect("Failed to compile regex");
    for source_file in &project.source_files {
        let source_path = format!("{}/src/{}", project.manifest_dir, source_file);
        let contents = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|_| panic!("Can't read file: {}", source_path));
        if re.is_match(&contents) {
            return Some(source_file.trim_end_matches(".java").replace("/", "."));
        }
    }
    None
}
