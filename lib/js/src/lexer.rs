/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[derive(Debug, Clone)]
pub(crate) enum Token {
    Eof,
    LeftParen,
    RightParen,
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
    Exponentiation,
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
    const fn new(keyword: &'static str, token: Token) -> Self {
        Keyword { keyword, token }
    }
}

const KEYWORDS: [Keyword; 4] = [
    Keyword::new("undefined", Token::Undefined),
    Keyword::new("null", Token::Null),
    Keyword::new("true", Token::Boolean(true)),
    Keyword::new("false", Token::Boolean(false)),
];

const SYMBOLS: [Keyword; 29] = [
    Keyword::new(">>>", Token::UnsignedRightShift),
    Keyword::new("===", Token::StrictEquals),
    Keyword::new("!==", Token::StrictNotEquals),
    Keyword::new("<<", Token::LeftShift),
    Keyword::new(">>", Token::SignedRightShift),
    Keyword::new("<=", Token::LessThenEquals),
    Keyword::new(">=", Token::GreaterThenEquals),
    Keyword::new("&&", Token::LogicalAnd),
    Keyword::new("||", Token::LogicalOr),
    Keyword::new("**", Token::Exponentiation),
    Keyword::new("==", Token::Equals),
    Keyword::new("!=", Token::NotEquals),
    Keyword::new("(", Token::LeftParen),
    Keyword::new(")", Token::RightParen),
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

pub(crate) fn lexer(text: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    'char_loop: while let Some(char) = chars.next() {
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

            for keyword in &KEYWORDS {
                if keyword.keyword == variable {
                    tokens.push(keyword.token.clone());
                    continue 'char_loop;
                }
            }
            tokens.push(Token::Variable(variable));
            continue;
        }

        'symbol_loop: for symbol in &SYMBOLS {
            let mut symbol_chars = symbol.keyword.chars();
            if char == symbol_chars.next().expect("Invalid symbol") {
                for expected_char in symbol_chars {
                    if let Some(next_char) = chars.peek() {
                        if *next_char != expected_char {
                            continue 'symbol_loop;
                        }
                    } else {
                        continue 'char_loop;
                    }
                }

                for _ in 0..(symbol.keyword.len() - 1) {
                    chars.next();
                }
                tokens.push(symbol.token.clone());
                continue 'char_loop;
            }
        }

        return Err(format!("Lexer: unknown character: {char}"));
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}
