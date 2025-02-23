/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[derive(Debug)]
pub(crate) enum Token {
    Number(i64),
    Variable(String),
    Eof,
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
}

pub(crate) fn lexer(text: &str) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = vec![];

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
            tokens.push(Token::Variable(variable));
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

        return Err(format!("Lexer: unknown character: {}", char));
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}
