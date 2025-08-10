/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[derive(Debug, Clone)]
pub(crate) enum Token {
    Eof,
    LParen,
    RParen,
    Comma,
    Semicolon,

    Undefined,
    Null,
    Number(i64),
    Variable(String),
    Boolean(bool),

    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Exponentiate,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    BitwiseNot,
    LeftShift,
    SignedRightShift,
    UnsignedRightShift,
    LessThen,
    LessThenEquals,
    GreaterThen,
    GreaterThenEquals,
    Equals,
    NotEquals,
    StrictEquals,
    StrictNotEquals,
    LogicalAnd,
    LogicalOr,
    LogicalNot,
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

    // NOTE: Sort by keyword length
    let symbols = [
        Keyword::new(">>>", Token::UnsignedRightShift),
        Keyword::new("===", Token::StrictEquals),
        Keyword::new("!==", Token::StrictNotEquals),
        Keyword::new("<<", Token::LeftShift),
        Keyword::new(">>", Token::SignedRightShift),
        Keyword::new("<=", Token::LessThenEquals),
        Keyword::new(">=", Token::GreaterThenEquals),
        Keyword::new("&&", Token::LogicalAnd),
        Keyword::new("||", Token::LogicalOr),
        Keyword::new("**", Token::Exponentiate),
        Keyword::new("==", Token::Equals),
        Keyword::new("!=", Token::NotEquals),
        Keyword::new("(", Token::LParen),
        Keyword::new(")", Token::RParen),
        Keyword::new(",", Token::Comma),
        Keyword::new(";", Token::Semicolon),
        Keyword::new("=", Token::Assign),
        Keyword::new("+", Token::Add),
        Keyword::new("-", Token::Subtract),
        Keyword::new("*", Token::Multiply),
        Keyword::new("/", Token::Divide),
        Keyword::new("%", Token::Remainder),
        Keyword::new("&", Token::BitwiseAnd),
        Keyword::new("|", Token::BitwiseOr),
        Keyword::new("^", Token::BitwiseXor),
        Keyword::new("~", Token::BitwiseNot),
        Keyword::new("<", Token::LessThen),
        Keyword::new(">", Token::GreaterThen),
        Keyword::new("!", Token::LogicalNot),
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

        for symbol in &symbols {
            let symbol_len = symbol.keyword.len();
            let mut matched = true;
            let mut collected = String::new();
            collected.push(char);
            for _ in 1..symbol_len {
                if let Some(next_char) = chars.peek() {
                    collected.push(*next_char);
                } else {
                    matched = false;
                    break;
                }
            }
            if matched && collected == symbol.keyword {
                for _ in 1..symbol_len {
                    chars.next();
                }
                tokens.push(symbol.token.clone());
                break;
            }
        }

        return Err(format!("Lexer: unknown character: {char}"));
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}
