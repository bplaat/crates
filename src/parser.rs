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

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    pub fn node(&mut self) -> Box<Node> {
        self.nodes()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn next(&mut self) {
        self.position += 1;
    }

    fn nodes(&mut self) -> Box<Node> {
        let mut nodes = vec![];
        loop {
            nodes.push(self.assign());
            match self.peek() {
                Token::Comma => {
                    self.next();
                }
                Token::Semicolon => {
                    self.next();
                }
                _ => {
                    break;
                }
            }
        }
        Box::new(Node::Nodes(nodes))
    }

    fn assign(&mut self) -> Box<Node> {
        let mut node = self.add();
        loop {
            match self.peek() {
                Token::Assign => {
                    self.next();
                    node = Box::new(Node::Assign(node, self.add()));
                }
                _ => {
                    break;
                }
            }
        }
        return node;
    }

    fn add(&mut self) -> Box<Node> {
        let mut node = self.mul();
        loop {
            match self.peek() {
                Token::Add => {
                    self.next();
                    node = Box::new(Node::Add(node, self.mul()));
                }
                Token::Sub => {
                    self.next();
                    node = Box::new(Node::Sub(node, self.mul()));
                }
                _ => {
                    break;
                }
            }
        }
        return node;
    }

    fn mul(&mut self) -> Box<Node> {
        let mut node = self.unary();
        loop {
            match self.peek() {
                Token::Mul => {
                    self.next();
                    node = Box::new(Node::Mul(node, self.unary()));
                }
                Token::Exp => {
                    self.next();
                    node = Box::new(Node::Exp(node, self.unary()));
                }
                Token::Div => {
                    self.next();
                    node = Box::new(Node::Div(node, self.unary()));
                }
                Token::Mod => {
                    self.next();
                    node = Box::new(Node::Mod(node, self.unary()));
                }
                _ => {
                    break;
                }
            }
        }
        return node;
    }

    fn unary(&mut self) -> Box<Node> {
        match self.peek() {
            Token::Add => {
                self.next();
                self.unary()
            }
            Token::Sub => {
                self.next();
                Box::new(Node::Neg(self.unary()))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Box<Node> {
        match self.peek() {
            Token::LParen => {
                self.next();
                let node = self.add();
                self.next();
                return node;
            }
            Token::Number(number) => {
                let node = Box::new(Node::Number(*number));
                self.next();
                node
            }
            Token::Variable(variable) => {
                let node = Box::new(Node::Variable(variable.clone()));
                self.next();
                node
            }
            _ => panic!("Unknown node type"),
        }
    }
}
