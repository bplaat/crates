/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::env;
use std::process::exit;

#[derive(PartialEq, Eq)]
pub(crate) enum Subcommand {
    Serve,
    ImportGoogleKeep,
    Help,
    Version,
}

pub(crate) struct Args {
    pub subcommand: Subcommand,
    pub path: Option<String>,
    pub email: Option<String>,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            subcommand: Subcommand::Serve,
            path: None,
            email: None,
        }
    }
}

pub(crate) fn parse_args() -> Args {
    let mut args = Args::default();
    let mut args_iter = env::args().skip(1);
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "serve" => args.subcommand = Subcommand::Serve,
            "import-google-keep" => {
                args.subcommand = Subcommand::ImportGoogleKeep;
                args.path = args_iter.next();
            }
            "--email" => {
                args.email = args_iter.next();
            }
            "help" | "-h" | "--help" => args.subcommand = Subcommand::Help,
            "version" | "--version" => args.subcommand = Subcommand::Version,
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
        "Usage: plaatnotes [SUBCOMMAND]

Subcommands:
  serve                   Start the HTTP server (default)
  import-google-keep      Import notes from a Google Keep Takeout folder or zip
    --email <email>         Email of the user to import notes for
  help                    Print this help message
  version                 Print the version number"
    );
}
