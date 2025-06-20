/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::env;
use std::fmt::{self, Display, Formatter};
use std::process::exit;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Subcommand {
    Build,
    Clean,
    Help,
    Rebuild,
    Run,
    Test,
    Version,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Profile {
    Debug,
    Release,
}

impl Display for Profile {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Profile::Debug => write!(f, "debug"),
            Profile::Release => write!(f, "release"),
        }
    }
}

pub(crate) struct Args {
    pub subcommand: Subcommand,
    pub manifest_dir: String,
    pub target_dir: String,
    pub profile: Profile,
    pub verbose: bool,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            subcommand: Subcommand::Help,
            manifest_dir: ".".to_string(),
            target_dir: "target".to_string(),
            profile: Profile::Debug,
            verbose: cfg!(debug_assertions),
        }
    }
}

pub(crate) fn parse_args() -> Args {
    let mut args = Args::default();
    let mut args_iter = env::args().skip(1);
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "clean" => args.subcommand = Subcommand::Clean,
            "build" => args.subcommand = Subcommand::Build,
            "help" | "-h" | "--help" => args.subcommand = Subcommand::Help,
            "run" => args.subcommand = Subcommand::Run,
            "rebuild" => args.subcommand = Subcommand::Rebuild,
            "test" => args.subcommand = Subcommand::Test,
            "version" | "--version" => args.subcommand = Subcommand::Version,
            "-v" | "--verbose" => args.verbose = true,
            "-C" | "--manifest-dir" => {
                args.manifest_dir = args_iter.next().expect("Invalid argument")
            }
            "-T" | "--target-dir" => {
                args.target_dir = args_iter.next().expect("Invalid argument");
            }
            "-r" | "--release" => args.profile = Profile::Release,
            _ => {
                eprintln!("Unknown argument: {}", arg);
                exit(1);
            }
        }
    }
    args
}
