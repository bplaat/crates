use crate::interpreter::Interpreter;
use crate::lexer::lexer;
use crate::parser::Parser;
use std::collections::HashMap;

pub struct Context {
    verbose: bool,
    env: HashMap<String, i64>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            verbose: false,
            env: HashMap::new(),
        }
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    pub fn eval(&mut self, text: &str) -> Result<i64, String> {
        if self.verbose {
            println!("Text: {}", text);
        }

        let tokens = lexer(text)?;
        if self.verbose {
            print!("Tokens: ");
            for token in &tokens {
                print!("{:?}, ", token);
            }
            println!();
        }

        let node = Parser::new(&tokens).node()?;
        if self.verbose {
            println!("Node: {:?}", node);
        }

        Interpreter::new(&mut self.env).eval(node)
    }
}
