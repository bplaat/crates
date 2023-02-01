#[derive(Debug)]
pub enum Token {
    Number(i64),
    Variable(String),
    EOF,
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

pub fn lexer(text: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    let mut chars = text.chars().peekable();
    loop {
        let char = chars.next();
        match char {
            Some(char) => {
                if char.is_whitespace() {
                    continue;
                }

                if char.is_digit(10) {
                    let mut number = String::from(char);
                    loop {
                        match chars.peek() {
                            Some(char) => {
                                if !char.is_digit(10) {
                                    break;
                                }
                                number.push(chars.next().unwrap());
                            }
                            None => break,
                        }
                    }
                    tokens.push(Token::Number(number.parse().unwrap()));
                    continue;
                }

                if char.is_alphabetic() {
                    let mut variable = String::from(char);
                    loop {
                        match chars.peek() {
                            Some(char) => {
                                if !char.is_alphanumeric() {
                                    break;
                                }
                                variable.push(chars.next().unwrap());
                            }
                            None => break,
                        }
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
                    if *chars.peek().unwrap() == '*' {
                        chars.next();
                        tokens.push(Token::Exp);
                        continue;
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

                panic!("Unknown character: {}", char);
            }
            None => {
                break;
            }
        }
    }

    tokens.push(Token::EOF);
    tokens
}
