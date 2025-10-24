use crate::error::CompileError;
use crate::ir::ast;
use super::lexer::Token;

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
        Self { tokens, position: 0 }
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
    
    fn parse_statement(&mut self) -> Result<Option<ast::Statement>, CompileError> {
        match self.peek() {
            Some(Token::Identifier(name)) => {
                self.parse_assignment_or_call(name.clone())
            }
            Some(Token::Newline) => {
                self.advance();
                Ok(None)
            }
            Some(Token::While) => {
                self.parse_while()
            }
            Some(Token::Pass) => {
                self.advance(); // consume 'pass'
                Ok(Some(ast::Statement::Pass))
            }

            _ => {
                self.advance();
                Ok(None)
            }
        }
    }
    
    fn parse_assignment_or_call(&mut self, name: String) -> Result<Option<ast::Statement>, CompileError> {
        self.advance(); // consume identifier
        
        match self.peek() {
            Some(Token::Assign) => {
                self.advance(); // consume '='
                let value = self.parse_expression()?;
                Ok(Some(ast::Statement::Assign { target: name, value }))
            }
            Some(Token::LParen) => {
                self.parse_function_call(name)
            }
            _ => {
                Err(CompileError::SyntaxError {
                    line: 1, // TODO: track line numbers
                    message: format!("Expected '=' or '(' after identifier '{}'", name),
                })
            }
        }
    }
    
    fn parse_function_call(&mut self, name: String) -> Result<Option<ast::Statement>, CompileError> {
        self.advance(); // consume '('
        
        // Пока просто парсим простые вызовы типа clear_screen()
        match name.as_str() {
            "clear" => {
                self.expect(Token::RParen)?;
                Ok(Some(ast::Statement::ClearScreen))
            }
            "print" => {
                let x = self.parse_expression()?;
                self.expect(Token::Comma)?;
                let y = self.parse_expression()?;
                self.expect(Token::Comma)?;

                // Ожидаем символьный литерал
                let character = match self.advance() {
                    Some(Token::CharLiteral(ch)) => *ch,
                    _ => return Err(CompileError::SyntaxError {
                        line: 1,
                        message: "Expected character literal in draw_char".to_string(),
                    }),
                };
                
                self.expect(Token::RParen)?;
                Ok(Some(ast::Statement::Print { x, y, character }))
            }
            _ => {
                Err(CompileError::SyntaxError {
                    line: 1,
                    message: format!("Unknown function: {}", name),
                })
            }
        }
    }
    
    fn parse_expression(&mut self) -> Result<ast::Expression, CompileError> {
            let mut left = self.parse_primary()?;
        
        while let Some(token) = self.peek() {
            match token {
                Token::Plus => {
                    self.advance(); // consume '+'
                    let right = self.parse_primary()?;
                    left = ast::Expression::BinaryOp {
                        left: Box::new(left),
                        op: ast::BinaryOperator::Add,
                        right: Box::new(right),
                    };
                }
                Token::Minus => {
                    self.advance(); // consume '-'
                    let right = self.parse_primary()?;
                    left = ast::Expression::BinaryOp {
                        left: Box::new(left),
                        op: ast::BinaryOperator::Subtract,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<ast::Expression, CompileError> {
        match self.advance() {
            Some(Token::Number(n)) => Ok(ast::Expression::Number(*n)),
            Some(Token::Identifier(name)) => Ok(ast::Expression::Variable(name.clone())),
            Some(Token::LParen) => {
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            _ => Err(CompileError::SyntaxError {
                line: 1,
                message: "Expected number, variable, or '('".to_string(),
            })
        }
    }

    fn parse_while(&mut self) -> Result<Option<ast::Statement>, CompileError> {
        self.advance(); // consume 'while'
        
        // Парсим условие
        let condition = self.parse_condition()?;
        self.expect(Token::Colon)?; // consume ':'
        self.consume_newlines();
        
        let body = self.parse_block()?;
        Ok(Some(ast::Statement::While { condition, body }))
    }

    fn parse_condition(&mut self) -> Result<ast::Condition, CompileError> {
        match self.peek() {
            Some(Token::True) => {
                self.advance(); // consume 'True'
                Ok(ast::Condition::True)
            }
            Some(Token::Identifier(_)) | Some(Token::Number(_)) => {
                // Парсим левое выражение
                let left = self.parse_expression()?;
                
                // Парсим оператор сравнения
                let op_token = self.advance();
                let condition = match op_token {
                    Some(Token::Equal) => {
                        // == сравнение
                        let right = self.parse_expression()?;
                        ast::Condition::Equal(left, right)
                    }
                    Some(Token::NotEqual) => {
                        // != сравнение  
                        let right = self.parse_expression()?;
                        ast::Condition::NotEqual(left, right)
                    }
                    Some(Token::Greater) => {
                        // > сравнение
                        let right = self.parse_expression()?;
                        ast::Condition::Greater(left, right)
                    }
                    Some(Token::Less) => {
                        // < сравнение - можно добавить если нужно
                        let right = self.parse_expression()?;
                        // Пока используем NotEqual как временное решение
                        ast::Condition::NotEqual(left, right)
                    }
                    _ => {
                        return Err(CompileError::SyntaxError {
                            line: 1,
                            message: "Expected comparison operator (==, !=, >)".to_string(),
                        });
                    }
                };
                
                Ok(condition)
            }
            // Позже добавим другие условия (v0 == 5 и т.д.)
            _ => Err(CompileError::SyntaxError {
                line: 1,
                message: "Only 'while True:' supported for now".to_string(),
            })
        }
    }

    fn lookahead(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.position + n)
    }

    fn parse_block(&mut self) -> Result<Vec<ast::Statement>, CompileError> {
        let mut body = Vec::new();
                
        // Парсим до конца или до следующего top-level statement
        while !self.is_at_end() {
            // Проверяем не начался ли новый top-level statement
            if let Some(token) = self.peek() {
                match token {
                    // Эти ключевые слова означают новый top-level statement
                    Token::While | Token::If | Token::Def => {
                        break;
                    }
                    // Пустая строка может разделять блоки
                    Token::Newline => {
                        // Смотрим что после newline
                        if let Some(next_token) = self.lookahead(1) {
                            if matches!(next_token, Token::While | Token::If | Token::Def) {
                                break;
                            }
                        }
                        self.advance(); // consume newline
                        continue;
                    }
                    _ => {}
                }
            }
            
            if let Some(statement) = self.parse_statement()? {
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
    
    fn expect(&mut self, expected: Token) -> Result<(), CompileError> {
        if let Some(token) = self.advance() {
            if *token == expected {
                Ok(())
            } else {
                Err(CompileError::SyntaxError {
                    line: 1,
                    message: format!("Expected {:?}, found {:?}", expected, token),
                })
            }
        } else {
            Err(CompileError::SyntaxError {
                line: 1,
                message: format!("Expected {:?}, but reached end of file", expected),
            })
        }
    }
    
    fn consume_newlines(&mut self) {
        while matches!(self.peek(), Some(Token::Newline)) {
            self.advance();
        }
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Some(Token::Eof) | None)
    }
}