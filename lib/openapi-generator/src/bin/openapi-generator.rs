/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! OpenAPI Generator cli

use openapi_generator::{generate, Generator};

struct Args {
    input: String,
    generator: Generator,
    output: String,
}

fn parse_args() -> Args {
    let mut args = Args {
        input: "openapi.yaml".to_string(),
        generator: Generator::Rust,
        output: "api.rs".to_string(),
    };
    let mut args_iter = std::env::args().skip(1);
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "-i" | "--input" => {
                if let Some(value) = args_iter.next() {
                    args.input = value;
                }
            }
            "-o" | "--output" => {
                if let Some(value) = args_iter.next() {
                    args.output = value;
                }
            }
            "-g" | "--generator" => {
                if let Some(value) = args_iter.next() {
                    args.generator = value.parse().expect("Invalid generator");
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                std::process::exit(1);
            }
        }
    }
    args
}

fn main() {
    let args = parse_args();
    generate(&args.input, args.generator, &args.output);
}
