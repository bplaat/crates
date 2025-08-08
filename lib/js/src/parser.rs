/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::lexer::Token;

#[derive(Debug)]
pub(crate) enum Node {
    Nodes(Vec<Node>),
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
    BitwiseAnd(Box<Node>, Box<Node>),
    BitwiseOr(Box<Node>, Box<Node>),
    BitwiseXor(Box<Node>, Box<Node>),
    BitwiseNot(Box<Node>),
    LeftShift(Box<Node>, Box<Node>),
    SignedRightShift(Box<Node>, Box<Node>),
    UnsignedRightShift(Box<Node>, Box<Node>),
}

pub(crate) struct Parser<'a> {
    tokens: &'a Vec<Token>,
    position: usize,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(tokens: &'a Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    pub(crate) fn node(&mut self) -> Result<Node, String> {
        self.nodes()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }

    fn next(&mut self) {
        self.position += 1;
    }

    fn nodes(&mut self) -> Result<Node, String> {
        let mut nodes = Vec::new();
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
        Ok(Node::Nodes(nodes))
    }

    fn assign(&mut self) -> Result<Node, String> {
        match self.peek_next() {
            Some(Token::Assign) => {
                let lhs = self.add()?;
                self.next();
                Ok(Node::Assign(Box::new(lhs), Box::new(self.assign()?)))
            }
            _ => self.shift(),
        }
    }

    fn shift(&mut self) -> Result<Node, String> {
        let mut node = self.bitwise()?;
        loop {
            match self.peek() {
                Token::LeftShift => {
                    self.next();
                    node = Node::LeftShift(Box::new(node), Box::new(self.bitwise()?));
                }
                Token::SignedRightShift => {
                    self.next();
                    node = Node::SignedRightShift(Box::new(node), Box::new(self.bitwise()?));
                }
                Token::UnsignedRightShift => {
                    self.next();
                    node = Node::UnsignedRightShift(Box::new(node), Box::new(self.bitwise()?));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn bitwise(&mut self) -> Result<Node, String> {
        let mut node = self.add()?;
        loop {
            match self.peek() {
                Token::BitwiseAnd => {
                    self.next();
                    node = Node::BitwiseAnd(Box::new(node), Box::new(self.add()?));
                }
                Token::BitwiseOr => {
                    self.next();
                    node = Node::BitwiseOr(Box::new(node), Box::new(self.add()?));
                }
                Token::BitwiseXor => {
                    self.next();
                    node = Node::BitwiseXor(Box::new(node), Box::new(self.add()?));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn add(&mut self) -> Result<Node, String> {
        let mut node = self.mul()?;
        loop {
            match self.peek() {
                Token::Add => {
                    self.next();
                    node = Node::Add(Box::new(node), Box::new(self.mul()?));
                }
                Token::Sub => {
                    self.next();
                    node = Node::Sub(Box::new(node), Box::new(self.mul()?));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn mul(&mut self) -> Result<Node, String> {
        let mut node = self.unary()?;
        loop {
            match self.peek() {
                Token::Mul => {
                    self.next();
                    node = Node::Mul(Box::new(node), Box::new(self.unary()?));
                }
                Token::Exp => {
                    self.next();
                    node = Node::Exp(Box::new(node), Box::new(self.unary()?));
                }
                Token::Div => {
                    self.next();
                    node = Node::Div(Box::new(node), Box::new(self.unary()?));
                }
                Token::Mod => {
                    self.next();
                    node = Node::Mod(Box::new(node), Box::new(self.unary()?));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(node)
    }

    fn unary(&mut self) -> Result<Node, String> {
        match self.peek() {
            Token::Add => {
                self.next();
                self.unary()
            }
            Token::Sub => {
                self.next();
                Ok(Node::Neg(Box::new(self.unary()?)))
            }
            Token::BitwiseNot => {
                self.next();
                Ok(Node::BitwiseNot(Box::new(self.unary()?)))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Node, String> {
        match self.peek() {
            Token::LParen => {
                self.next();
                let node = self.add()?;
                self.next();
                Ok(node)
            }
            Token::Number(number) => {
                let node = Node::Number(*number);
                self.next();
                Ok(node)
            }
            Token::Variable(variable) => {
                let node = Node::Variable(variable.clone());
                self.next();
                Ok(node)
            }
            _ => Err(String::from("Parser: unknown node type")),
        }
    }
}
