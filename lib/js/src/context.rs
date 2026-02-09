/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::stdlib::env;
use crate::value::Value;

/// Context
#[derive(Default)]
pub struct Context {
    verbose: bool,
    env: Rc<RefCell<IndexMap<String, Value>>>,
}

impl Context {
    /// Create a new context
    pub fn new() -> Self {
        Self {
            verbose: false,
            env: env(),
        }
    }

    /// Set verbose
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Evaluate script
    pub fn eval(&mut self, text: &str) -> Result<Value, String> {
        if self.verbose {
            println!("Text: {text}");
        }

        let tokens = Lexer::new(text).tokens()?;
        if self.verbose {
            print!("Tokens: ");
            for token in &tokens {
                print!("{token:?}, ");
            }
            println!();
        }

        let node = Parser::new(&tokens).parse()?;
        if self.verbose {
            println!("Node: {node:?}");
        }

        Interpreter::new(self.env.clone()).eval(&node)
    }
}
