pub mod lexer;
pub mod parser;

use crate::error::CompileError;
use crate::ir::ast;

/// Главная функция парсера - из текста в AST
pub fn parse(source: &str) -> Result<ast::Program, CompileError> {
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse_tokens(tokens)?;
    Ok(program)
}