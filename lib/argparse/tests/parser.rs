/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Argparse integration tests

use argparse::{ErrorKind, Parser, Subcommand};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Subcommand)]
enum Command {
    #[arg(default, help = "Show help")]
    Help,
    #[arg(help = "Show version")]
    Version,
    #[arg(alias = "r", help = "Run command")]
    Run,
}

#[derive(Debug, Parser)]
#[arg(name = "demo")]
struct Args {
    #[arg(subcommand)]
    command: Command,
    #[arg(short = 'o', long = "output", value = "file", help = "Write output")]
    output: Option<String>,
    #[arg(
        short = 'I',
        long = "include",
        value = "path",
        help = "Add include path"
    )]
    includes: Vec<String>,
    #[arg(short = 'q', long = "quiet", help = "Reduce output")]
    quiet: bool,
    #[arg(positional, value = "input")]
    input: Option<String>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            command: Command::Help,
            output: None,
            includes: Vec::new(),
            quiet: false,
            input: None,
        }
    }
}

#[test]
fn parses_subcommands_options_and_positionals() {
    let args = Args::parse_from([
        "r",
        "-o",
        "out.txt",
        "-I",
        "include",
        "--include",
        "src",
        "-q",
        "main.c",
    ])
    .expect("Parse failed");

    assert_eq!(args.command, Command::Run);
    assert_eq!(args.output.as_deref(), Some("out.txt"));
    assert_eq!(args.includes, ["include", "src"]);
    assert!(args.quiet);
    assert_eq!(args.input.as_deref(), Some("main.c"));
}

#[test]
fn parses_default_help_and_version_commands() {
    let args = Args::parse_from(["--help"]).expect("Parse failed");
    assert_eq!(args.command, Command::Help);

    let args = Args::parse_from(["-v"]).expect("Parse failed");
    assert_eq!(args.command, Command::Version);
}

#[test]
fn returns_errors_for_unknown_args_and_missing_values() {
    let err = Args::parse_from(["--missing"]).expect_err("Expected parse error");
    assert_eq!(err.message(), "Unknown argument: --missing");

    let err = Args::parse_from(["--output"]).expect_err("Expected parse error");
    assert_eq!(err.message(), "Expected value after --output");
}

#[test]
fn generates_help_text() {
    assert_eq!(
        Args::help(),
        "Usage: demo [SUBCOMMAND] <input> [OPTIONS]\n\nOptions:\n  -o <file>, --output <file>     Write output\n  -I <path>, --include <path>    Add include path\n  -q, --quiet                    Reduce output\n  -h, --help                     Print this help message\n  -v, --version                  Print the version number\n\nSubcommands:\n  help       Show help\n  version    Show version\n  run, r     Run command"
    );
}

#[derive(Debug, Parser)]
#[arg(name = "cargo thing")]
struct BoolActionArgs {
    #[arg(short = 'h', long = "help", help = "Print help")]
    help: bool,
    #[arg(short = 'v', long = "version", help = "Print version")]
    version: bool,
}

impl Default for BoolActionArgs {
    fn default() -> Self {
        Self {
            help: true,
            version: false,
        }
    }
}

#[test]
fn supports_bool_help_and_version_fields() {
    let args = BoolActionArgs::parse_from(["version"]).expect("Parse failed");
    assert!(args.version);

    assert!(!BoolActionArgs::help().contains("-h, --help                Print this help message"));
}

#[derive(Debug, Default, Parser)]
#[arg(name = "files")]
struct FilesArgs {
    #[arg(positional, value = "file")]
    files: Vec<String>,
}

#[test]
fn supports_vec_positionals() {
    let args = FilesArgs::parse_from(["a.txt", "b.txt"]).expect("Parse failed");
    assert_eq!(args.files, ["a.txt", "b.txt"]);
}

#[derive(Debug, Default, Parser)]
#[arg(name = "plain")]
struct PlainArgs {
    #[arg(short = 'q', long = "quiet", help = "Reduce output")]
    quiet: bool,
}

#[test]
fn supports_default_help_and_version_without_fields() {
    let err = PlainArgs::parse_from(["--help"]).expect_err("Expected help action");
    assert_eq!(err.kind(), &ErrorKind::Help);
    assert!(err.message().starts_with("Usage: plain [OPTIONS]"));

    let err = PlainArgs::parse_from(["--version"]).expect_err("Expected version action");
    assert_eq!(err.kind(), &ErrorKind::Version);
    assert_eq!(err.message(), "plain v0.1.0");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Subcommand)]
enum DefaultCommand {
    #[arg(default, help = "Serve")]
    Serve,
    #[arg(help = "Show help")]
    Help,
    #[arg(help = "Show version")]
    Version,
}

#[derive(Debug, Parser)]
#[arg(name = "default-command")]
struct DefaultCommandArgs {
    #[arg(subcommand)]
    command: DefaultCommand,
}

impl Default for DefaultCommandArgs {
    fn default() -> Self {
        Self {
            command: DefaultCommand::Serve,
        }
    }
}

#[test]
fn subcommand_default_does_not_steal_help() {
    let args = DefaultCommandArgs::parse_from(["--help"]).expect("Parse failed");
    assert_eq!(args.command, DefaultCommand::Help);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Subcommand)]
enum ScopedCommand {
    #[arg(default, help = "Serve")]
    Serve,
    #[arg(name = "import", help = "Import data")]
    Import,
}

#[derive(Debug, Parser)]
#[arg(name = "scoped")]
struct ScopedArgs {
    #[arg(subcommand)]
    command: ScopedCommand,
    #[arg(positional, command = "import", value = "path")]
    path: Option<String>,
}

impl Default for ScopedArgs {
    fn default() -> Self {
        Self {
            command: ScopedCommand::Serve,
            path: None,
        }
    }
}

#[test]
fn supports_command_scoped_positionals() {
    let args = ScopedArgs::parse_from(["import", "data.zip"]).expect("Parse failed");
    assert_eq!(args.command, ScopedCommand::Import);
    assert_eq!(args.path.as_deref(), Some("data.zip"));

    let err = ScopedArgs::parse_from(["serve", "data.zip"]).expect_err("Expected parse error");
    assert_eq!(err.message(), "Unknown argument: data.zip");
}
