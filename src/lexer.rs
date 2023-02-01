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
                }

                if char == '(' {
                    tokens.push(Token::LParen);
                }
                if char == ')' {
                    tokens.push(Token::RParen);
                }
                if char == ',' {
                    tokens.push(Token::Comma);
                }
                if char == ';' {
                    tokens.push(Token::Semicolon);
                }
                if char == '=' {
                    tokens.push(Token::Assign);
                }
                if char == '+' {
                    tokens.push(Token::Add);
                }
                if char == '-' {
                    tokens.push(Token::Sub);
                }
                if char == '*' {
                    if *chars.peek().unwrap() == '*' {
                        chars.next();
                        tokens.push(Token::Exp);
                        continue;
                    }
                    tokens.push(Token::Mul);
                }
                if char == '/' {
                    tokens.push(Token::Div);
                }
                if char == '%' {
                    tokens.push(Token::Mod);
                }
            }
            None => {
                break;
            }
        }
    }

    tokens.push(Token::EOF);
    tokens
}
