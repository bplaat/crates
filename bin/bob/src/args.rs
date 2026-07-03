/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Display, Formatter};

use argparse::{Parser, Subcommand as SubcommandParser};

#[derive(Clone, Copy, PartialEq, Eq, SubcommandParser)]
pub(crate) enum Subcommand {
    #[arg(help = "Build the project")]
    Build,
    #[arg(help = "Remove build artifacts")]
    Clean,
    #[arg(name = "clean-cache", help = "Clean global bob cache")]
    CleanCache,
    #[arg(help = "Print this help message")]
    Help,
    #[arg(help = "Clean and build the project")]
    Rebuild,
    #[arg(help = "Build and run the build artifact")]
    Run,
    #[arg(help = "Clean, build and run the build artifact")]
    Rerun,
    #[arg(help = "Build and run the unit tests")]
    Test,
    #[arg(help = "Clean, build and run the unit tests")]
    Retest,
    #[arg(help = "Print the version number")]
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

#[derive(Parser)]
#[arg(name = "bob")]
pub(crate) struct Args {
    #[arg(subcommand)]
    pub subcommand: Subcommand,
    #[arg(
        short = 'C',
        long = "manifest-dir",
        value = "dir",
        help = "Change to directory <dir> before doing anything"
    )]
    pub manifest_dir: String,
    #[arg(
        short = 'T',
        long = "target-dir",
        value = "dir",
        help = "Write artifacts to directory <dir>"
    )]
    pub target_dir: String,
    pub profile: Profile,
    #[arg(
        long = "target",
        value = "target",
        help = "Build for the specified target"
    )]
    pub target: Option<String>,
    #[arg(
        short = 'r',
        long = "release",
        help = "Build artifacts in release mode"
    )]
    pub release: bool,
    #[arg(long = "verbose", help = "Print verbose output")]
    pub verbose: bool,
    #[arg(
        short = '1',
        long = "single-threaded",
        help = "Run tasks single threaded"
    )]
    pub single_threaded: bool,
    #[arg(
        short = 'j',
        long = "jobs",
        value = "count",
        help = "Use <count> threads for building"
    )]
    pub thread_count: Option<usize>,
    pub clean_first: bool,
    #[arg(short = 't', long = "time", help = "Show time taken for the build")]
    pub show_time: bool,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            subcommand: Subcommand::Help,
            manifest_dir: ".".to_string(),
            target_dir: "target".to_string(),
            profile: Profile::Debug,
            target: None,
            release: false,
            verbose: false,
            single_threaded: false,
            thread_count: None,
            clean_first: false,
            show_time: false,
        }
    }
}

pub(crate) fn parse_args() -> Args {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    let mut parse_args = Vec::new();
    let mut clean_first = false;
    let mut test_profile = false;
    let mut i = 0;
    while i < raw_args.len() {
        match raw_args[i].as_str() {
            "test" => {
                parse_args.push(raw_args[i].clone());
                test_profile = true;
            }
            _ => parse_args.push(raw_args[i].clone()),
        }
        i += 1;
    }

    let mut args = Args::parse_from(parse_args).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(1);
    });
    if args.release {
        args.profile = Profile::Release;
    }
    if args.single_threaded {
        args.thread_count = Some(1);
    }
    if args.subcommand == Subcommand::Rebuild {
        args.subcommand = Subcommand::Build;
        clean_first = true;
    }
    if args.subcommand == Subcommand::Rerun {
        args.subcommand = Subcommand::Run;
        clean_first = true;
    }
    if args.subcommand == Subcommand::Retest {
        args.subcommand = Subcommand::Test;
        clean_first = true;
        test_profile = true;
    }
    if test_profile {
        args.profile = Profile::Test;
    }
    if clean_first {
        args.clean_first = true;
    }
    args
}
