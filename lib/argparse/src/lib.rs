/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A small derive-based command line argument parser

use std::error;
use std::fmt::{self, Display, Formatter};
use std::process::exit;

#[cfg(feature = "derive")]
pub use argparse_derive::{Parser, Subcommand};

/// Parser result
pub type Result<T> = std::result::Result<T, Error>;

/// Parser error
#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    message: String,
    kind: ErrorKind,
}

/// Parser error kind
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Normal parse error
    Message,
    /// Help output requested
    Help,
    /// Version output requested
    Version,
}

impl Error {
    /// Create a new parser error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ErrorKind::Message,
        }
    }

    /// Create help output action
    pub fn help(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ErrorKind::Help,
        }
    }

    /// Create version output action
    pub fn version(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ErrorKind::Version,
        }
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the error kind
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Print the error or action and exit
    pub fn exit(self) -> ! {
        if matches!(self.kind, ErrorKind::Help | ErrorKind::Version) {
            println!("{self}");
            exit(0);
        } else {
            eprintln!("{self}");
            exit(1);
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for Error {}

/// Command metadata
#[derive(Clone, Copy)]
pub struct Command {
    /// Command name
    pub name: &'static str,
    /// Command aliases
    pub aliases: &'static [&'static str],
    /// Command help text
    pub help: &'static str,
}

/// Option metadata
#[derive(Clone, Copy)]
pub struct OptionSpec {
    /// Short option
    pub short: Option<char>,
    /// Long option
    pub long: &'static str,
    /// Value placeholder
    pub value: Option<&'static str>,
    /// Option help text
    pub help: &'static str,
}

/// Parser trait
pub trait Parser: Sized {
    /// Parse from process arguments
    fn parse() -> Self {
        match Self::parse_from(std::env::args().skip(1)) {
            Ok(args) => args,
            Err(err) => err.exit(),
        }
    }

    /// Parse from an iterator of arguments
    fn parse_from<I, S>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>;

    /// Generate help text
    fn help() -> String;

    /// Generate version text
    fn version() -> String;
}

/// Subcommand trait
pub trait Subcommand: Sized {
    /// Parse subcommand name or alias
    fn parse_subcommand(command: &str) -> Option<Self>;

    /// Get all subcommands
    fn subcommands() -> &'static [Command];

    /// Get help command value
    fn help_command() -> Option<Self>;

    /// Get version command value
    fn version_command() -> Option<Self>;
}

/// Format parser help
pub fn format_help(
    usage: &str,
    options: &[OptionSpec],
    subcommands: &[Command],
    include_help: bool,
    include_version: bool,
) -> String {
    let mut output = String::new();
    output.push_str("Usage: ");
    output.push_str(usage);
    output.push('\n');

    let mut options = options.to_vec();
    if include_help {
        options.push(OptionSpec {
            short: Some('h'),
            long: "help",
            value: None,
            help: "Print this help message",
        });
    }
    if include_version {
        options.push(OptionSpec {
            short: Some('v'),
            long: "version",
            value: None,
            help: "Print the version number",
        });
    }

    if !options.is_empty() {
        output.push_str("\nOptions:\n");
        let rows: Vec<(String, &str)> = options
            .iter()
            .map(|option| (format_option_usage(option), option.help))
            .collect();
        push_aligned_rows(&mut output, &rows);
    }

    if !subcommands.is_empty() {
        output.push_str("\nSubcommands:\n");
        let rows: Vec<(String, &str)> = subcommands
            .iter()
            .map(|command| (format_command_usage(command), command.help))
            .collect();
        push_aligned_rows(&mut output, &rows);
    }

    output.trim_end().to_string()
}

fn format_command_usage(command: &Command) -> String {
    if command.aliases.is_empty() {
        command.name.to_string()
    } else {
        let mut usage = command.name.to_string();
        for alias in command.aliases {
            usage.push_str(", ");
            usage.push_str(alias);
        }
        usage
    }
}

fn format_option_usage(option: &OptionSpec) -> String {
    let value = option.value.map(|value| format!(" <{value}>"));
    match (option.short, option.long.is_empty(), value) {
        (Some(short), false, Some(value)) => format!("-{short}{value}, --{}{value}", option.long),
        (Some(short), false, None) => format!("-{short}, --{}", option.long),
        (Some(short), true, Some(value)) => format!("-{short}{value}"),
        (Some(short), true, None) => format!("-{short}"),
        (None, false, Some(value)) => format!("--{}{value}", option.long),
        (None, false, None) => format!("--{}", option.long),
        (None, true, Some(value)) => value.trim_start().to_string(),
        (None, true, None) => String::new(),
    }
}

fn push_aligned_rows(output: &mut String, rows: &[(String, &str)]) {
    let width = rows.iter().map(|(usage, _)| usage.len()).max().unwrap_or(0);
    for (usage, help) in rows {
        output.push_str("  ");
        output.push_str(usage);
        if help.is_empty() {
            output.push('\n');
        } else {
            for _ in 0..(width - usage.len() + 4) {
                output.push(' ');
            }
            output.push_str(help);
            output.push('\n');
        }
    }
}
