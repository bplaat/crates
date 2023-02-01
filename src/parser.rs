use crate::lexer::Token;

#[derive(Debug)]
pub enum Node {
    Nodes(Vec<Box<Node>>),
    Number(i64),
    Variable(String),
    Assign(Box<Node>, Box<Node>),
    Neg(Box<Node>),
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Exp(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Mod(Box<Node>, Box<Node>),
}

struct Parser<'a> {
    tokens: &'a Vec<Token>,
    position: usize,
}

pub fn parser(tokens: &Vec<Token>) -> Box<Node> {
    let mut parser = Box::new(Parser {
        tokens,
        position: 0,
    });
    return parser_nodes(&mut parser);
}

fn parser_nodes(parser: &mut Box<Parser>) -> Box<Node> {
    let mut nodes = vec![];
    loop {
        nodes.push(parser_assign(parser));
        match parser.tokens[parser.position] {
            Token::Comma => {
                parser.position += 1;
            }
            Token::Semicolon => {
                parser.position += 1;
            }
            _ => {
                break;
            }
        }
    }
    return Box::new(Node::Nodes(nodes));
}

fn parser_assign(parser: &mut Box<Parser>) -> Box<Node> {
    let mut node = parser_add(parser);
    loop {
        match parser.tokens[parser.position] {
            Token::Assign => {
                parser.position += 1;
                node = Box::new(Node::Assign(node, parser_add(parser)));
            }
            _ => {
                break;
            }
        }
    }
    return node;
}

fn parser_add(parser: &mut Box<Parser>) -> Box<Node> {
    let mut node = parser_mul(parser);
    loop {
        match parser.tokens[parser.position] {
            Token::Add => {
                parser.position += 1;
                node = Box::new(Node::Add(node, parser_mul(parser)));
            }
            Token::Sub => {
                parser.position += 1;
                node = Box::new(Node::Sub(node, parser_mul(parser)));
            }
            _ => {
                break;
            }
        }
    }
    return node;
}

fn parser_mul(parser: &mut Box<Parser>) -> Box<Node> {
    let mut node = parser_unary(parser);
    loop {
        match parser.tokens[parser.position] {
            Token::Mul => {
                parser.position += 1;
                node = Box::new(Node::Mul(node, parser_unary(parser)));
            }
            Token::Exp => {
                parser.position += 1;
                node = Box::new(Node::Exp(node, parser_unary(parser)));
            }
            Token::Div => {
                parser.position += 1;
                node = Box::new(Node::Div(node, parser_unary(parser)));
            }
            Token::Mod => {
                parser.position += 1;
                node = Box::new(Node::Mod(node, parser_unary(parser)));
            }
            _ => {
                break;
            }
        }
    }
    return node;
}

fn parser_unary(parser: &mut Box<Parser>) -> Box<Node> {
    match &parser.tokens[parser.position] {
        Token::Add => {
            parser.position += 1;
            parser_unary(parser)
        }
        Token::Sub => {
            parser.position += 1;
            Box::new(Node::Neg(parser_unary(parser)))
        }
        _ => parser_primary(parser),
    }
}

fn parser_primary(parser: &mut Box<Parser>) -> Box<Node> {
    match &parser.tokens[parser.position] {
        Token::LParen => {
            parser.position += 1;
            let node = parser_add(parser);
            parser.position += 1;
            return node;
        }
        Token::Number(number) => {
            parser.position += 1;
            Box::new(Node::Number(*number))
        }
        Token::Variable(variable) => {
            parser.position += 1;
            Box::new(Node::Variable(variable.clone()))
        }
        _ => panic!("Unknown node type"),
    }
}
