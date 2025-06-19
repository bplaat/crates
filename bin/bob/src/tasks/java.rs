/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs;
use std::process::{Command, exit};

use indexmap::IndexMap;

use crate::Project;
use crate::args::Profile;
use crate::executor::Executor;

// MARK: Javac tasks
pub(crate) fn detect_java(project: &Project) -> bool {
    project
        .source_files
        .iter()
        .any(|path| path.ends_with(".java"))
}

pub(crate) fn generate_javac_tasks(project: &Project, executor: &mut Executor) {
    let classes_dir = format!("{}/{}/classes", project.target_dir, project.profile);
    let modules = find_modules(&project.source_files);
    let module_deps = find_dependencies(&modules);

    let mut javac_flags = format!(
        "-Xlint -Werror {}",
        if project.profile == Profile::Release {
            "-g:none"
        } else {
            "-g"
        }
    );
    if !project.manifest.build.javac_flags.is_empty() {
        javac_flags.push(' ');
        javac_flags.push_str(&project.manifest.build.javac_flags);
    }

    let classpath = format!(
        "{}{}",
        classes_dir,
        if !project.manifest.build.classpath.is_empty() {
            format!(":{}", project.manifest.build.classpath.join(":"))
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

pub(crate) fn run_java_class(project: &Project) {
    let status = Command::new("java")
        .arg("-cp")
        .arg(format!(
            "{}/{}/classes",
            project.target_dir, project.profile
        ))
        .arg(find_main_class(project).unwrap_or_else(|| {
            eprintln!("Can't find main class");
            exit(1);
        }))
        .status()
        .expect("Failed to execute java");
    exit(status.code().unwrap_or(1));
}

// MARK: Jar tasks
pub(crate) fn detect_jar(project: &Project) -> bool {
    project.manifest.package.metadata.jar.is_some()
}

pub(crate) fn generate_jar_tasks(project: &Project, executor: &mut Executor) {
    let classes_dir = format!("{}/{}/classes", project.target_dir, project.profile);
    let modules = find_modules(&project.source_files);

    let main_class = project
        .manifest
        .package
        .metadata
        .jar
        .as_ref()
        .and_then(|jar| jar.main_class.clone())
        .unwrap_or_else(|| {
            find_main_class(project).unwrap_or_else(|| {
                eprintln!("Can't find main class");
                exit(1);
            })
        });

    let jar_file = format!(
        "{}/{}/{}-{}.jar",
        project.target_dir,
        project.profile,
        project.manifest.package.name,
        project.manifest.package.version
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

pub(crate) fn run_jar(project: &Project) {
    let status = Command::new("java")
        .arg("-jar")
        .arg(format!(
            "{}/{}/{}-{}.jar",
            project.target_dir,
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
            let parts = source_file
                .split("src/")
                .nth(1)
                .or_else(|| source_file.split("src-gen/").nth(1))
                .expect("Should be some")
                .split(['/', '\\'])
                .collect::<Vec<_>>();
            modules
                .entry(parts[0..parts.len() - 1].join("."))
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

fn find_main_class(project: &Project) -> Option<String> {
    let re = regex::Regex::new(r"(public\s+)?static\s+void\s+main\s*\(")
        .expect("Failed to compile regex");
    for source_file in &project.source_files {
        if let Ok(contents) = fs::read_to_string(source_file) {
            if re.is_match(&contents) {
                return Some(
                    source_file
                        .split("src/")
                        .nth(1)
                        .or_else(|| source_file.split("src-gen/").nth(1))
                        .expect("Should be some")
                        .trim_end_matches(".java")
                        .replace(['/', '\\'], "."),
                );
            }
        }
    }
    None
}
