use thiserror::Error;

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