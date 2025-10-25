use crate::error::{CompileError, ParseError};
use crate::ir::ast;
use crate::span::Span;
use super::lexer::{Token, TokenKind};

pub fn parse_tokens(tokens: Vec<Token>) -> Result<ast::Program, CompileError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { 
            tokens, 
            position: 0,
        }
    }

    // Вспомогательные методы для работы с TokenKind
    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
    }
    
    fn check_kind(&self, kind: TokenKind) -> bool {
        self.peek_kind() == Some(&kind)
    }
    
    fn expect_kind(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        let current_line = self.current_line();
        let current_column = self.current_column();
        
        match self.advance() {
            Some(token) if token.kind == expected => Ok(()),
            Some(token) => Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found: token.kind.clone(),
                line: token.span.line,
                column: token.span.column,
            }),
            None => Err(ParseError::UnexpectedEof {
                line: current_line,
                column: current_column,
            }),
        }
    }
    
    fn parse_program(&mut self) -> Result<ast::Program, CompileError> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            if let Some(statement) = self.parse_statement()? {
                statements.push(statement);
            }
            self.consume_newlines();
        }
        
        Ok(ast::Program { statements })
    }

    fn is_at_end(&self) -> bool {
        self.check_kind(TokenKind::Eof)
    }
    
    fn parse_statement(&mut self) -> Result<Option<ast::Statement>, CompileError> {
        match self.peek_kind() {
            Some(TokenKind::Identifier(name)) => {
                self.parse_assignment_or_call(name.clone()).map_err(Into::into)
            }
            Some(TokenKind::Newline) => {
                self.advance();
                Ok(None)
            }
            Some(TokenKind::While) => {
                self.parse_while().map_err(Into::into)
            }
            Some(TokenKind::For) => {
                self.parse_for().map_err(Into::into)
            }
            Some(TokenKind::Pass) => {
                self.advance();
                Ok(None)
            }
            _ => {
                self.advance();
                Ok(None)
            }
        }
    }
    
    fn parse_assignment_or_call(&mut self, name: String) -> Result<Option<ast::Statement>, ParseError> {
        let start_span = self.current_span();
        self.advance();
        
        match self.peek_kind() {
            Some(TokenKind::Assign) => {
                self.advance();
                let value = self.parse_expression()?;
                Ok(Some(ast::Statement::Assign { 
                    target: name, 
                    value, 
                    span: self.current_span() 
                }))
            }
            Some(TokenKind::LParen) => {
                self.parse_function_call(name, &start_span)
            }
            _ => {
                let current = self.current_span();
                Err(ParseError::UnexpectedToken {
                    expected: "'=' or '('".to_string(),
                    found: self.peek_kind().cloned().unwrap_or(TokenKind::Eof),
                    line: current.line,
                    column: current.column,
                })
            }
        }  
    }

    fn parse_function_call(&mut self, name: String, start_span: &Span) -> Result<Option<ast::Statement>, ParseError> {
        self.expect_kind(TokenKind::LParen)?;
        
        match name.as_str() {
            "clear" => {
                self.expect_kind(TokenKind::RParen)?;
                Ok(Some(ast::Statement::ClearScreen))
            }
            "print" => {
                let x = self.parse_expression()?;
                self.expect_kind(TokenKind::Comma)?;
                let y = self.parse_expression()?;
                self.expect_kind(TokenKind::Comma)?;

                let current_line = self.current_line();
                let current_column = self.current_column();
                
                let char_token = self.advance()
                    .ok_or(ParseError::UnexpectedEof { 
                        line: current_line, 
                        column: current_column 
                    })?;
                    
                let character = match char_token.kind {
                    TokenKind::CharLiteral(ch) => ch,
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            expected: "character literal".to_string(),
                            found: char_token.kind.clone(),
                            line: char_token.span.line,
                            column: char_token.span.column,
                        });
                    }
                };
                
                self.expect_kind(TokenKind::RParen)?;
                let span = self.create_span_from(start_span);
                Ok(Some(ast::Statement::Print { 
                    x, y, character, 
                    span 
                }))
            }
            "range" => {
                // range обрабатывается только внутри for, отдельно не поддерживается
                Err(ParseError::SyntaxError {
                    line: start_span.line,
                    column: start_span.column,
                    message: "range() can only be used in for loops".to_string(),
                })
            }
            _ => {
                Err(ParseError::SyntaxError {
                    line: start_span.line,
                    column: start_span.column,
                    message: format!("Unknown function: {}", name),
                })
            }
        }
    }
    
    fn parse_expression(&mut self) -> Result<ast::Expression, ParseError> {
        let mut left = self.parse_primary()?;
        
        while let Some(kind) = self.peek_kind() {
            match kind {
                TokenKind::Plus => {
                    self.advance();
                    let right = self.parse_primary()?;
                    left = ast::Expression::BinaryOp {
                        left: Box::new(left),
                        op: ast::BinaryOperator::Add,
                        right: Box::new(right),
                        span: self.current_span(),
                    };
                }
                TokenKind::Minus => {
                    self.advance();
                    let right = self.parse_primary()?;
                    left = ast::Expression::BinaryOp {
                        left: Box::new(left),
                        op: ast::BinaryOperator::Subtract,
                        right: Box::new(right),
                        span: self.current_span(),
                    };
                }
                _ => break,
            }
        }
        
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<ast::Expression, ParseError> {
        let current_line = self.current_line();
        let current_column = self.current_column();
        
        let token = self.advance()
            .ok_or(ParseError::UnexpectedEof { 
                line: current_line, 
                column: current_column 
            })?;
            
        match &token.kind {
            TokenKind::Number(n) => Ok(ast::Expression::Number(*n, token.span.clone())),
            TokenKind::Identifier(name) => Ok(ast::Expression::Variable(name.clone(), token.span.clone())),
            TokenKind::LParen => {
                let expr = self.parse_expression()?;
                self.expect_kind(TokenKind::RParen)?;
                Ok(expr)
            }
            _ => {
                Err(ParseError::UnexpectedToken {
                    expected: "number, variable, or '('".to_string(),
                    found: token.kind.clone(),
                    line: token.span.line,
                    column: token.span.column,
                })
            }
        }
    }

    fn parse_while(&mut self) -> Result<Option<ast::Statement>, ParseError> {
        let while_token = self.advance().unwrap().clone(); // Клонируем токен
        let condition = self.parse_condition()?;
        self.expect_kind(TokenKind::Colon)?;
        self.consume_newlines();
        let body = self.parse_block()?;
        
        let span = Span {
            line: while_token.span.line,
            column: while_token.span.column,
            start: while_token.span.start,
            end: self.current_span().end,
        };
        
        Ok(Some(ast::Statement::While { 
            condition, 
            body,
            span,
        }))
    }

    fn parse_for(&mut self) -> Result<Option<ast::Statement>, ParseError> {
        let for_token = self.advance().unwrap().clone();
        
        let variable = match self.advance() {
            Some(Token { kind: TokenKind::Identifier(name), .. }) => name.clone(),
            Some(token) => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: token.kind.clone(),
                    line: token.span.line,
                    column: token.span.column,
                });
            }
            None => {
                return Err(ParseError::UnexpectedEof {
                    line: self.current_line(),
                    column: self.current_column(),
                });
            }
        };
        
        // Ожидаем 'in'
        self.expect_kind(TokenKind::In)?;
        
        // Ожидаем 'range('
        self.expect_kind(TokenKind::Identifier("range".to_string()))?;
        self.expect_kind(TokenKind::LParen)?;
        
        // Парсим аргументы range
        let (start, end) = self.parse_range_arguments()?;
        
        self.expect_kind(TokenKind::RParen)?;
        
        // Ожидаем ':'
        self.expect_kind(TokenKind::Colon)?;
        self.consume_newlines();
        
        // Парсим тело цикла
        let body = self.parse_block()?;
        
        let span = Span {
            line: for_token.span.line,
            column: for_token.span.column,
            start: for_token.span.start,
            end: self.current_span().end,
        };
        
        Ok(Some(ast::Statement::For {
            variable,
            start,
            end,
            body,
            span,
        }))
    }

    fn parse_range_arguments(&mut self) -> Result<(ast::Expression, ast::Expression), ParseError> {
        // Парсим первый аргумент (start)
        let start = self.parse_expression()?;
        
        // Проверяем, есть ли второй аргумент (end)
        if self.check_kind(TokenKind::Comma) {
            self.advance(); // consume ','
            let end = self.parse_expression()?;
            Ok((start, end))
        } else {
            // Если только один аргумент, то start=0, end=аргумент
            Ok((
                ast::Expression::Number(0, self.current_span()), // start = 0
                start // end = переданный аргумент
            ))
        }
    }

    fn parse_condition(&mut self) -> Result<ast::Condition, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::True) => {
                self.advance();
                Ok(ast::Condition::True)
            }
            Some(TokenKind::Identifier(_)) | Some(TokenKind::Number(_)) => {
                let left = self.parse_expression()?;

                let current_line = self.current_line();
                let current_column = self.current_column();
                
                let op_token = self.advance()
                    .ok_or_else(|| {
                        let line = current_line;
                        let column = current_column;
                        ParseError::UnexpectedEof { line, column }
                    })?;
                    
                let condition = match &op_token.kind {
                    TokenKind::Equal => {
                        let right = self.parse_expression()?;
                        ast::Condition::Equal(left, right)
                    }
                    TokenKind::NotEqual => {
                        let right = self.parse_expression()?;
                        ast::Condition::NotEqual(left, right)
                    }
                    TokenKind::Greater => {
                        let right = self.parse_expression()?;
                        ast::Condition::Greater(left, right)
                    }
                    TokenKind::Less => {
                        let right = self.parse_expression()?;
                        ast::Condition::Less(left, right)
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            expected: "comparison operator (==, !=, >, <)".to_string(),
                            found: op_token.kind.clone(),
                            line: op_token.span.line,
                            column: op_token.span.column,
                        });
                    }
                };
                
                Ok(condition)
            }
            _ => {
                let current = self.current_span();
                Err(ParseError::SyntaxError {
                    line: current.line,
                    column: current.column,
                    message: "Expected condition".to_string(),
                })
            }
        }
    }

    fn parse_block(&mut self) -> Result<Vec<ast::Statement>, ParseError> {
        let mut body = Vec::new();
                
        while !self.is_at_end() {
            if let Some(kind) = self.peek_kind() {
                match kind {
                    TokenKind::While | TokenKind::If | TokenKind::Def => {
                        break;
                    }
                    TokenKind::Newline => {
                        if let Some(next_token) = self.lookahead(1) {
                            if matches!(next_token.kind, TokenKind::While | TokenKind::If | TokenKind::Def | TokenKind::For) {
                                break;
                            }
                        }
                        self.advance();
                        continue;
                    }
                    _ => {}
                }
            }
            
            if let Some(statement) = self.parse_statement().map_err(|e: CompileError| {
                ParseError::SyntaxError { 
                    line: 1, // TODO: get real line from CompileError
                    column: 1, 
                    message: e.to_string() 
                }
            })? {
                body.push(statement);
            } else {
                break;
            }
        }
        
        Ok(body)
    }
    
    // Вспомогательные методы
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
    
    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        self.position += 1;
        token
    }
    
    fn lookahead(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.position + n)
    }
    
    fn current_span(&self) -> Span {
        self.peek()
            .map(|t| t.span.clone())
            .unwrap_or_else(|| Span {
                line: 1,
                column: 1,
                start: 0,
                end: 0,
            })
    }
    
    fn current_line(&self) -> usize {
        self.peek().map(|t| t.span.line).unwrap_or(1)
    }
    
    fn current_column(&self) -> usize {
        self.peek().map(|t| t.span.column).unwrap_or(1)
    }
    
    fn create_span_from(&self, start_span: &Span) -> Span {
        let end_span = self.current_span();
        Span {
            line: start_span.line,
            column: start_span.column,
            start: start_span.start,
            end: end_span.end,
        }
    }
    
    fn consume_newlines(&mut self) {
        while self.check_kind(TokenKind::Newline) {
            self.advance();
        }
    }
}