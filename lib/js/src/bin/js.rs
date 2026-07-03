/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../../README.md")]

use std::io;
use std::io::Write;

use argparse::Parser;
use js::Context;

#[derive(Default, Parser)]
#[arg(name = "js")]
struct Args {
    #[arg(long = "verbose", help = "Print verbose output")]
    verbose: bool,
    #[arg(positional, value = "script")]
    text: Option<String>,
}

fn repl(verbose: bool) {
    println!("BassieJS");
    let mut context = Context::new();
    context.set_verbose(verbose);
    loop {
        print!("> ");
        _ = io::stdout().flush();

        let mut text = String::new();
        _ = io::stdin().read_line(&mut text);

        if text == "\n" || text == "\r\n" {
            continue;
        }
        if text == ".exit\n" || text == ".exit\r\n" {
            break;
        }

        match context.eval(text.as_str()) {
            Ok(result) => {
                if verbose {
                    println!("Result: {result:?}");
                } else {
                    println!("{result}");
                }
            }
            Err(err) => println!("Error: {err}"),
        }
    }
}

fn main() {
    // Parse args
    let args = Args::parse();

    // Start repl
    let Some(text) = args.text else {
        repl(args.verbose);
        return;
    };

    // Or execute
    let mut context = Context::new();
    context.set_verbose(args.verbose);
    match context.eval(&text) {
        Ok(result) => {
            if args.verbose {
                println!("Result: {result:?}");
            } else {
                println!("{result:?}");
            }
        }
        Err(err) => println!("Error: {err}"),
    }
}
