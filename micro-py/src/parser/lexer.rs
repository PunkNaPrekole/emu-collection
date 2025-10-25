use crate::error::LexerError;
use crate::error::CompileError;
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
    column: usize,
    current_pos: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Ключевые слова
    If, 
    Else, 
    While, 
    For, 
    Def, 
    Return, 
    True, 
    False, 
    Pass, 
    None, 
    Not, 
    In,
    // Операторы
    Assign,        // =
    Plus,          // +
    Minus,         // -
    Equal,         // ==
    NotEqual,      // !=
    Greater,       // >
    Less,          // <
    Star,          // *
    Slash,         // /
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
    StringLiteral(String),
    // Специальные
    Newline,
    Indent,
    Dedent,
    Eof,
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, CompileError> {
    let lexer = Lexer::new(source);
    lexer.tokenize().map_err(|e| CompileError::LexerError {
        message: e.to_string(),
    })
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            line: 1,
            column: 1,
            current_pos: 0,
        }
    }
    
    fn next_char(&mut self) -> Option<char> {
        let ch = self.chars.next();
        if ch == Some('\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self.current_pos += 1;
        ch
    }
    
    fn peek_char(&mut self) -> Option<&char> {
        self.chars.peek()
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.peek_char() {
            if ch.is_whitespace() && ch != '\n' {
                self.next_char();
            } else {
                break;
            }
        }
    }
    
    fn read_number(&mut self) -> Result<Token, LexerError> {
        let start = self.current_pos;
        let line = self.line;
        let column = self.column;
        
        let mut num_str = String::new();
        
        while let Some(&ch) = self.peek_char() {
            match ch {
                '0'..='9' | 'a'..='f' | 'A'..='F' | 'x' | 'X' | 'b' | 'B' | '_' => {
                    num_str.push(ch);
                    self.next_char();
                }
                _ => break,
            }
        }
        
        // Пропускаем подчеркивания (1_000_000)
        let clean_num = num_str.replace('_', "");
        
        let value = if clean_num.starts_with("0x") {
            u16::from_str_radix(&clean_num[2..], 16).map_err(|_| LexerError::InvalidNumber {
                number: clean_num.clone(),
                line,
                column,
            })?
        } else if clean_num.starts_with("0b") {
            u16::from_str_radix(&clean_num[2..], 2).map_err(|_| LexerError::InvalidNumber {
                number: clean_num.clone(),
                line,
                column,
            })?
        } else {
            clean_num.parse().map_err(|_| LexerError::InvalidNumber {
                number: clean_num,
                line,
                column,
            })?
        };
        
        Ok(Token {
            kind: TokenKind::Number(value),
            span: Span { line, column, start, end: self.current_pos },
        })
    }
    
    fn read_char_literal(&mut self) -> Result<Token, LexerError> {
        let start = self.current_pos;
        let line = self.line;
        let column = self.column;
        
        self.next_char(); // Пропускаем открывающую кавычку
        
        let ch = match self.next_char() {
            Some('\\') => self.read_escape_sequence()?,
            Some('\'') => return Err(LexerError::UnclosedCharLiteral { line, column }),
            Some(ch) => ch,
            None => return Err(LexerError::UnclosedCharLiteral { line, column }),
        };
        
        // Проверяем закрывающую кавычку
        if self.next_char() != Some('\'') {
            return Err(LexerError::UnclosedCharLiteral { line, column });
        }
        
        Ok(Token {
            kind: TokenKind::CharLiteral(ch),
            span: Span { line, column, start, end: self.current_pos },
        })
    }
    
    fn read_escape_sequence(&mut self) -> Result<char, LexerError> {
        let line = self.line;
        let column = self.column;
        
        match self.next_char() {
            Some('n') => Ok('\n'),
            Some('t') => Ok('\t'),
            Some('r') => Ok('\r'),
            Some('\\') => Ok('\\'),
            Some('\'') => Ok('\''),
            Some('0') => Ok('\0'),
            Some('x') => self.read_hex_escape(),
            Some(ch) => Err(LexerError::UnknownEscape { seq: ch, line, column }),
            None => Err(LexerError::UnclosedCharLiteral { line, column }),
        }
    }
    
    fn read_hex_escape(&mut self) -> Result<char, LexerError> {
        let mut hex_digits = String::new();
        let line = self.line;
        let column = self.column;
        
        for _ in 0..2 {
            if let Some(&ch) = self.peek_char() {
                if ch.is_ascii_hexdigit() {
                    hex_digits.push(ch);
                    self.next_char();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if hex_digits.len() != 2 {
            return Err(LexerError::UnknownEscape { seq: 'x', line, column });
        }
        
        u8::from_str_radix(&hex_digits, 16)
            .map(|byte| byte as char)
            .map_err(|_| LexerError::UnknownEscape { seq: 'x', line, column })
    }
    
    fn read_identifier(&mut self) -> Token {
        let start = self.current_pos;
        let line = self.line;
        let column = self.column;
        
        let mut ident = String::new();
        
        while let Some(&ch) = self.peek_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.next_char();
            } else {
                break;
            }
        }
        
        let kind = match ident.as_str() {
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "def" => TokenKind::Def,
            "return" => TokenKind::Return,
            "True" => TokenKind::True,
            "False" => TokenKind::False,
            "None" => TokenKind::None,
            "pass" => TokenKind::Pass,
            "in" => TokenKind::In,
            _ => TokenKind::Identifier(ident),
        };
        
        Token {
            kind,
            span: Span { line, column, start, end: self.current_pos },
        }
    }
    
    fn read_string_literal(&mut self) -> Result<Token, LexerError> {
        let start = self.current_pos;
        let line = self.line;
        let column = self.column;
        
        self.next_char(); // Пропускаем открывающую кавычку
        
        let mut string_content = String::new();
        
        while let Some(&ch) = self.peek_char() {
            match ch {
                '"' => {
                    self.next_char(); // Пропускаем зыкрывающую кавычку
                    break;
                }
                '\\' => {
                    self.next_char(); //  Пропускаем \\
                    string_content.push(self.read_escape_sequence()?);
                }
                '\n' => {
                    return Err(LexerError::UnclosedCharLiteral { line, column });
                }
                _ => {
                    string_content.push(ch);
                    self.next_char();
                }
            }
        }
        
        Ok(Token {
            kind: TokenKind::StringLiteral(string_content),
            span: Span { line, column, start, end: self.current_pos },
        })
    }
    
    pub fn tokenize(mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        
        while let Some(&ch) = self.peek_char() {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.skip_whitespace();
                }
                '\n' => {
                    tokens.push(Token {
                        kind: TokenKind::Newline,
                        span: Span { line: self.line, column: self.column, start: self.current_pos, end: self.current_pos + 1 },
                    });
                    self.next_char();
                }
                '0'..='9' => {
                    tokens.push(self.read_number()?);
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    tokens.push(self.read_identifier());
                }
                '\'' => {
                    tokens.push(self.read_char_literal()?);
                }
                '"' => {
                    tokens.push(self.read_string_literal()?);
                }
                '=' => {
                    let token = self.read_double_char('=', TokenKind::Assign, TokenKind::Equal);
                    tokens.push(token);
                }
                '!' => {
                    let token = self.read_double_char('=', TokenKind::Not, TokenKind::NotEqual);
                    tokens.push(token);
                }
                '+' => { tokens.push(self.single_char_token(TokenKind::Plus)); }
                '-' => { tokens.push(self.single_char_token(TokenKind::Minus)); }
                '*' => { tokens.push(self.single_char_token(TokenKind::Star)); }
                '/' => { tokens.push(self.single_char_token(TokenKind::Slash)); }
                '>' => { tokens.push(self.single_char_token(TokenKind::Greater)); }
                '<' => { tokens.push(self.single_char_token(TokenKind::Less)); }
                '(' => { tokens.push(self.single_char_token(TokenKind::LParen)); }
                ')' => { tokens.push(self.single_char_token(TokenKind::RParen)); }
                ':' => { tokens.push(self.single_char_token(TokenKind::Colon)); }
                ',' => { tokens.push(self.single_char_token(TokenKind::Comma)); }
                '#' => {
                    self.skip_comment();
                }
                _ => {
                    return Err(LexerError::UnexpectedChar {
                        char: ch,
                        line: self.line,
                        column: self.column,
                    });
                }
            }
        }
        
        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span { line: self.line, column: self.column, start: self.current_pos, end: self.current_pos },
        });
        
        Ok(tokens)
    }
    
    fn single_char_token(&mut self, kind: TokenKind) -> Token {
        let start = self.current_pos;
        let line = self.line;
        let column = self.column;
        self.next_char();
        Token { kind, span: Span { line, column, start, end: self.current_pos } }
    }
    
    fn read_double_char(&mut self, expected: char, single_kind: TokenKind, double_kind: TokenKind) -> Token {
        let start = self.current_pos;
        let line = self.line;
        let column = self.column;
        
        self.next_char(); // Читаем первый символ
        if self.peek_char() == Some(&expected) {
            self.next_char(); // Читаем второй символ
            Token { kind: double_kind, span: Span { line, column, start, end: self.current_pos } }
        } else {
            Token { kind: single_kind, span: Span { line, column, start, end: self.current_pos } }
        }
    }
    
    fn skip_comment(&mut self) {
        while let Some(&ch) = self.peek_char() {
            if ch == '\n' {
                break;
            }
            self.next_char();
        }
    }
}