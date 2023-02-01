use crate::lexer::Token;

#[derive(Debug)]
pub enum Node {
    Nodes(Vec<Box<Node>>),
    Number(i64),
    Variable(String),
    Neg(Box<Node>),
    Assign(Box<Node>, Box<Node>),
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Exp(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Mod(Box<Node>, Box<Node>),
}

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Parser {
            tokens: tokens,
            position: 0,
        }
    }

    pub fn node(&mut self) -> Result<Box<Node>, String> {
        self.nodes()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn next(&mut self) {
        self.position += 1;
    }

    fn nodes(&mut self) -> Result<Box<Node>, String> {
        let mut nodes = vec![];
        loop {
            nodes.push(self.assign()?);
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
        Ok(Box::new(Node::Nodes(nodes)))
    }

    fn assign(&mut self) -> Result<Box<Node>, String> {
        if self.position + 1 >= self.tokens.len() {
            return self.add();
        }
        match self.tokens[self.position + 1] {
            Token::Assign => {
                let lhs = self.add()?;
                self.next();
                Ok(Box::new(Node::Assign(lhs, self.assign()?)))
            }
            _ => self.add(),
        }
    }

    fn add(&mut self) -> Result<Box<Node>, String> {
        let mut node = self.mul()?;
        loop {
            match self.peek() {
                Token::Add => {
                    self.next();
                    node = Box::new(Node::Add(node, self.mul()?));
                }
                Token::Sub => {
                    self.next();
                    node = Box::new(Node::Sub(node, self.mul()?));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn mul(&mut self) -> Result<Box<Node>, String> {
        let mut node = self.unary()?;
        loop {
            match self.peek() {
                Token::Mul => {
                    self.next();
                    node = Box::new(Node::Mul(node, self.unary()?));
                }
                Token::Exp => {
                    self.next();
                    node = Box::new(Node::Exp(node, self.unary()?));
                }
                Token::Div => {
                    self.next();
                    node = Box::new(Node::Div(node, self.unary()?));
                }
                Token::Mod => {
                    self.next();
                    node = Box::new(Node::Mod(node, self.unary()?));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn unary(&mut self) -> Result<Box<Node>, String> {
        match self.peek() {
            Token::Add => {
                self.next();
                self.unary()
            }
            Token::Sub => {
                self.next();
                Ok(Box::new(Node::Neg(self.unary()?)))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Box<Node>, String> {
        match self.peek() {
            Token::LParen => {
                self.next();
                let node = self.add()?;
                self.next();
                Ok(node)
            }
            Token::Number(number) => {
                let node = Box::new(Node::Number(*number));
                self.next();
                Ok(node)
            }
            Token::Variable(variable) => {
                let node = Box::new(Node::Variable(variable.clone()));
                self.next();
                Ok(node)
            }
            _ => Err(String::from("Parser: unknown node type")),
        }
    }
}
