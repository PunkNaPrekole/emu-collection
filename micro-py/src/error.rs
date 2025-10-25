use thiserror::Error;

use crate::parser::lexer::TokenKind;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Syntax error at line {line}: {message}")]
    SyntaxError { line: usize, message: String },
    
    #[error("Lexer error: {message}")]
    LexerError { message: String },
    
    #[error("Unknown register: {name}")]
    UnknownRegister { name: String },
    
    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error("Backend compiling error")]
    BackendError { message: String },
}

#[derive(Debug, thiserror::Error)]
pub enum LexerError {
    #[error("Unexpected character '{char}' at {line}:{column}")]
    UnexpectedChar { char: char, line: usize, column: usize },
    
    #[error("Unclosed character literal at {line}:{column}")]
    UnclosedCharLiteral { line: usize, column: usize },
    
    #[error("Invalid number format '{number}' at {line}:{column}")]
    InvalidNumber { number: String, line: usize, column: usize },
    
    #[error("Unknown escape sequence '\\{seq}' at {line}:{column}")]
    UnknownEscape { seq: char, line: usize, column: usize },
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Syntax error at {line}:{column}: {message}")]
    SyntaxError { line: usize, column: usize, message: String },
    
    #[error("Unexpected token {found:?} at {line}:{column}, expected {expected}")]
    UnexpectedToken { 
        expected: String, 
        found: TokenKind, 
        line: usize, 
        column: usize 
    },
    
    #[error("Unexpected end of file at {line}:{column}")]
    UnexpectedEof { line: usize, column: usize },
}

impl From<ParseError> for CompileError {
    fn from(error: ParseError) -> Self {
        CompileError::SyntaxError {
            line: 1, // TODO: использовать реальные линии из error
            message: error.to_string(),
        }
    }
}