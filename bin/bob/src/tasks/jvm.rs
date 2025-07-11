/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::fs;
use std::process::{Command, exit};

use regex::Regex;

use crate::Bobje;
use crate::args::Profile;
use crate::executor::Executor;
use crate::manifest::JarDependency;

// MARK: Java/Kotlin tasks
pub(crate) fn detect_java_kotlin(bobje: &Bobje) -> bool {
    bobje
        .source_files
        .iter()
        .any(|path| path.ends_with(".java"))
        || detect_kotlin_from_source_files(&bobje.source_files)
}

pub(crate) fn detect_kotlin_from_source_files(source_files: &[String]) -> bool {
    source_files.iter().any(|path| path.ends_with(".kt"))
}

pub(crate) fn generate_javac_kotlinc_tasks(bobje: &Bobje, executor: &mut Executor) {
    let classes_dir = format!("{}/classes", bobje.out_dir());
    let modules = find_modules(bobje);
    let module_deps = find_dependencies(&modules);

    let mut javac_flags = "-Xlint -Werror".to_string();
    if bobje.profile == Profile::Debug {
        javac_flags.push_str(" -g");
    }
    if !bobje.manifest.build.javac_flags.is_empty() {
        javac_flags.push(' ');
        javac_flags.push_str(&bobje.manifest.build.javac_flags);
    }

    let mut kotlinc_flags = "-Werror".to_string();
    if !bobje.manifest.build.kotlinc_flags.is_empty() {
        kotlinc_flags.push(' ');
        kotlinc_flags.push_str(&bobje.manifest.build.kotlinc_flags);
    }

    #[cfg(windows)]
    let class_separator = ";";
    #[cfg(not(windows))]
    let class_separator = ":";
    let classpath = format!(
        "{}{}",
        classes_dir,
        if !bobje.manifest.build.classpath.is_empty() {
            format!(
                "{}{}",
                class_separator,
                bobje.manifest.build.classpath.join(class_separator)
            )
        } else {
            "".to_string()
        }
    );

    for module in &modules {
        let mut inputs = module.source_files.clone();
        if let Some(dependencies) = module_deps.get(&module.name) {
            for dependency_module in dependencies {
                let classes_module_dir = format!(
                    "{}/{}",
                    classes_dir,
                    dependency_module.name.replace('.', "/")
                );
                if !inputs.contains(&classes_module_dir) {
                    inputs.push(classes_module_dir);
                }
            }
        }
        for dependency_bobje in bobje.dependencies.values() {
            if dependency_bobje.r#type == crate::BobjeType::ExternalJar {
                inputs.push(format!(
                    "{}/{}",
                    classes_dir,
                    dependency_bobje
                        .jar
                        .as_ref()
                        .expect("Should be some")
                        .package
                        .replace('.', "/")
                ));
            }
        }

        let mut commands = Vec::new();

        // Javac
        let java_files = module
            .source_files
            .iter()
            .filter(|f| f.ends_with(".java"))
            .cloned()
            .collect::<Vec<_>>();
        if !java_files.is_empty() {
            commands.push(format!(
                "javac {} -cp {} -d {} {}",
                javac_flags,
                classpath,
                classes_dir,
                java_files.join(" ")
            ));
        }

        // Kotlinc
        let kotlin_files = module
            .source_files
            .iter()
            .filter(|f| f.ends_with(".kt"))
            .cloned()
            .collect::<Vec<_>>();
        if !kotlin_files.is_empty() {
            commands.push(format!(
                "kotlinc {} -cp {} -d {} {}",
                kotlinc_flags,
                classpath,
                classes_dir,
                module
                    .source_files
                    .iter()
                    .filter(|f| f.ends_with(".kt"))
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
        }

        executor.add_task_cmd(
            commands.join(" && "),
            inputs,
            vec![format!("{}/{}", classes_dir, module.name.replace('.', "/"))],
        );
    }
}

pub(crate) fn run_java_class(bobje: &Bobje) -> ! {
    let status = Command::new("java")
        .arg("-cp")
        .arg(format!("{}/classes", bobje.out_dir()))
        .arg(find_main_class(bobje).unwrap_or_else(|| {
            eprintln!("Can't find main class");
            exit(1);
        }))
        .status()
        .expect("Failed to execute java");
    exit(status.code().unwrap_or(1))
}

// MARK: Jar tasks
pub(crate) fn detect_jar(bobje: &Bobje) -> bool {
    bobje.manifest.package.metadata.jar.is_some()
}

pub(crate) fn download_extract_jar_tasks(
    bobje: &Bobje,
    executor: &mut Executor,
    jar: &JarDependency,
) {
    // Add download task
    let downloaded_jar = format!(
        "{}/jar-cache/{}-{}.jar",
        bobje.target_dir, bobje.name, bobje.version
    );
    executor.add_task_cmd(
        format!("wget {} -O {}", jar.url, downloaded_jar),
        vec![],
        vec![downloaded_jar.clone()],
    );

    // Add extract task
    let classes_dir = format!("{}/classes", bobje.out_dir());
    executor.add_task_cmd(
        format!(
            "cd {} && jar xf {}",
            classes_dir,
            downloaded_jar.replace(&bobje.target_dir, "../..")
        ),
        vec![downloaded_jar],
        vec![format!("{}/{}", classes_dir, jar.package.replace('.', "/"))],
    );
}

pub(crate) fn generate_jar_tasks(bobje: &Bobje, executor: &mut Executor) {
    let classes_dir = format!("{}/classes", bobje.out_dir());
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

    // Minify names and tree shake classes with ProGuard
    let optimized_classes_dir = format!("{}-optimized", classes_dir);
    if bobje.profile == Profile::Release {
        let java_home = std::env::var("JAVA_HOME").expect("$JAVA_HOME not set");
        let mut keeps = vec![format!(
            "public class {} {{ public static void main(java.lang.String[]); }}",
            main_class
        )];
        if let Some(jar) = bobje.manifest.package.metadata.jar.as_ref() {
            keeps.extend(jar.proguard_keep.clone());
        }

        executor.add_task_cmd(
            format!(
                "proguard -injars {} -outjars {} -libraryjars {}/jmods/java.base.jmod {} > /dev/null && rm -rf {}/META-INF && find {} -name '*.kotlin_builtins' -delete && find {} -type d -empty -delete",
                classes_dir, optimized_classes_dir, java_home,
                keeps
                    .iter()
                    .map(|keep| format!("-keep '{}'", keep))
                    .collect::<Vec<_>>()
                    .join(" "),
                optimized_classes_dir,
                optimized_classes_dir,
                optimized_classes_dir
            ),
            modules
                .iter()
                .map(|module| format!("{}/{}", classes_dir, module.name.replace('.', "/")))
                .collect::<Vec<_>>(),
            vec![optimized_classes_dir.clone()],
        );
    }

    // Build JAR file
    let jar_file = format!("{}/{}-{}.jar", bobje.out_dir(), bobje.name, bobje.version);
    executor.add_task_cmd(
        format!(
            "jar cfe {} {} -C {} .",
            jar_file,
            main_class,
            if bobje.profile == Profile::Release {
                &optimized_classes_dir
            } else {
                &classes_dir
            }
        ),
        if bobje.profile == Profile::Release {
            vec![optimized_classes_dir]
        } else {
            modules
                .iter()
                .map(|module| format!("{}/{}", classes_dir, module.name.replace('.', "/")))
                .collect::<Vec<_>>()
        },
        vec![jar_file],
    );
}

pub(crate) fn run_jar(bobje: &Bobje) -> ! {
    let status = Command::new("java")
        .arg("-jar")
        .arg(format!(
            "{}/{}-{}.jar",
            bobje.out_dir(),
            bobje.name,
            bobje.version
        ))
        .status()
        .expect("Failed to execute java");
    exit(status.code().unwrap_or(1))
}

// MARK: Utils
pub(crate) fn get_class_name(source_file: &str) -> String {
    source_file
        .split("src/")
        .nth(1)
        .or_else(|| source_file.split("src-gen/").nth(1))
        .expect("Should be some")
        .trim_end_matches(".java")
        .trim_end_matches(".kt")
        .replace(['/', '\\'], ".")
}

fn get_module_name(source_file: &str) -> String {
    get_class_name(source_file)
        .rsplit_once('.')
        .map_or("", |(prefix, _)| prefix)
        .to_string()
}

#[derive(Clone)]
pub(crate) struct Module {
    pub name: String,
    pub source_files: Vec<String>,
}

pub(crate) fn find_modules(bobje: &Bobje) -> Vec<Module> {
    let mut modules = Vec::new();
    for dependency_bobje in bobje.dependencies.values() {
        for module in find_modules(dependency_bobje) {
            modules.push(module);
        }
    }
    for source_file in &bobje.source_files {
        if source_file.ends_with(".java") || source_file.ends_with(".kt") {
            let module_name = get_module_name(source_file);
            if let Some(module) = modules.iter_mut().find(|m| m.name == module_name) {
                module.source_files.push(source_file.to_string());
            } else {
                modules.push(Module {
                    name: module_name.clone(),
                    source_files: vec![source_file.to_string()],
                });
            }
        }
    }
    modules
}

fn find_dependencies(modules: &Vec<Module>) -> HashMap<String, Vec<Module>> {
    let mut module_deps = HashMap::new();
    for module in modules {
        for source_file in &module.source_files {
            if let Ok(contents) = fs::read_to_string(source_file) {
                for other_module in modules {
                    if other_module.name == module.name {
                        continue;
                    }
                    let re = Regex::new(&format!(r"import {}.\w+[;\n\r]", other_module.name))
                        .expect("Failed to compile regex");
                    if re.is_match(&contents) {
                        module_deps
                            .entry(module.name.clone())
                            .or_insert_with(Vec::new)
                            .push(other_module.clone());
                    }
                }
            }
        }
    }
    module_deps
}

fn find_main_class(bobje: &Bobje) -> Option<String> {
    let java_re =
        Regex::new(r"(public\s+)?static\s+void\s+main\s*\(").expect("Failed to compile regex");
    let kotlin_re = Regex::new(r"fun\s+main\s*\(").expect("Failed to compile regex");
    for source_file in &bobje.source_files {
        if source_file.ends_with(".java") {
            if let Ok(contents) = fs::read_to_string(source_file) {
                if java_re.is_match(&contents) {
                    return Some(get_class_name(source_file));
                }
            }
        }
        if source_file.ends_with(".kt") {
            if let Ok(contents) = fs::read_to_string(source_file) {
                if kotlin_re.is_match(&contents) {
                    return Some(get_class_name(source_file) + "Kt");
                }
            }
        }
    }
    None
}
