use std::env;
use std::io;
use std::io::Write;
use std::collections::HashMap;

mod interpreter;
mod lexer;
mod parser;

fn repl() {
    println!("BassieCalc");
    let mut env = HashMap::new();
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

        let tokens = lexer::lexer(text.as_str());
        print!("Tokens: ");
        for token in &tokens {
            print!("{:?}, ", token);
        }
        println!();

        let node = parser::parser(&tokens);
        println!("Node: {:?}", node);

        let result = interpreter::interpreter(&mut env, node);
        println!("Result: {}", result);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        repl();
        return;
    }

    let text = args[1].as_str();

    let tokens = lexer::lexer(text);
    print!("Tokens: ");
    for token in &tokens {
        print!("{:?}, ", token);
    }
    println!();

    let node = parser::parser(&tokens);
    println!("Node: {:?}", node);

    let mut env = HashMap::new();
    let result = interpreter::interpreter(&mut env, node);
    println!("Result: {}", result);
}
