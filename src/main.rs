use std::env;
mod lexer;
mod parser;
mod interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("BassieCalc");
        return;
    }

    let text = args[1].as_str();

    let tokens = lexer::lexer(text);
    print!("lexer::Tokens: ");
    for token in &tokens {
        print!("{:?}, ", token);
    }
    println!();

    let node = parser::parser(&tokens);
    println!("Node: {:?}", node);

    let result = interpreter::interpreter(node);
    println!("Result: {}", result);
}
