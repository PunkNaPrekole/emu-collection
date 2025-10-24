use crate::error::CompileError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Ключевые слова
    If, Else, While, Def, Return, True, Pass,
    // Операторы
    Assign,        // =
    Plus,          // +
    Minus,         // -
    Equal,         // ==
    NotEqual,      // !=
    Greater,       // >
    Less,          // <
    // Скобки
    LParen,        // (
    RParen,        // )
    Colon,         // :
    Comma,         // ,
    Qoute,         // "
    SQoute,        // '
    // Идентификаторы и литералы
    Identifier(String),
    Number(u16),
    CharLiteral(char),
    // Специальные
    Newline,
    Indent,
    Dedent,
    Eof,
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, CompileError> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    let mut line = 1;
    
    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' => {
                chars.next();
            }
            '\n' => {
                tokens.push(Token::Newline);
                chars.next();
                line += 1;
            }
            '0'..='9' => {
                let num = parse_number(&mut chars);
                tokens.push(Token::Number(num));
            }
            'a'..='z' | 'A'..='Z' | '_' | 'v' => {
                let ident = parse_identifier(&mut chars);
                let token = match ident.as_str() {
                    "if" => Token::If,
                    "else" => Token::Else, 
                    "while" => Token::While,
                    "def" => Token::Def,
                    "return" => Token::Return,
                    "True" => Token::True,
                    "pass" => Token::Pass,
                    _ => Token::Identifier(ident),
                };
                tokens.push(token);
            }
            '=' => {
                chars.next();
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::Equal);
                } else {
                    tokens.push(Token::Assign);
                }
            }
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '>' => {
                chars.next();
                tokens.push(Token::Greater);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            ':' => {
                chars.next();
                tokens.push(Token::Colon);
            }
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            }
            '\'' => {
                chars.next(); // consume opening quote
                // Parse character literal
                let char_token = parse_char_literal(&mut chars);
                tokens.push(char_token);
                // Expect closing quote
                if let Some('\'') = chars.peek() {
                    chars.next();
                } else {
                    return Err(CompileError::LexerError {
                        message: "Unclosed character literal".to_string(),
                    });
                }
            }
            '"' => {
                chars.next();
                // For now, just skip string literals or handle them as quotes
                tokens.push(Token::Qoute);
                // Skip until closing quote (simple implementation)
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        chars.next();
                        break;
                    }
                    chars.next();
                }
            }
            '#' => {
                // Комментарии - пропускаем до конца строки
                while let Some(&ch) = chars.peek() {
                    if ch == '\n' { break; }
                    chars.next();
                }
            }
            _ => {
                return Err(CompileError::LexerError {
                    message: format!("Unexpected character: '{}'", ch)
                });
            }
        }
    }
    
    tokens.push(Token::Eof);
    Ok(tokens)
}

fn parse_number<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> u16 {
    let mut num_str = String::new();
    
    while let Some(&ch) = chars.peek() {
        match ch {
            '0'..='9' | 'a'..='f' | 'A'..='F' | 'x' | 'b' => {
                num_str.push(ch);
                chars.next();
            }
            _ => break,
        }
    }
    
    // Простой парсинг чисел
    if num_str.starts_with("0x") {
        u16::from_str_radix(&num_str[2..], 16).unwrap_or(0)
    } else if num_str.starts_with("0b") {
        u16::from_str_radix(&num_str[2..], 2).unwrap_or(0)
    } else {
        num_str.parse().unwrap_or(0)
    }
}

fn parse_identifier<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> String {
    let mut ident = String::new();
    
    while let Some(&ch) = chars.peek() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                ident.push(ch);
                chars.next();
            }
            _ => break,
        }
    }
    
    ident
}

fn parse_char_literal<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Token {
    if let Some(&ch) = chars.peek() {
        chars.next();
        Token::CharLiteral(ch)
    } else {
        Token::CharLiteral('\0') // fallback for empty char literal
    }
}