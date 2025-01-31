/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Display, Formatter};

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
    pub(crate) manifest_dir: String,
    pub(crate) subcommand: SubCommand,
    pub(crate) profile: Profile,
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
pub(crate) enum SubCommand {
    Clean,
    Build,
    Help,
    Run,
    Test,
    Version,
}

pub(crate) fn parse_args() -> Args {
    let mut args = Args::default();
    let mut args_iter = std::env::args().skip(1);
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "c" | "clean" => args.subcommand = SubCommand::Clean,
            "b" | "build" => args.subcommand = SubCommand::Build,
            "h" | "help" | "-h" | "--help" => args.subcommand = SubCommand::Help,
            "r" | "run" => args.subcommand = SubCommand::Run,
            "t" | "test" => args.subcommand = SubCommand::Test,
            "v" | "version" | "-v" | "--version" => args.subcommand = SubCommand::Version,
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
