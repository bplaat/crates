/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! # Bassie's Obvious Builder (bob)
//!
//! A simple, zero-config meta-build system for my projects, it's just a ninja build file generator.

use std::fs::{self, File};
use std::io::Write;

use rules::Rule;

use crate::manifest::Manifest;

mod manifest;
mod rules;

// MARK: Arguments
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Profile {
    Debug,
    Release,
}

struct Args {
    manifest_dir: String,
    subcommand: SubCommand,
    profile: Profile,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            manifest_dir: ".".to_string(),
            subcommand: SubCommand::Help,
            profile: Profile::Debug,
        }
    }
}

#[derive(PartialEq, Eq)]
enum SubCommand {
    Clean,
    Build,
    Help,
    Run,
}

fn parse_args() -> Args {
    let mut args = Args::default();
    let mut args_iter = std::env::args().skip(1);
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "clean" => args.subcommand = SubCommand::Clean,
            "build" => args.subcommand = SubCommand::Build,
            "run" => args.subcommand = SubCommand::Run,
            "help" => args.subcommand = SubCommand::Help,
            "-C" => args.manifest_dir = args_iter.next().expect("Invalid argument"),
            "-r" | "--release" => args.profile = Profile::Release,
            _ => {
                eprintln!("Unknown argument: {}", arg);
                std::process::exit(1);
            }
        }
    }
    args
}

// MARK: Utils
fn index_files(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    let entries = fs::read_dir(dir).expect("Can't read directory");
    for entry in entries {
        let entry = entry.expect("Can't read directory entry");
        let path = entry.path();
        if path.is_dir() {
            files.extend(index_files(&path.to_string_lossy()));
        } else {
            files.push(path.to_string_lossy().to_string());
        }
    }
    files
}

// MARK: Main
pub(crate) struct Project {
    manifest_dir: String,
    manifest: Manifest,
    profile: Profile,
    source_files: Vec<String>,
}

fn main() {
    let args = parse_args();

    if args.subcommand == SubCommand::Help {
        println!("Usage: bob [SUBCOMMAND] [OPTIONS]");
        println!();
        println!("Options:");
        println!("  -C <dir>    Change to directory <dir> before doing anything");
        println!();
        println!("Subcommands:");
        println!("  clean       Remove build artifacts");
        println!("  build       Build the project");
        println!("  run         Run the build artifact after building");
        println!("  help        Print this help message");
        return;
    }

    // Read manifest
    let manifest: Manifest = toml::from_str(
        &fs::read_to_string(format!("{}/bob.toml", args.manifest_dir))
            .expect("Can't read bob.toml file"),
    )
    .expect("Can't parse bob.toml file");

    // Clean build artifacts
    if args.subcommand == SubCommand::Clean {
        fs::remove_dir_all(format!("{}/target", args.manifest_dir))
            .expect("Can't remove target directory");
        return;
    }

    // Create target/ directory
    fs::create_dir_all(format!("{}/target", args.manifest_dir))
        .expect("Can't create target directory");

    // Index source files
    let source_dir = format!("{}/src/", args.manifest_dir);
    let source_files: Vec<String> = index_files(&source_dir)
        .into_iter()
        .map(|file| {
            file.strip_prefix(&source_dir)
                .expect("Should be some")
                .to_string()
        })
        .collect();

    // Generate ninja file
    let project = Project {
        manifest_dir: args.manifest_dir.clone(),
        manifest,
        profile: args.profile,
        source_files: source_files.clone(),
    };
    let generated_rules = generate_ninja_file(&project);

    // Run ninja
    let status = std::process::Command::new("ninja")
        .arg("-C")
        .arg(format!("{}/target", args.manifest_dir))
        .status()
        .expect("Failed to execute ninja");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    // Run build artifact
    if args.subcommand == SubCommand::Run {
        if generated_rules.contains(&Rule::Bundle) {
            rules::cx::run_bundle(&project);
        }
        if generated_rules.contains(&Rule::Ld) {
            rules::cx::run_ld(&project);
        }

        if generated_rules.contains(&Rule::Jar) {
            rules::java::run_jar(&project);
        }
        if generated_rules.contains(&Rule::Java) {
            rules::java::run_java(&project);
        }
        panic!("No build artifact to run");
    }
}

fn generate_ninja_file(project: &Project) -> Vec<Rule> {
    // Generate ninja file
    let mut f = File::create(format!("{}/target/build.ninja", project.manifest_dir))
        .expect("Can't create build.ninja file");

    // Base variables
    _ = writeln!(f, "# This file is generated by bob, do not edit!");
    _ = writeln!(f, "name = {}", project.manifest.package.name);
    if let Some(identifier) = &project.manifest.package.identifier {
        _ = writeln!(f, "identifier = {}", identifier);
    }
    _ = writeln!(f, "version = {}", project.manifest.package.version);
    _ = writeln!(f, "manifest_dir = ..");
    _ = writeln!(f, "source_dir = $manifest_dir/src");
    _ = writeln!(f, "target_dir = $manifest_dir/target");

    // Determine needed rules
    let mut needed_rules = Vec::new();
    for file in &project.source_files {
        if file.ends_with(".c") && !needed_rules.contains(&Rule::C) {
            needed_rules.push(Rule::C);
        }
        if file.ends_with(".cpp") && !needed_rules.contains(&Rule::Cpp) {
            needed_rules.push(Rule::Cpp);
        }
        if file.ends_with(".m") && !needed_rules.contains(&Rule::Objc) {
            needed_rules.push(Rule::Objc);
        }
        if file.ends_with(".mm") && !needed_rules.contains(&Rule::Objcpp) {
            needed_rules.push(Rule::Objcpp);
        }
        if file.ends_with(".java") && !needed_rules.contains(&Rule::Java) {
            needed_rules.push(Rule::Java);
        }
    }
    for file in &project.source_files {
        if (file.ends_with(".c")
            || file.ends_with(".cpp")
            || file.ends_with(".m")
            || file.ends_with(".mm"))
            && !needed_rules.contains(&Rule::Ld)
        {
            needed_rules.push(Rule::Ld);
        }
    }
    if let Some(metadata) = project.manifest.package.metadata.as_ref() {
        if metadata.jar.is_some() {
            needed_rules.push(Rule::Jar);
        }
        if metadata.bundle.is_some() {
            needed_rules.push(Rule::Bundle);
        }
    }

    // Generate rules
    for rule in &needed_rules {
        match rule {
            Rule::C => rules::cx::generate_c(&mut f, project),
            Rule::Cpp => rules::cx::generate_cpp(&mut f, project),
            Rule::Objc => rules::cx::generate_objc(&mut f, project),
            Rule::Objcpp => rules::cx::generate_objcpp(&mut f, project),
            Rule::Ld => rules::cx::generate_ld(&mut f, project),
            Rule::Bundle => rules::cx::generate_bundle(&mut f, project),
            Rule::Java => rules::java::generate_java(&mut f, project),
            Rule::Jar => rules::java::generate_jar(&mut f, project),
        };
    }
    needed_rules
}
