use crate::ir::ast;
use crate::error::CompileError;

use super::Backend;

pub struct Chip8Backend {
    code: Vec<u8>,
    labels: std::collections::HashMap<String, u16>,
    current_address: u16,
    patches: Vec<(u16, u16)>,
}

impl Backend for Chip8Backend {
    fn compile(&mut self, program: &ast::Program) -> Result<Vec<u8>, CompileError> {
        self.compile_program(program)
    }
}

impl Chip8Backend {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            labels: std::collections::HashMap::new(),
            current_address: 0x200,
            patches: Vec::new(),
        }
    }

    pub fn compile_program(&mut self, program: &ast::Program) -> Result<Vec<u8>, CompileError> {
        // Первый проход - сбор меток и генерация кода
        for statement in &program.statements {
            self.compile_statement(statement)?;
        }

        self.emit_instruction(0x00FD); // остановка программы

        for i in 0..self.patches.len() {
            let (placeholder_addr, _) = self.patches[i];
            let target_addr = self.current_address;
            self.patch_jump(placeholder_addr, target_addr);
        }
        
        Ok(self.code.clone())
    }

    fn compile_statement(&mut self, statement: &ast::Statement) -> Result<(), CompileError> {
        match statement {
            ast::Statement::Assign { target, value, .. } => {
                self.compile_assign(target, value)?;
            }
            ast::Statement::Print { x, y, character, .. } => {
                self.compile_draw_char(x, y, *character)?;
            }
            ast::Statement::ClearScreen => {
                self.compile_clear_screen();
            }
            ast::Statement::Label { name } => {
                self.labels.insert(name.clone(), self.current_address);
            }
            ast::Statement::While { condition, body, .. } => {
                self.compile_while(condition, body)?;
            }
            ast::Statement::For { variable, start, end, body, .. } => {
                self.compile_for(variable, start, end, body)?;
            }
            ast::Statement::Pass => {
                // pass не генерирует никакого кода - это пустая операция
            }
            _ => {
                return Err(CompileError::SyntaxError {
                    line: 1,
                    message: format!("Not implemented: {:?}", statement),
                });
            }
        }
        Ok(())
    }

    fn compile_assign(&mut self, target: &str, value: &ast::Expression) -> Result<(), CompileError> {
        match value {
            ast::Expression::Number(n, _) => {
                // v0 = 10 -> 0x600A (LD V0, 10)
                let reg = self.parse_register(target)?;
                self.emit_instruction(0x6000 | ((reg as u16) << 8) | (*n as u16));
            }
            ast::Expression::Variable(var, _) => {
                // v0 = v1 -> 0x8010 (LD V0, V1)
                let reg_dest = self.parse_register(target)?;
                let reg_src = self.parse_register(var)?;
                self.emit_instruction(0x8000 | ((reg_dest as u16) << 8) | ((reg_src as u16) << 4));
            }
            ast::Expression::BinaryOp { left, op, right, .. } => {
                self.compile_binary_op(target, left, op, right)?;
            }
        }
        Ok(())
    }

    fn compile_while(&mut self, condition: &ast::Condition, body: &[ast::Statement]) -> Result<(), CompileError> {
        match condition {
            ast::Condition::True => {
                let loop_start = self.current_address;
            
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
                
                self.emit_instruction(0x1000 | loop_start);
            }
            _ => {
                let condition_check_addr = self.current_address;
            
                // Компилируем условие, которое должно пропустить прыжок вне цикла если условие истинно
                // То есть: если условие ложно - прыгаем за цикл
                self.compile_condition(condition, true)?; // true = прыгать если ложно
                
                // Запоминаем адрес прыжка вне цикла (пока заглушка)
                let exit_jump_placeholder = self.emit_jump_placeholder();
                
                // Компилируем тело цикла
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
                
                // Прыжок обратно к проверке условия
                self.emit_instruction(0x1000 | condition_check_addr); // JP condition_check_addr
                
                // Теперь знаем адрес выхода из цикла - исправляем прыжок
                let exit_addr = self.current_address;
                self.patch_jump(exit_jump_placeholder, exit_addr);
            }
        }
        Ok(())
    }

    fn compile_for(
        &mut self, 
        variable: &str, 
        start: &ast::Expression, 
        end: &ast::Expression, 
        body: &[ast::Statement]
    ) -> Result<(), CompileError> {
        let reg_counter = self.parse_register(variable)?;
        
        // 1. Инициализация счетчика: variable = start
        self.compile_assign(variable, start)?;
        
        // 2. Метка начала цикла
        let loop_start = self.current_address;
        
        // 3. Проверка условия: if variable >= end: break
        // Загружаем end во временный регистр
        let temp_reg = 0xE; // Используем VE как временный регистр
        
        match end {
            ast::Expression::Number(n, _) => {
                // VE = end
                self.emit_instruction(0x6000 | ((temp_reg as u16) << 8) | (*n as u16));
            }
            ast::Expression::Variable(var, _) => {
                let reg_end = self.parse_register(var)?;
                // VE = reg_end
                self.emit_instruction(0x8000 | ((temp_reg as u16) << 8) | ((reg_end as u16) << 4));
            }
            _ => {
                return Err(CompileError::BackendError {
                    message: "Complex end expressions in for loops not supported yet".to_string(),
                });
            }
        }
        
        // Сравниваем: if reg_counter >= VE → выходим из цикла
        // Vx - VE, устанавливает VF=1 если НЕ было заёма (Vx >= VE)
        self.emit_instruction(0x8005 | ((reg_counter as u16) << 8) | ((temp_reg as u16) << 4));
        
        // Если VF == 1 (reg_counter >= end), прыгаем за цикл
        self.emit_instruction(0x3000 | ((0xF as u16) << 8) | 0x01); // SE VF, 1
        let exit_jump_placeholder = self.emit_jump_placeholder();
        
        // 4. Тело цикла
        for stmt in body {
            self.compile_statement(stmt)?;
        }
        
        // 5. Инкремент счетчика: variable = variable + 1
        self.emit_instruction(0x7001 | ((reg_counter as u16) << 8)); // ADD Vx, 1
        
        // 6. Прыжок обратно к проверке условия
        self.emit_instruction(0x1000 | loop_start); // JP loop_start
        
        // 7. Исправляем прыжок выхода
        let exit_addr = self.current_address;
        self.patch_jump(exit_jump_placeholder, exit_addr);
        
        Ok(())
    }
    fn compile_condition(&mut self, condition: &ast::Condition, jump_on_false: bool) -> Result<(), CompileError> {
        match condition {
            ast::Condition::True => {
                // Ничего не делаем - условие всегда истинно
            }
            ast::Condition::Equal(left, right) => {
                self.compile_equality_check(left, right, jump_on_false)?;
            }
            ast::Condition::NotEqual(left, right) => {
                self.compile_equality_check(left, right, !jump_on_false)?;
            }
            ast::Condition::Greater(left, right) => {
                self.compile_greater_check(left, right, jump_on_false)?;
            }
            ast::Condition::Less(_left, _right) => {
                ()
            }
            ast::Condition::KeyPressed(_key_expr) => {
                ()
            }
        }
        Ok(())
    }

    fn compile_equality_check(
        &mut self, 
        left: &ast::Expression, 
        right: &ast::Expression,
        jump_when_equal: bool
    ) -> Result<(), CompileError> {
        match (left, right) {
            // v0 == 5 или v0 != 5
            (ast::Expression::Variable(left_var, _), ast::Expression::Number(n, _)) => {
                let reg = self.parse_register(left_var)?;
                
                if jump_when_equal {
                    // Прыгаем если Vx == n
                    self.emit_instruction(0x3000 | ((reg as u16) << 8) | (*n as u16)); // SE Vx, byte
                } else {
                    // Прыгаем если Vx != n  
                    self.emit_instruction(0x4000 | ((reg as u16) << 8) | (*n as u16)); // SNE Vx, byte
                }
                
                // Условный прыжок (2 байта пропускаются если условие истинно)
                let jump_addr = self.emit_jump_placeholder();
                self.patches.push((jump_addr, 0)); // Запомним для исправления позже
            }
            
            // v0 == v1 или v0 != v1
            (ast::Expression::Variable(left_var, _), ast::Expression::Variable(right_var, _)) => {
                let reg_left = self.parse_register(left_var)?;
                let reg_right = self.parse_register(right_var)?;
                
                if jump_when_equal {
                    // Прыгаем если Vx == Vy
                    self.emit_instruction(0x5000 | ((reg_left as u16) << 8) | ((reg_right as u16) << 4)); // SE Vx, Vy
                } else {
                    // Прыгаем если Vx != Vy
                    self.emit_instruction(0x9000 | ((reg_left as u16) << 8) | ((reg_right as u16) << 4)); // SNE Vx, Vy
                }
                
                let jump_addr = self.emit_jump_placeholder();
                self.patches.push((jump_addr, 0));
            }
            
            _ => return Err(CompileError::BackendError {
                message: "Complex equality comparisons not supported yet".to_string(),
            }),
        }
        Ok(())
    }

    fn compile_greater_check(
        &mut self,
        left: &ast::Expression,
        right: &ast::Expression,
        jump_when_greater: bool
    ) -> Result<(), CompileError> {
        // Для v0 > 5: делаем v0 - 5 и смотрим на флаг borrow
        // VF = 1 если НЕ было заёма (v0 >= 5), VF = 0 если был заём (v0 < 5)
        
        match (left, right) {
            (ast::Expression::Variable(left_var, _), ast::Expression::Number(n, _)) => {
                let reg_left = self.parse_register(left_var)?;
                let temp_reg = 0xE; // Используем VE как временный регистр
                
                // VE = n
                self.emit_instruction(0x6000 | ((temp_reg as u16) << 8) | (*n as u16));
                
                // Vx - VE, устанавливает VF
                self.emit_instruction(0x8005 | ((reg_left as u16) << 8) | ((temp_reg as u16) << 4));
                
                if jump_when_greater {
                    // Прыгаем если Vx > VE (VF == 1)
                    self.emit_instruction(0x3000 | ((0xF as u16) << 8) | 0x01); // SE VF, 1
                } else {
                    // Прыгаем если Vx <= VE (VF == 0)
                    self.emit_instruction(0x3000 | ((0xF as u16) << 8) | 0x00); // SE VF, 0
                }
                
                let jump_addr = self.emit_jump_placeholder();
                self.patches.push((jump_addr, 0));
            }
            
            _ => return Err(CompileError::BackendError {
                message: "Complex greater comparisons not supported yet".to_string(),
            }),
        }
        Ok(())
    }

    fn compile_binary_op(
        &mut self,
        target: &str,
        left: &ast::Expression,
        op: &ast::BinaryOperator,
        right: &ast::Expression,
    ) -> Result<(), CompileError> {
        let reg_target = self.parse_register(target)?;
        
        match op {
            ast::BinaryOperator::Add => {
                match (left, right) {
                    // Случай: v0 = v1 + 5
                    (ast::Expression::Variable(left_var, _), ast::Expression::Number(n, _)) => {
                        let reg_left = self.parse_register(left_var)?;
                        // Загружаем левую переменную в целевой регистр
                        self.emit_instruction(0x8000 | ((reg_target as u16) << 8) | ((reg_left as u16) << 4));
                        // Добавляем число
                        self.emit_instruction(0x7000 | ((reg_target as u16) << 8) | (*n as u16));
                    }
                    // Случай: v0 = 5 + v1  
                    (ast::Expression::Number(n, _), ast::Expression::Variable(right_var, _)) => {
                        // Загружаем число в целевой регистр
                        self.emit_instruction(0x6000 | ((reg_target as u16) << 8) | (*n as u16));
                        // Добавляем переменную
                        let reg_right = self.parse_register(right_var)?;
                        self.emit_instruction(0x8004 | ((reg_target as u16) << 8) | ((reg_right as u16) << 4));
                    }
                    // Случай: v0 = v1 + v2
                    (ast::Expression::Variable(left_var, _), ast::Expression::Variable(right_var, _)) => {
                        let reg_left = self.parse_register(left_var)?;
                        let reg_right = self.parse_register(right_var)?;
                        // Загружаем левую переменную
                        self.emit_instruction(0x8000 | ((reg_target as u16) << 8) | ((reg_left as u16) << 4));
                        // Добавляем правую переменную
                        self.emit_instruction(0x8004 | ((reg_target as u16) << 8) | ((reg_right as u16) << 4));
                    }
                    _ => return Err(CompileError::SyntaxError {
                        line: 1,
                        message: "Complex expressions not supported yet".to_string(),
                    }),
                }
            }
            _ => return Err(CompileError::SyntaxError {
                line: 1,
                message: format!("Operator {:?} not implemented", op),
            }),
        }
        Ok(())
    }

    fn _compile_to_register(&mut self, reg: u8, expr: &ast::Expression) -> Result<(), CompileError> {
        match expr {
            ast::Expression::Number(n, _) => {
                self.emit_instruction(0x6000 | ((reg as u16) << 8) | (*n as u16));
            }
            ast::Expression::Variable(var, _) => {
                let reg_src = self.parse_register(var)?;
                self.emit_instruction(0x8000 | ((reg as u16) << 8) | ((reg_src as u16) << 4));
            }
            _ => return Err(CompileError::SyntaxError {
                line: 1,
                message: "Complex expressions not supported yet".to_string(),
            }),
        }
        Ok(())
    }

    fn compile_draw_char(&mut self, x: &ast::Expression, y: &ast::Expression, character: char) -> Result<(), CompileError> {
        // Определяем адрес шрифта для символа
        let font_address = match character {
            '0' => 0x50,
            '1' => 0x55,
            '2' => 0x5A,
            '3' => 0x5F,
            '4' => 0x64,
            '5' => 0x69,
            '6' => 0x6E,
            '7' => 0x73,
            '8' => 0x78,
            '9' => 0x7D,
            'A' | 'a' => 0x82,
            'B' | 'b' => 0x87,
            'C' | 'c' => 0x8C,
            'D' | 'd' => 0x91,
            'E' | 'e' => 0x96,
            'F' | 'f' => 0x9B,
            _ => return Err(CompileError::SyntaxError {
                line: 1,
                message: format!("Unsupported character: '{}'", character),
            }),
        };
        
        // Устанавливаем I на адрес шрифта
        self.emit_instruction(0xA000 | font_address); // LD I, font_address
        
        // Компилируем координаты
        let reg_x = match x {
            ast::Expression::Variable(var, _) => self.parse_register(var)?,
            _ => return Err(CompileError::SyntaxError {
                line: 1,
                message: "Draw x must be a register".to_string(),
            }),
        };
        
        let reg_y = match y {
            ast::Expression::Variable(var, _) => self.parse_register(var)?,
            _ => return Err(CompileError::SyntaxError {
                line: 1,
                message: "Draw y must be a register".to_string(),
            }),
        };
        
        // Рисуем спрайт
        self.emit_instruction(0xD000 | ((reg_x as u16) << 8) | ((reg_y as u16) << 4) | 5);
        Ok(())
    }

    fn compile_clear_screen(&mut self) {
        // CLS -> 0x00E0
        self.emit_instruction(0x00E0);
    }

    fn parse_register(&self, name: &str) -> Result<u8, CompileError> {
        if name.len() == 2 && name.starts_with('v') {
            if let Ok(reg) = u8::from_str_radix(&name[1..], 16) {
                if reg < 16 {
                    return Ok(reg);
                }
            }
        }
        Err(CompileError::UnknownRegister { name: name.to_string() })
    }

    fn emit_instruction(&mut self, instruction: u16) {
        self.code.push((instruction >> 8) as u8);
        self.code.push(instruction as u8);
        self.current_address += 2;
    }

        fn emit_jump_placeholder(&mut self) -> u16 {
        let address = self.current_address;
        self.emit_instruction(0x1000); // JP 0 - заглушка
        address
    }

    fn patch_jump(&mut self, placeholder_address: u16, target_address: u16) {
        // Преобразуем абсолютный адрес в индекс в массиве code
        let code_index = (placeholder_address - 0x200) as usize;
        
        // Проверяем границы
        if code_index >= self.code.len() {
            eprintln!("Warning: Attempted to patch out-of-bounds address: 0x{:04X}", placeholder_address);
            return;
        }
        
        let instruction = 0x1000 | target_address;
        self.code[code_index] = (instruction >> 8) as u8;
        self.code[code_index + 1] = instruction as u8;
    }
}