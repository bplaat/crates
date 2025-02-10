/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! OpenAPI Generator cli

#![forbid(unsafe_code)]

use openapi_generator::{generate_schemas, Generator};

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
            "-i" | "--input" => args.input = args_iter.next().expect("Invalid argument"),
            "-o" | "--output" => args.output = args_iter.next().expect("Invalid argument"),
            "-g" | "--generator" => {
                args.generator = args_iter
                    .next()
                    .expect("Invalid argument")
                    .parse()
                    .expect("Invalid generator");
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
    generate_schemas(&args.input, args.generator, &args.output);
}
