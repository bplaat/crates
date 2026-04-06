/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use clap::Parser;

#[derive(Parser)]
#[command(about = "C compiler and run helper")]
pub(crate) struct Args {
    /// Input source files (positional)
    pub(crate) files: Vec<String>,

    #[arg(short = 'o', long, help = "Output file", value_name = "FILE")]
    pub(crate) output: Option<String>,

    #[arg(short = 'I', long = "include", help = "Add include search path", value_name = "PATH")]
    pub(crate) include_paths: Vec<String>,

    #[arg(short = 'S', long = "source", help = "Output assembly source")]
    pub(crate) flag_source: bool,

    #[arg(short = 'c', long = "compile", help = "Output compiled object file")]
    pub(crate) flag_compile: bool,

    #[arg(short = 'r', long = "run", help = "Run after compiling")]
    pub(crate) flag_run: bool,

    #[arg(short = 'R', long = "run-leaks", help = "Run with leak detection")]
    pub(crate) flag_run_leaks: bool,
}
