/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs;
use std::process::{Command, exit};

use indexmap::IndexMap;

use crate::Bobje;
use crate::args::Profile;
use crate::executor::Executor;

// MARK: Javac tasks
pub(crate) fn detect_java(bobje: &Bobje) -> bool {
    bobje
        .source_files
        .iter()
        .any(|path| path.ends_with(".java"))
}

pub(crate) fn generate_javac_tasks(bobje: &Bobje, executor: &mut Executor) {
    let classes_dir = format!("{}/{}/classes", bobje.target_dir, bobje.profile);
    let modules = find_modules(bobje);
    let module_deps = find_dependencies(&modules);

    let mut javac_flags = format!(
        "-Xlint -Werror {}",
        if bobje.profile == Profile::Release {
            "-g:none"
        } else {
            "-g"
        }
    );
    if !bobje.manifest.build.javac_flags.is_empty() {
        javac_flags.push(' ');
        javac_flags.push_str(&bobje.manifest.build.javac_flags);
    }

    let classpath = format!(
        "{}{}",
        classes_dir,
        if !bobje.manifest.build.classpath.is_empty() {
            format!(":{}", bobje.manifest.build.classpath.join(":"))
        } else {
            "".to_string()
        }
    );

    for (module, source_files) in &modules {
        let mut inputs = source_files.clone();
        if let Some(dependencies) = module_deps.get(module) {
            for dependency in dependencies {
                inputs.push(format!("{}/{}", classes_dir, dependency.replace('.', "/")));
            }
        }
        executor.add_task_cmd(
            format!(
                "javac {} -cp {} -d {} {}",
                javac_flags,
                classpath,
                classes_dir,
                source_files.join(" ")
            ),
            inputs,
            vec![format!("{}/{}", classes_dir, module.replace('.', "/"))],
        );
    }
}

pub(crate) fn run_java_class(bobje: &Bobje) {
    let status = Command::new("java")
        .arg("-cp")
        .arg(format!("{}/{}/classes", bobje.target_dir, bobje.profile))
        .arg(find_main_class(bobje).unwrap_or_else(|| {
            eprintln!("Can't find main class");
            exit(1);
        }))
        .status()
        .expect("Failed to execute java");
    exit(status.code().unwrap_or(1));
}

// MARK: Jar tasks
pub(crate) fn detect_jar(bobje: &Bobje) -> bool {
    bobje.manifest.package.metadata.jar.is_some()
}

pub(crate) fn generate_jar_tasks(bobje: &Bobje, executor: &mut Executor) {
    let classes_dir = format!("{}/{}/classes", bobje.target_dir, bobje.profile);
    let modules = find_modules(bobje);

    let main_class = bobje
        .manifest
        .package
        .metadata
        .jar
        .as_ref()
        .and_then(|jar| jar.main_class.clone())
        .unwrap_or_else(|| {
            find_main_class(bobje).unwrap_or_else(|| {
                eprintln!("Can't find main class");
                exit(1);
            })
        });

    let jar_file = format!(
        "{}/{}/{}-{}.jar",
        bobje.target_dir,
        bobje.profile,
        bobje.manifest.package.name,
        bobje.manifest.package.version
    );
    executor.add_task_cmd(
        format!("jar cfe {} {} -C {} .", jar_file, main_class, classes_dir),
        modules
            .keys()
            .map(|module| format!("{}/{}", classes_dir, module.replace('.', "/")))
            .collect::<Vec<_>>(),
        vec![jar_file],
    );
}

pub(crate) fn run_jar(bobje: &Bobje) {
    let status = Command::new("java")
        .arg("-jar")
        .arg(format!(
            "{}/{}/{}-{}.jar",
            bobje.target_dir,
            bobje.profile,
            bobje.manifest.package.name,
            bobje.manifest.package.version
        ))
        .status()
        .expect("Failed to execute java");
    exit(status.code().unwrap_or(1));
}

// MARK: Utils
fn get_class_name(source_file: &str) -> String {
    source_file
        .split("src/")
        .nth(1)
        .or_else(|| source_file.split("src-gen/").nth(1))
        .expect("Should be some")
        .trim_end_matches(".java")
        .replace(['/', '\\'], ".")
}

fn get_module_name(source_file: &str) -> String {
    get_class_name(source_file)
        .rsplit_once('.')
        .map_or("", |(prefix, _)| prefix)
        .to_string()
}

pub(crate) fn find_modules(bobje: &Bobje) -> IndexMap<String, Vec<String>> {
    let mut modules = IndexMap::new();
    for dependency_bobje in bobje.dependencies.values() {
        let other_modules = find_modules(dependency_bobje);
        for (module, source_files) in other_modules {
            modules
                .entry(module)
                .or_insert_with(Vec::new)
                .extend(source_files);
        }
    }

    for source_file in &bobje.source_files {
        if source_file.ends_with(".java") {
            modules
                .entry(get_module_name(source_file))
                .or_insert_with(Vec::new)
                .push(source_file.clone());
        }
    }
    modules
}

fn find_dependencies(modules: &IndexMap<String, Vec<String>>) -> IndexMap<String, Vec<String>> {
    let mut module_deps = IndexMap::new();
    for (module, source_files) in modules {
        for source_file in source_files {
            if let Ok(contents) = fs::read_to_string(source_file) {
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

fn find_main_class(bobje: &Bobje) -> Option<String> {
    let re = regex::Regex::new(r"(public\s+)?static\s+void\s+main\s*\(")
        .expect("Failed to compile regex");
    for source_file in &bobje.source_files {
        if let Ok(contents) = fs::read_to_string(source_file) {
            if re.is_match(&contents) {
                return Some(get_class_name(source_file));
            }
        }
    }
    None
}
