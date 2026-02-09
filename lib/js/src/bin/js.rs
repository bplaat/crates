/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../../README.md")]

use std::io::Write;
use std::{env, io};

use js::Context;

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
                    println!("{}", result);
                }
            }
            Err(err) => println!("Error: {err}"),
        }
    }
}

fn main() {
    // Parse args
    let args: Vec<String> = env::args().skip(1).collect();
    let mut verbose = false;
    let mut text = "";
    for arg in &args {
        if arg == "-v" {
            verbose = true;
        } else {
            text = arg.as_str();
        }
    }

    // Start repl
    if text.is_empty() {
        repl(verbose);
        return;
    }

    // Or execute
    let mut context = Context::new();
    context.set_verbose(verbose);
    match context.eval(text) {
        Ok(result) => {
            if verbose {
                println!("Result: {result:?}");
            } else {
                println!("{result:?}");
            }
        }
        Err(err) => println!("Error: {err}"),
    }
}
