use std::env;
use std::io;
use std::io::Write;

mod context;
mod interpreter;
mod lexer;
mod parser;
use crate::context::Context;

fn repl() {
    println!("BassieCalc");
    let mut context = Context::new();
    loop {
        let mut text = String::new();
        print!("> ");
        _ = io::stdout().flush();
        _ = io::stdin().read_line(&mut text);

        if text == "\n" {
            continue;
        }
        if text == ".exit\n" {
            break;
        }

        println!("Result: {}", context.eval(text.as_str()));
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        repl();
        return;
    }

    let mut context = Context::new();
    println!("Result: {}", context.eval(args[1].as_str()));
}
