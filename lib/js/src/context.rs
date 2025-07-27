/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;

use crate::interpreter::Interpreter;
use crate::lexer::lexer;
use crate::parser::Parser;

/// Context
#[derive(Default)]
pub struct Context {
    verbose: bool,
    env: HashMap<String, i64>,
}

impl Context {
    /// Create a new context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set verbose
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Evaluate script
    pub fn eval(&mut self, text: &str) -> Result<i64, String> {
        if self.verbose {
            println!("Text: {text}");
        }

        let tokens = lexer(text)?;
        if self.verbose {
            print!("Tokens: ");
            for token in &tokens {
                print!("{token:?}, ");
            }
            println!();
        }

        let node = Parser::new(&tokens).node()?;
        if self.verbose {
            println!("Node: {node:?}");
        }

        Interpreter::new(&mut self.env).eval(&node)
    }
}
