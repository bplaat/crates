/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! OpenAPI Generator cli

use argparse::Parser;
use openapi_generator::{Generator, generate_schemas};

#[derive(Parser)]
#[arg(name = "openapi-generator")]
struct Args {
    #[arg(
        short = 'i',
        long = "input",
        value = "file",
        help = "Read OpenAPI schema from file"
    )]
    input: String,
    #[arg(
        short = 'g',
        long = "generator",
        value = "generator",
        help = "Generator to use"
    )]
    generator: Generator,
    #[arg(
        short = 'o',
        long = "output",
        value = "file",
        help = "Write generated output to file"
    )]
    output: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            input: "openapi.yaml".to_string(),
            generator: Generator::Rust,
            output: "api.rs".to_string(),
        }
    }
}

fn main() {
    let args = Args::parse();
    generate_schemas(&args.input, &args.output, args.generator);
}
