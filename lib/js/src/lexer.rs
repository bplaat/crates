/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[derive(Debug, Clone)]
pub(crate) enum Token {
    Eof,

    Undefined,
    Null,
    Number(i64),
    Variable(String),
    Boolean(bool),

    LParen,
    RParen,
    Comma,
    Semicolon,

    Assign,
    Add,
    Sub,
    Mul,
    Exp,
    Div,
    Mod,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    BitwiseNot,
    LeftShift,
    SignedRightShift,
    UnsignedRightShift,
}

struct Keyword {
    keyword: &'static str,
    token: Token,
}

impl Keyword {
    fn new(keyword: &'static str, token: Token) -> Self {
        Keyword { keyword, token }
    }
}

pub(crate) fn lexer(text: &str) -> Result<Vec<Token>, String> {
    let keywords = [
        Keyword::new("undefined", Token::Undefined),
        Keyword::new("null", Token::Null),
        Keyword::new("true", Token::Boolean(true)),
        Keyword::new("false", Token::Boolean(false)),
    ];
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(char) = chars.next() {
        if char.is_whitespace() {
            continue;
        }

        if char.is_ascii_digit() {
            let mut number = String::from(char);
            while let Some(char) = chars.peek() {
                if !char.is_ascii_digit() {
                    break;
                }
                number.push(chars.next().expect("Invalid number"));
            }
            tokens.push(Token::Number(number.parse().expect("Invalid number")));
            continue;
        }

        if char.is_alphabetic() {
            let mut variable = String::from(char);
            while let Some(char) = chars.peek() {
                if !char.is_alphanumeric() {
                    break;
                }
                variable.push(chars.next().expect("Invalid variable"));
            }

            let mut found = false;
            for keyword in &keywords {
                if keyword.keyword == variable {
                    tokens.push(keyword.token.clone());
                    found = true;
                    break;
                }
            }
            if !found {
                tokens.push(Token::Variable(variable));
            }
            continue;
        }

        if char == '(' {
            tokens.push(Token::LParen);
            continue;
        }
        if char == ')' {
            tokens.push(Token::RParen);
            continue;
        }
        if char == ',' {
            tokens.push(Token::Comma);
            continue;
        }
        if char == ';' {
            tokens.push(Token::Semicolon);
            continue;
        }
        if char == '=' {
            tokens.push(Token::Assign);
            continue;
        }
        if char == '+' {
            tokens.push(Token::Add);
            continue;
        }
        if char == '-' {
            tokens.push(Token::Sub);
            continue;
        }
        if char == '*' {
            if let Some(c) = chars.peek() {
                if *c == '*' {
                    chars.next();
                    tokens.push(Token::Exp);
                    continue;
                }
            }
            tokens.push(Token::Mul);
            continue;
        }
        if char == '/' {
            tokens.push(Token::Div);
            continue;
        }
        if char == '%' {
            tokens.push(Token::Mod);
            continue;
        }
        if char == '&' {
            tokens.push(Token::BitwiseAnd);
            continue;
        }
        if char == '|' {
            tokens.push(Token::BitwiseOr);
            continue;
        }
        if char == '^' {
            tokens.push(Token::BitwiseXor);
            continue;
        }
        if char == '~' {
            tokens.push(Token::BitwiseNot);
            continue;
        }
        if char == '<' {
            if let Some('<') = chars.peek() {
                chars.next();
                chars.next();
                tokens.push(Token::LeftShift);
                continue;
            }
        }
        if char == '>' {
            if let Some('>') = chars.peek() {
                chars.next();
                if let Some('>') = chars.peek() {
                    chars.next();
                    tokens.push(Token::UnsignedRightShift);
                    continue;
                } else {
                    tokens.push(Token::SignedRightShift);
                    continue;
                }
            }
        }

        return Err(format!("Lexer: unknown character: {char}"));
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}
