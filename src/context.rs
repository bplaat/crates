use crate::interpreter::Interpreter;
use crate::lexer::lexer;
use crate::parser::Parser;
use std::collections::HashMap;

pub struct Context {
    env: HashMap<String, i64>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            env: HashMap::new(),
        }
    }

    pub fn eval(&mut self, text: &str) -> Result<i64, String> {
        let tokens = lexer(text)?;
        print!("Tokens: ");
        for token in &tokens {
            print!("{:?}, ", token);
        }
        println!();

        let node = Parser::new(&tokens).node()?;
        println!("Node: {:?}", node);

        Interpreter::new(&mut self.env).eval(node)
    }
}
