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
    If {
        condition: Box<Node>,
        then_branch: Box<Node>,
        else_branch: Option<Box<Node>>,
    },
    Switch {
        expression: Box<Node>,
        cases: Vec<(Node, Node)>,
        default: Option<Box<Node>>,
    },
    While {
        condition: Box<Node>,
        body: Box<Node>,
    },
    DoWhile {
        body: Box<Node>,
        condition: Box<Node>,
    },
    For {
        init: Option<Box<Node>>,
        condition: Option<Box<Node>>,
        update: Option<Box<Node>>,
        body: Box<Node>,
    },
    Continue,
    Break,

    Value(Value),
    Variable(String),

    Assign(Box<Node>, Box<Node>),
    AddAssign(Box<Node>, Box<Node>),
    SubtractAssign(Box<Node>, Box<Node>),
    MultiplyAssign(Box<Node>, Box<Node>),
    DivideAssign(Box<Node>, Box<Node>),
    RemainderAssign(Box<Node>, Box<Node>),
    ExponentiationAssign(Box<Node>, Box<Node>),
    BitwiseAndAssign(Box<Node>, Box<Node>),
    BitwiseOrAssign(Box<Node>, Box<Node>),
    BitwiseXorAssign(Box<Node>, Box<Node>),
    LeftShiftAssign(Box<Node>, Box<Node>),
    SignedRightShiftAssign(Box<Node>, Box<Node>),
    UnsignedRightShiftAssign(Box<Node>, Box<Node>),

    Ternary {
        condition: Box<Node>,
        then_branch: Box<Node>,
        else_branch: Box<Node>,
    },

    Add(Box<Node>, Box<Node>),
    Subtract(Box<Node>, Box<Node>),
    Multiply(Box<Node>, Box<Node>),
    Divide(Box<Node>, Box<Node>),
    Remainder(Box<Node>, Box<Node>),
    Exponentiation(Box<Node>, Box<Node>),
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
    UnaryMinus(Box<Node>),
    UnaryLogicalNot(Box<Node>),
    UnaryTypeof(Box<Node>),
    UnaryPreIncrement(Box<Node>),
    UnaryPreDecrement(Box<Node>),
    UnaryPostIncrement(Box<Node>),
    UnaryPostDecrement(Box<Node>),
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

    pub(crate) fn parse(&mut self) -> Result<Node, String> {
        self.statements()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn peek_at(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.position + n)
    }

    fn next(&mut self) {
        self.position += 1;
    }

    fn statements(&mut self) -> Result<Node, String> {
        let mut nodes = Vec::new();
        loop {
            match self.peek() {
                Token::Case | Token::Default | Token::RightBrace | Token::Eof => break,
                _ => nodes.push(self.statement()?),
            }

            // Automatic Semicolon Insertion (ASI) rules
            match self.peek() {
                Token::Semicolon => {
                    self.next();
                }
                Token::RightBrace | Token::Eof => {
                    // ASI: semicolon inserted before }, EOF
                    break;
                }
                _ => {
                    // ASI: semicolon inserted at end of line (we assume line breaks here)
                    continue;
                }
            }
        }
        Ok(Node::Nodes(nodes))
    }

    fn block(&mut self) -> Result<Node, String> {
        if let Token::LeftBrace = self.peek() {
            self.next();
            let node = self.statements()?;
            if let Token::RightBrace = self.peek() {
                self.next();
                Ok(node)
            } else {
                Err(String::from("Parser: expected '}' at end of block"))
            }
        } else {
            self.statement()
        }
    }

    fn statement(&mut self) -> Result<Node, String> {
        match self.peek() {
            Token::If => {
                self.next();
                if let Token::LeftParen = self.peek() {
                    self.next();
                    let condition = self.ternary()?;
                    if let Token::RightParen = self.peek() {
                        self.next();
                        let then_branch = self.block()?;
                        let else_branch = if let Token::Else = self.peek() {
                            self.next();
                            Some(Box::new(self.block()?))
                        } else {
                            None
                        };
                        Ok(Node::If {
                            condition: Box::new(condition),
                            then_branch: Box::new(then_branch),
                            else_branch,
                        })
                    } else {
                        Err(String::from("Parser: expected ')' after if condition"))
                    }
                } else {
                    Err(String::from("Parser: expected '(' after 'if'"))
                }
            }
            Token::Switch => {
                self.next();
                if let Token::LeftParen = self.peek() {
                    self.next();
                    let expression = self.ternary()?;
                    if let Token::RightParen = self.peek() {
                        self.next();
                        if let Token::LeftBrace = self.peek() {
                            self.next();
                            let mut cases = Vec::new();
                            let mut default = None;
                            loop {
                                match self.peek() {
                                    Token::Case => {
                                        self.next();
                                        let case_expr = self.ternary()?;
                                        if let Token::Colon = self.peek() {
                                            self.next();
                                            let case_body = if let Token::LeftBrace = self.peek() {
                                                self.block()?
                                            } else {
                                                self.statements()?
                                            };
                                            cases.push((case_expr, case_body));
                                        } else {
                                            return Err(String::from(
                                                "Parser: expected ':' after case expression",
                                            ));
                                        }
                                    }
                                    Token::Default => {
                                        self.next();
                                        if let Token::Colon = self.peek() {
                                            self.next();
                                            let default_body = if let Token::LeftBrace = self.peek()
                                            {
                                                self.block()?
                                            } else {
                                                self.statements()?
                                            };
                                            default = Some(Box::new(default_body));
                                        } else {
                                            return Err(String::from(
                                                "Parser: expected ':' after default",
                                            ));
                                        }
                                    }
                                    Token::RightBrace => {
                                        self.next();
                                        break;
                                    }
                                    _ => {
                                        return Err(String::from(
                                            "Parser: expected 'case', 'default', or '}' in switch statement",
                                        ));
                                    }
                                }
                            }
                            Ok(Node::Switch {
                                expression: Box::new(expression),
                                cases,
                                default,
                            })
                        } else {
                            Err(String::from("Parser: expected '{' after switch expression"))
                        }
                    } else {
                        Err(String::from("Parser: expected ')' after switch expression"))
                    }
                } else {
                    Err(String::from("Parser: expected '(' after 'switch'"))
                }
            }
            Token::While => {
                self.next();
                if let Token::LeftParen = self.peek() {
                    self.next();
                    let condition = self.ternary()?;
                    if let Token::RightParen = self.peek() {
                        self.next();
                        let body = self.block()?;
                        Ok(Node::While {
                            condition: Box::new(condition),
                            body: Box::new(body),
                        })
                    } else {
                        Err(String::from("Parser: expected ')' after while condition"))
                    }
                } else {
                    Err(String::from("Parser: expected '(' after 'while'"))
                }
            }
            Token::Do => {
                self.next();
                let body = self.block()?;
                if let Token::While = self.peek() {
                    self.next();
                    if let Token::LeftParen = self.peek() {
                        self.next();
                        let condition = self.ternary()?;
                        if let Token::RightParen = self.peek() {
                            self.next();
                            Ok(Node::DoWhile {
                                body: Box::new(body),
                                condition: Box::new(condition),
                            })
                        } else {
                            Err(String::from(
                                "Parser: expected ')' after do-while condition",
                            ))
                        }
                    } else {
                        Err(String::from(
                            "Parser: expected '(' after 'while' in do-while",
                        ))
                    }
                } else {
                    Err(String::from("Parser: expected 'while' after do block"))
                }
            }
            Token::For => {
                self.next();
                if let Token::LeftParen = self.peek() {
                    self.next();
                    let init = if let Token::Semicolon = self.peek() {
                        None
                    } else {
                        Some(self.comma()?)
                    };
                    if let Token::Semicolon = self.peek() {
                        self.next();
                        let condition = if let Token::Semicolon = self.peek() {
                            None
                        } else {
                            Some(self.comma()?)
                        };
                        if let Token::Semicolon = self.peek() {
                            self.next();
                            let update = if let Token::RightParen = self.peek() {
                                None
                            } else {
                                Some(self.comma()?)
                            };
                            if let Token::RightParen = self.peek() {
                                self.next();
                                let body = self.block()?;

                                Ok(Node::For {
                                    init: init.map(Box::new),
                                    condition: condition.map(Box::new),
                                    update: update.map(Box::new),
                                    body: Box::new(body),
                                })
                            } else {
                                Err(String::from("Parser: expected ')' after for loop"))
                            }
                        } else {
                            Err(String::from("Parser: expected ';' after for condition"))
                        }
                    } else {
                        Err(String::from("Parser: expected ';' after for init"))
                    }
                } else {
                    Err(String::from("Parser: expected '(' after 'for'"))
                }
            }
            Token::Break => {
                self.next();
                Ok(Node::Break)
            }
            Token::Continue => {
                self.next();
                Ok(Node::Continue)
            }
            _ => self.comma(),
        }
    }

    fn comma(&mut self) -> Result<Node, String> {
        let node = self.assign()?;
        if let Token::Comma = self.peek() {
            let mut nodes = vec![node];
            while let Token::Comma = self.peek() {
                self.next();
                nodes.push(self.assign()?);
            }
            Ok(Node::Nodes(nodes))
        } else {
            Ok(node)
        }
    }

    fn assign(&mut self) -> Result<Node, String> {
        // Handle variable declarations
        if let Token::Var | Token::Let | Token::Const = self.peek() {
            self.next();
        }

        match self.peek_at(1) {
            Some(Token::Assign) => {
                let lhs = self.ternary()?;
                self.next();
                Ok(Node::Assign(Box::new(lhs), Box::new(self.assign()?)))
            }
            Some(Token::AddAssign) => {
                let lhs = self.ternary()?;
                self.next();
                Ok(Node::AddAssign(Box::new(lhs), Box::new(self.assign()?)))
            }
            Some(Token::SubtractAssign) => {
                let lhs = self.ternary()?;
                self.next();
                Ok(Node::SubtractAssign(
                    Box::new(lhs),
                    Box::new(self.assign()?),
                ))
            }
            Some(Token::MultiplyAssign) => {
                let lhs = self.ternary()?;
                self.next();
                Ok(Node::MultiplyAssign(
                    Box::new(lhs),
                    Box::new(self.assign()?),
                ))
            }
            Some(Token::DivideAssign) => {
                let lhs = self.ternary()?;
                self.next();
                Ok(Node::DivideAssign(Box::new(lhs), Box::new(self.assign()?)))
            }
            Some(Token::RemainderAssign) => {
                let lhs = self.ternary()?;
                self.next();
                Ok(Node::RemainderAssign(
                    Box::new(lhs),
                    Box::new(self.assign()?),
                ))
            }
            Some(Token::ExponentiationAssign) => {
                let lhs = self.ternary()?;
                self.next();
                Ok(Node::ExponentiationAssign(
                    Box::new(lhs),
                    Box::new(self.assign()?),
                ))
            }
            Some(Token::BitwiseAndAssign) => {
                let lhs = self.ternary()?;
                self.next();
                Ok(Node::BitwiseAndAssign(
                    Box::new(lhs),
                    Box::new(self.assign()?),
                ))
            }
            Some(Token::BitwiseOrAssign) => {
                let lhs = self.ternary()?;
                self.next();
                Ok(Node::BitwiseOrAssign(
                    Box::new(lhs),
                    Box::new(self.assign()?),
                ))
            }
            Some(Token::BitwiseXorAssign) => {
                let lhs = self.ternary()?;
                self.next();
                Ok(Node::BitwiseXorAssign(
                    Box::new(lhs),
                    Box::new(self.assign()?),
                ))
            }
            Some(Token::LeftShiftAssign) => {
                let lhs = self.ternary()?;
                self.next();
                Ok(Node::LeftShiftAssign(
                    Box::new(lhs),
                    Box::new(self.assign()?),
                ))
            }
            Some(Token::SignedRightShiftAssign) => {
                let lhs = self.ternary()?;
                self.next();
                Ok(Node::SignedRightShiftAssign(
                    Box::new(lhs),
                    Box::new(self.assign()?),
                ))
            }
            Some(Token::UnsignedRightShiftAssign) => {
                let lhs = self.ternary()?;
                self.next();
                Ok(Node::UnsignedRightShiftAssign(
                    Box::new(lhs),
                    Box::new(self.assign()?),
                ))
            }
            _ => self.ternary(),
        }
    }

    fn ternary(&mut self) -> Result<Node, String> {
        let condition = self.logical()?;
        if let Token::Question = self.peek() {
            self.next();
            let if_branch = self.ternary()?;
            if let Token::Colon = self.peek() {
                self.next();
                let else_branch = self.ternary()?;
                Ok(Node::Ternary {
                    condition: Box::new(condition),
                    then_branch: Box::new(if_branch),
                    else_branch: Box::new(else_branch),
                })
            } else {
                Err(String::from("Parser: expected ':' in ternary expression"))
            }
        } else {
            Ok(condition)
        }
    }

    fn logical(&mut self) -> Result<Node, String> {
        let mut node = self.relational()?;
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
        let mut node = self.equality()?;
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
                Token::Exponentiation => {
                    self.next();
                    node = Node::Exponentiation(Box::new(node), Box::new(self.unary()?));
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
                Ok(Node::UnaryLogicalNot(Box::new(self.unary()?)))
            }
            Token::Increment => {
                self.next();
                Ok(Node::UnaryPreIncrement(Box::new(self.unary()?)))
            }
            Token::Decrement => {
                self.next();
                Ok(Node::UnaryPreDecrement(Box::new(self.unary()?)))
            }
            Token::Typeof => {
                self.next();
                Ok(Node::UnaryTypeof(Box::new(self.unary()?)))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Node, String> {
        match self.peek() {
            Token::LeftParen => {
                self.next();
                let node = self.ternary()?;
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
            Token::String(string) => {
                let node = Node::Value(Value::String(string.clone()));
                self.next();
                Ok(node)
            }
            Token::Variable(variable) => {
                let node = Node::Variable(variable.clone());
                self.next();
                if let Token::Increment = self.peek() {
                    self.next();
                    Ok(Node::UnaryPostIncrement(Box::new(node)))
                } else if let Token::Decrement = self.peek() {
                    self.next();
                    Ok(Node::UnaryPostDecrement(Box::new(node)))
                } else {
                    Ok(node)
                }
            }
            _ => Err(format!("Parser: unknown node type: {:?}", self.peek())),
        }
    }
}
