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
    Question,
    Colon,
    Semicolon,

    Undefined,
    Null,
    Number(f64),
    String(String),
    Variable(String),
    Boolean(bool),

    Var,
    Let,
    Const,

    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    RemainderAssign,
    ExponentiationAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    LeftShiftAssign,
    SignedRightShiftAssign,
    UnsignedRightShiftAssign,
    Typeof,
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

const KEYWORDS: [Keyword; 8] = [
    Keyword::new("undefined", Token::Undefined),
    Keyword::new("null", Token::Null),
    Keyword::new("true", Token::Boolean(true)),
    Keyword::new("false", Token::Boolean(false)),
    Keyword::new("typeof", Token::Typeof),
    Keyword::new("var", Token::Var),
    Keyword::new("let", Token::Let),
    Keyword::new("const", Token::Const),
];

// NOTE: Sort by length descending to match longest first
const SYMBOLS: [Keyword; 43] = [
    Keyword::new(">>>=", Token::UnsignedRightShiftAssign),
    Keyword::new(">>>", Token::UnsignedRightShift),
    Keyword::new("===", Token::StrictEquals),
    Keyword::new("!==", Token::StrictNotEquals),
    Keyword::new(">>=", Token::SignedRightShiftAssign),
    Keyword::new("<<=", Token::LeftShiftAssign),
    Keyword::new("**=", Token::ExponentiationAssign),
    Keyword::new("+=", Token::AddAssign),
    Keyword::new("-=", Token::SubtractAssign),
    Keyword::new("*=", Token::MultiplyAssign),
    Keyword::new("/=", Token::DivideAssign),
    Keyword::new("%=", Token::RemainderAssign),
    Keyword::new("&=", Token::BitwiseAndAssign),
    Keyword::new("|=", Token::BitwiseOrAssign),
    Keyword::new("^=", Token::BitwiseXorAssign),
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
    Keyword::new("?", Token::Question),
    Keyword::new(":", Token::Colon),
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

#[derive(Clone)]
pub(crate) struct Lexer {
    chars: Vec<char>,
    position: usize,
}

impl Lexer {
    pub(crate) fn new(text: &str) -> Self {
        Self {
            chars: text.chars().collect(),
            position: 0,
        }
    }

    fn peek(&self) -> Option<&char> {
        self.chars.get(self.position)
    }

    fn peek_at(&self, n: usize) -> Option<&char> {
        self.chars.get(self.position + n)
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek().cloned();
        self.position += 1;
        ch
    }

    pub(crate) fn tokens(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        'char_loop: while let Some(char) = self.next() {
            if char.is_whitespace() {
                continue;
            }

            if char == '0'
                && matches!(
                    self.peek(),
                    Some('x') | Some('X') | Some('o') | Some('O') | Some('b') | Some('B')
                )
            {
                self.next();
                let radix = match self.chars[self.position - 1] {
                    'x' | 'X' => 16,
                    'o' | 'O' => 8,
                    'b' | 'B' => 2,
                    _ => unreachable!(),
                };
                let mut num_str = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_alphanumeric() {
                        num_str.push(self.next().expect("Invalid number"));
                    } else {
                        break;
                    }
                }
                let num = u64::from_str_radix(&num_str, radix).expect("Invalid number");
                tokens.push(Token::Number(num as f64));
                continue;
            }

            if char == '.' || (char.is_ascii_digit()) {
                let mut number = String::from(char);
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() || *c == '.' {
                        number.push(self.next().expect("Invalid number"));
                    } else if *c == 'e' || *c == 'E' {
                        number.push(self.next().expect("Invalid number"));
                        if let Some(sign) = self.peek()
                            && (*sign == '+' || *sign == '-')
                        {
                            number.push(self.next().expect("Invalid number"));
                        }
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(number.parse().expect("Invalid number")));
                continue;
            }

            if char == '"' || char == '\'' {
                let mut string = String::new();
                while let Some(next_char) = self.next() {
                    if next_char == char {
                        break;
                    }
                    string.push(next_char);
                }
                tokens.push(Token::String(string));
                continue;
            }

            if char.is_alphabetic() {
                let mut variable = String::from(char);
                while let Some(char) = self.peek() {
                    if !char.is_alphanumeric() {
                        break;
                    }
                    variable.push(self.next().expect("Invalid variable"));
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
                let x = symbol_chars.next().expect("Invalid symbol");
                if char == x {
                    for (i, expected_char) in symbol_chars.enumerate() {
                        if let Some(next_char) = self.peek_at(i) {
                            if *next_char != expected_char {
                                continue 'symbol_loop;
                            }
                        } else {
                            continue 'char_loop;
                        }
                    }

                    for _ in 0..(symbol.keyword.len() - 1) {
                        self.next();
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
}
