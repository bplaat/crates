/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::Value;
use crate::lexer::Token;

#[derive(Debug)]
pub(crate) enum Node {
    Nodes(Vec<Node>),
    Value(Value),
    Variable(String),
    Assign(Box<Node>, Box<Node>),

    UnaryMinus(Box<Node>),
    Add(Box<Node>, Box<Node>),
    Subtract(Box<Node>, Box<Node>),
    Multiply(Box<Node>, Box<Node>),
    Divide(Box<Node>, Box<Node>),
    Remainder(Box<Node>, Box<Node>),
    Exponentiate(Box<Node>, Box<Node>),
    BitwiseAnd(Box<Node>, Box<Node>),
    BitwiseOr(Box<Node>, Box<Node>),
    BitwiseXor(Box<Node>, Box<Node>),
    BitwiseNot(Box<Node>),
    LeftShift(Box<Node>, Box<Node>),
    SignedRightShift(Box<Node>, Box<Node>),
    UnsignedRightShift(Box<Node>, Box<Node>),
    Equals(Box<Node>, Box<Node>),
    StrictEquals(Box<Node>, Box<Node>),
    NotEquals(Box<Node>, Box<Node>),
    StrictNotEquals(Box<Node>, Box<Node>),
    LessThen(Box<Node>, Box<Node>),
    LessThenEquals(Box<Node>, Box<Node>),
    GreaterThen(Box<Node>, Box<Node>),
    GreaterThenEquals(Box<Node>, Box<Node>),
    LogicalAnd(Box<Node>, Box<Node>),
    LogicalOr(Box<Node>, Box<Node>),
    LogicalNot(Box<Node>),
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
            _ => self.logical(),
        }
    }

    fn logical(&mut self) -> Result<Node, String> {
        let mut node = self.shift()?;
        loop {
            match self.peek() {
                Token::LogicalAnd => {
                    self.next();
                    node = Node::LogicalAnd(Box::new(node), Box::new(self.relational()?));
                }
                Token::LogicalOr => {
                    self.next();
                    node = Node::LogicalOr(Box::new(node), Box::new(self.relational()?));
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn relational(&mut self) -> Result<Node, String> {
        let mut node = self.shift()?;
        loop {
            match self.peek() {
                Token::LessThen => {
                    self.next();
                    node = Node::LessThen(Box::new(node), Box::new(self.equality()?));
                }
                Token::LessThenEquals => {
                    self.next();
                    node = Node::LessThenEquals(Box::new(node), Box::new(self.equality()?));
                }
                Token::GreaterThen => {
                    self.next();
                    node = Node::GreaterThen(Box::new(node), Box::new(self.equality()?));
                }
                Token::GreaterThenEquals => {
                    self.next();
                    node = Node::GreaterThenEquals(Box::new(node), Box::new(self.equality()?));
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn equality(&mut self) -> Result<Node, String> {
        let mut node = self.shift()?;
        loop {
            match self.peek() {
                Token::Equals => {
                    self.next();
                    node = Node::Equals(Box::new(node), Box::new(self.shift()?));
                }
                Token::StrictEquals => {
                    self.next();
                    node = Node::StrictEquals(Box::new(node), Box::new(self.shift()?));
                }
                Token::NotEquals => {
                    self.next();
                    node = Node::NotEquals(Box::new(node), Box::new(self.shift()?));
                }
                Token::StrictNotEquals => {
                    self.next();
                    node = Node::StrictNotEquals(Box::new(node), Box::new(self.shift()?));
                }
                _ => break,
            }
        }
        Ok(node)
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
                _ => break,
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
                _ => break,
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
                Token::Subtract => {
                    self.next();
                    node = Node::Subtract(Box::new(node), Box::new(self.mul()?));
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn mul(&mut self) -> Result<Node, String> {
        let mut node = self.unary()?;
        loop {
            match self.peek() {
                Token::Multiply => {
                    self.next();
                    node = Node::Multiply(Box::new(node), Box::new(self.unary()?));
                }
                Token::Divide => {
                    self.next();
                    node = Node::Divide(Box::new(node), Box::new(self.unary()?));
                }
                Token::Remainder => {
                    self.next();
                    node = Node::Remainder(Box::new(node), Box::new(self.unary()?));
                }
                Token::Exponentiate => {
                    self.next();
                    node = Node::Exponentiate(Box::new(node), Box::new(self.unary()?));
                }
                _ => break,
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
            Token::Subtract => {
                self.next();
                Ok(Node::UnaryMinus(Box::new(self.unary()?)))
            }
            Token::BitwiseNot => {
                self.next();
                Ok(Node::BitwiseNot(Box::new(self.unary()?)))
            }
            Token::LogicalNot => {
                self.next();
                Ok(Node::LogicalNot(Box::new(self.unary()?)))
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
            Token::Undefined => {
                self.next();
                Ok(Node::Value(Value::Undefined))
            }
            Token::Null => {
                self.next();
                Ok(Node::Value(Value::Null))
            }
            Token::Boolean(boolean) => {
                let node = Node::Value(Value::Boolean(*boolean));
                self.next();
                Ok(node)
            }
            Token::Number(number) => {
                let node = Node::Value(Value::Number(*number));
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
