/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use argparse::Parser;

#[derive(Debug, Default, Parser)]
#[arg(name = "ccc")]
pub(crate) struct Args {
    #[arg(positional, value = "file")]
    pub(crate) files: Vec<String>,
    #[arg(
        short = 'o',
        long = "output",
        value = "output",
        help = "Write output to file"
    )]
    pub(crate) output: Option<String>,
    #[arg(
        short = 'I',
        long = "include",
        value = "include",
        help = "Add include path"
    )]
    pub(crate) include_paths: Vec<String>,
    #[arg(short = 'S', long = "source", help = "Only generate C source")]
    pub(crate) flag_source: bool,
    #[arg(short = 'c', long = "compile", help = "Compile without linking")]
    pub(crate) flag_compile: bool,
    #[arg(short = 'r', long = "run", help = "Run output after compiling")]
    pub(crate) flag_run: bool,
    #[arg(short = 'R', long = "run-leaks", help = "Run output with leak checks")]
    pub(crate) flag_run_leaks: bool,
}

pub(crate) fn parse_args() -> Args {
    let mut raw_args = Vec::new();
    for arg in std::env::args().skip(1) {
        if let Some(path) = arg.strip_prefix("-I").filter(|path| !path.is_empty()) {
            raw_args.push("-I".to_string());
            raw_args.push(path.to_string());
        } else {
            raw_args.push(arg);
        }
    }

    let args = Args::parse_from(raw_args).unwrap_or_else(|err| err.exit());

    if args.files.is_empty() {
        eprintln!("Usage: ccc <file> [-o output] [-I include] [-S] [-c] [-r] [-R]");
        std::process::exit(1);
    }

    args
}
