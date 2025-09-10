/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Display, Formatter};
use std::process::exit;
use std::{env, thread};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Subcommand {
    Build,
    Clean,
    CleanCache,
    Help,
    Run,
    Test,
    Version,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Profile {
    Debug,
    Release,
    Test,
}

impl Display for Profile {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Profile::Debug => write!(f, "debug"),
            Profile::Release => write!(f, "release"),
            Profile::Test => write!(f, "test"),
        }
    }
}

pub(crate) struct Args {
    pub subcommand: Subcommand,
    pub manifest_dir: String,
    pub target_dir: String,
    pub profile: Profile,
    pub target: Option<String>,
    pub verbose: bool,
    pub thread_count: usize,
    pub clean_first: bool,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            subcommand: Subcommand::Help,
            manifest_dir: ".".to_string(),
            target_dir: "target".to_string(),
            profile: Profile::Debug,
            target: None,
            verbose: false,
            thread_count: thread::available_parallelism().map_or(1, |n| n.get()),
            clean_first: false,
        }
    }
}

pub(crate) fn parse_args() -> Args {
    let mut args = Args::default();
    let mut args_iter = env::args().skip(1);
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "clean" => args.subcommand = Subcommand::Clean,
            "clean-cache" => args.subcommand = Subcommand::CleanCache,
            "build" => args.subcommand = Subcommand::Build,
            "rebuild" => {
                args.subcommand = Subcommand::Build;
                args.clean_first = true;
            }
            "help" | "-h" | "--help" => args.subcommand = Subcommand::Help,
            "run" => args.subcommand = Subcommand::Run,
            "rerun" => {
                args.subcommand = Subcommand::Run;
                args.clean_first = true;
            }
            "test" => {
                args.subcommand = Subcommand::Test;
                args.profile = Profile::Test;
            }
            "retest" => {
                args.subcommand = Subcommand::Test;
                args.profile = Profile::Test;
                args.clean_first = true;
            }
            "version" | "--version" => args.subcommand = Subcommand::Version,
            "-C" | "--manifest-dir" => {
                args.manifest_dir = args_iter.next().expect("Invalid argument")
            }
            "-T" | "--target-dir" => {
                args.target_dir = args_iter.next().expect("Invalid argument");
            }
            "--target" => {
                args.target = Some(args_iter.next().expect("Invalid argument"));
            }
            "-r" | "--release" => args.profile = Profile::Release,
            "-v" | "--verbose" => args.verbose = true,
            "--single-threaded" => args.thread_count = 1,
            "--thread-count" => {
                args.thread_count = args_iter
                    .next()
                    .and_then(|s| s.parse().ok())
                    .expect("Invalid argument")
            }
            _ => {
                eprintln!("Unknown argument: {arg}");
                exit(1);
            }
        }
    }
    args
}

pub(crate) fn subcommand_help() {
    println!(
        r"Usage: bob [SUBCOMMAND] [OPTIONS]

Options:
  -C <dir>, --manifest-dir    Change to directory <dir> before doing anything
  -T <dir>, --target-dir      Write artifacts to directory <dir>
  -r, --release               Build artifacts in release mode
  -v, --verbose               Print verbose output
  --target <target>           Build for the specified target (e.g., x86_64-unknown-linux-gnu)
  --single-threaded           Run tasks single threaded
  --thread-count <count>      Use <count> threads for building (default: number of available cores)

Subcommands:
  clean                       Remove build artifacts
  clean-cache                 Clean global bob cache
  build                       Build the project
  rebuild                     Clean and build the project
  help                        Print this help message
  run                         Build and run the build artifact
  rerun                       Clean, build and run the build artifact
  test                        Build and run the unit tests
  retest                      Clean, build and run the unit tests
  version                     Print the version number"
    );
}
