/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use argparse::Parser;

#[derive(Debug, Parser)]
#[arg(name = "cargo bundle")]
pub(crate) struct Args {
    #[arg(
        short = 'p',
        long = "path",
        value = "dir",
        help = "Build crate in <dir>"
    )]
    pub path: String,
    #[arg(long = "zip", help = "Also create a zip archive")]
    pub zip: bool,
    #[arg(long = "dmg", help = "Also create a DMG installer")]
    pub dmg: bool,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            path: ".".to_string(),
            zip: false,
            dmg: false,
        }
    }
}

pub(crate) fn parse_args() -> Option<Args> {
    let raw_args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|arg| arg != "bundle")
        .collect();
    parse_args_from(raw_args)
}

fn parse_args_from(raw_args: Vec<String>) -> Option<Args> {
    if raw_args.is_empty() {
        return None;
    }
    Some(Args::parse_from(raw_args).unwrap_or_else(|err| err.exit()))
}

pub(crate) fn help() {
    println!("{}", Args::help());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_help_is_handled_by_argparse() {
        let err = Args::parse_from(["--help", "--zip"]).expect_err("Expected help action");
        assert_eq!(err.kind(), &argparse::ErrorKind::Help);
    }

    #[test]
    fn run_options_return_args() {
        let args = parse_args_from(vec!["--zip".to_string()]).expect("Expected args");
        assert!(args.zip);
    }

    #[test]
    fn no_args_returns_help_action() {
        assert!(parse_args_from(Vec::new()).is_none());
    }
}
