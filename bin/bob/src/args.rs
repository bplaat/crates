/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::env;
use std::fmt::{self, Display, Formatter};

use clap::Parser;

// MARK: Public types (unchanged API for main.rs / bobje.rs)

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
    pub thread_count: Option<usize>,
    pub clean_first: bool,
    pub show_time: bool,
    #[cfg(feature = "javac-server")]
    pub disable_javac_server: bool,
}

// MARK: Internal raw args (parsed via derive)

#[derive(Clone, Copy, PartialEq, Eq, clap::Subcommand)]
enum RawSubcommand {
    Build,
    Rebuild,
    Clean,
    #[command(name = "clean-cache")]
    CleanCache,
    Help,
    Run,
    Rerun,
    Test,
    Retest,
    Version,
}

#[derive(Parser)]
#[command(about = "Bob build system", version)]
struct RawArgs {
    #[arg(subcommand)]
    subcommand: Option<RawSubcommand>,

    #[arg(short = 'C', long = "manifest-dir", help = "Change to directory", value_name = "DIR", default_value = ".")]
    manifest_dir: String,

    #[arg(short = 'T', long = "target-dir", help = "Write artifacts to directory", value_name = "DIR", default_value = "target")]
    target_dir: String,

    #[arg(short = 'r', long, help = "Build in release mode")]
    release: bool,

    #[arg(short = 't', long = "time", help = "Show time taken")]
    show_time: bool,

    #[arg(short = 'v', long, help = "Print verbose output")]
    verbose: bool,

    #[arg(long, help = "Build for target", value_name = "TARGET")]
    target: Option<String>,

    #[arg(short = '1', long = "single-threaded", help = "Run tasks single threaded")]
    single_threaded: bool,

    #[arg(short = 'j', long = "jobs", alias = "thread-count", help = "Thread count", value_name = "N")]
    thread_count: Option<usize>,

    #[cfg(feature = "javac-server")]
    #[arg(long = "disable-javac-server", help = "Disable the javac server")]
    disable_javac_server: bool,
}

// MARK: Parse function

pub(crate) fn parse_args() -> Args {
    let raw = RawArgs::parse();

    let clean_first = matches!(
        raw.subcommand,
        Some(RawSubcommand::Rebuild) | Some(RawSubcommand::Rerun) | Some(RawSubcommand::Retest)
    );

    let subcommand = match raw.subcommand {
        Some(RawSubcommand::Build) | Some(RawSubcommand::Rebuild) => Subcommand::Build,
        Some(RawSubcommand::Clean) => Subcommand::Clean,
        Some(RawSubcommand::CleanCache) => Subcommand::CleanCache,
        Some(RawSubcommand::Run) | Some(RawSubcommand::Rerun) => Subcommand::Run,
        Some(RawSubcommand::Test) | Some(RawSubcommand::Retest) => Subcommand::Test,
        Some(RawSubcommand::Version) => Subcommand::Version,
        Some(RawSubcommand::Help) | None => Subcommand::Help,
    };

    let profile = match raw.subcommand {
        Some(RawSubcommand::Test) | Some(RawSubcommand::Retest) => Profile::Test,
        _ => {
            if raw.release {
                Profile::Release
            } else {
                Profile::Debug
            }
        }
    };

    let thread_count = if raw.single_threaded {
        Some(1)
    } else {
        raw.thread_count
    };

    Args {
        subcommand,
        manifest_dir: raw.manifest_dir,
        target_dir: raw.target_dir,
        profile,
        target: raw.target,
        verbose: raw.verbose,
        thread_count,
        clean_first,
        show_time: raw.show_time,
        #[cfg(feature = "javac-server")]
        disable_javac_server: raw.disable_javac_server
            || cfg!(windows)
            || env::var("CI").is_ok(),
    }
}

pub(crate) fn subcommand_help() {
    println!(
        r"Usage: bob [SUBCOMMAND] [OPTIONS]

Options:
  -C <dir>, --manifest-dir              Change to directory <dir> before doing anything
  -T <dir>, --target-dir                Write artifacts to directory <dir>
  -r, --release                         Build artifacts in release mode
  -t, --time                            Show time taken for the build
  -v, --verbose                         Print verbose output
  --target <target>                     Build for the specified target (e.g., x86_64-unknown-linux-gnu)
  -1, --single-threaded                 Run tasks single threaded
  -j, --jobs, --thread-count <count>    Use <count> threads for building (default: number of available cores)"
    );

    #[cfg(feature = "javac-server")]
    println!(
        "  --disable-javac-server                Disable the spawning and use of the javac server for faster Java compilation"
    );

    println!(
        r"
Subcommands:
  clean                                 Remove build artifacts
  clean-cache                           Clean global bob cache
  build                                 Build the project
  rebuild                               Clean and build the project
  help                                  Print this help message
  run                                   Build and run the build artifact
  rerun                                 Clean, build and run the build artifact
  test                                  Build and run the unit tests
  retest                                Clean, build and run the unit tests
  version                               Print the version number"
    );
}
