use std::fs;
use crate::constants::{MEMORY_SIZE, PROGRAM_START, FONT_SET, FONT_START};
use crate::display::Display;
use crate::keyboard::Keyboard;

pub struct CPU {
    // 16 регистров общего назначения (V0-VF)
    pub registers: [u8; 16],
    // Index register - хранит адреса памяти
    pub index_register: u16,
    // Program counter - текущая инструкция
    pub program_counter: u16,
    // Стек для вызовов подпрограмм
    pub stack: [u16; 16],
    // Указатель стека
    pub stack_pointer: u8,
    // Память 4KB
    pub memory: [u8; MEMORY_SIZE],
    // Таймеры 
    pub delay_timer: u8,
    pub sound_timer: u8,
    // Дисплей
    pub display: Display,
    pub keyboard: Keyboard,
    pub waiting_for_key: Option<usize>,
    pub running: bool,
}

impl CPU {
    pub fn new() -> Self {
        let mut cpu = CPU {
            registers: [0; 16],
            index_register: 0,
            program_counter: PROGRAM_START as u16,
            stack: [0; 16],
            stack_pointer: 0,
            memory: [0; MEMORY_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            display: Display::new(),
            keyboard: Keyboard::new(),
            waiting_for_key: None,
            running: true,
        };
        
        // Загружаем шрифты в память
        cpu.load_fonts();
        cpu
    }
    
    fn load_fonts(&mut self) {
        let font_start = FONT_START;
        self.memory[font_start..font_start + FONT_SET.len()].copy_from_slice(&FONT_SET);
    }

    pub fn load_rom(&mut self, filename: &str) -> Result<(), String> {
        // Читаем файл
        let rom_data = fs::read(filename)
            .map_err(|e| format!("Failed to read ROM file: {}", e))?;
        
        // Проверяем что ROM помещается в память
        if rom_data.len() > (MEMORY_SIZE - PROGRAM_START as usize) {
            return Err("ROM too large to fit in memory".to_string());
        }
        
        // Копируем ROM в память начиная с 0x200
        let start = PROGRAM_START as usize;
        self.memory[start..start + rom_data.len()].copy_from_slice(&rom_data);
        
        println!("ROM loaded: {} bytes", rom_data.len());
        Ok(())
    }

    fn fetch(&mut self) -> u16 {
        // Берем два байта из памяти
        let higher_byte = self.memory[self.program_counter as usize] as u16;
        let lower_byte = self.memory[(self.program_counter + 1) as usize] as u16;
        
        // Объединяем в одну 16-битную инструкцию
        let opcode = (higher_byte << 8) | lower_byte;
        
        println!("FETCH: PC={:04X}, Opcode={:04X}", self.program_counter, opcode);
        
        // Переходим к следующей инструкции
        self.program_counter += 2;
        
        opcode
    }

    pub fn update_timers(&mut self) {
        // Обновляем таймеры (60 Гц)
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            if self.sound_timer == 0 {
                println!("BEEP! (Sound timer reached 0)");
                // пока заглушка
            }
        }
    }
    
    pub fn cycle(&mut self) {
        if !self.running {
            return;
        }
        //
        if self.waiting_for_key.is_some() {
            return;
        }

        // FETCH - получаем инструкцию
        let opcode = self.fetch();
        
        // EXECUTE - выполняем инструкцию
        self.execute(opcode);
        
        // Обновляем таймеры
        self.update_timers();
    }
    
    fn execute(&mut self, opcode: u16) {
        // Разбиваем опкод на части для удобства декодирования
        let nibbles = (
            (opcode & 0xF000) >> 12,  // Первый ниббл
            (opcode & 0x0F00) >> 8,   // Второй ниббл (регистр X)
            (opcode & 0x00F0) >> 4,   // Третий ниббл (регистр Y)  
            (opcode & 0x000F)         // Четвертый ниббл
        );

        let nnn = opcode & 0x0FFF;       // Адрес (12 бит)
        let kk = (opcode & 0x00FF) as u8; // Байт (8 бит)
        let x = nibbles.1 as usize;    // Индекс регистра X
        let y = nibbles.2 as usize;    // Индекс регистра Y
        let n = nibbles.3 as usize;    // Полубайт (4 бита)

        println!("Decoding: {:04X} -> {:X}{:X}{:X}{:X}", opcode, nibbles.0, nibbles.1, nibbles.2, nibbles.3);

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(),  // Очстить экран
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(),  // Возврат из подпрограммы
            (0x1, _, _, _) => self.op_1nnn(nnn),     // Прыжок на адрес NNN
            (0x2, _, _, _) => self.op_2nnn(nnn),     // Вызов подпрограммы по адресу NNN
            (0xB, _, _, _) => self.op_bnnn(nnn),     // Прыжок на адрес V0 + NNN
            (0x6, _, _, _) => self.op_6xkk(x, kk),   // Загрузить значение KK в регистр VX
            (0x7, _, _, _) => self.op_7xkk(x, kk),   // Прибавить KK к регистру VX
            (0xA, _, _, _) => self.op_annn(nnn),     // Установить индексный регистр I = NNN
            (0xD, _, _, _) => self.op_dxyn(x, y, n), // Нарисовать спрайт в координатах (VX, VY) высотой N
            (0x3, _, _, _) => self.op_3xkk(x, kk),   // Пропустить следующую инструкцию если VX == KK
            (0x4, _, _, _) => self.op_4xkk(x, kk),   // Пропустить следующую инструкцию если VX != KK
            (0x5, _, _, 0x0) => self.op_5xy0(x, y),  // Пропустить следующую инструкцию если VX == VY
            (0x9, _, _, 0x0) => self.op_9xy0(x, y),  // Пропустить следующую инструкцию если VX != VY
            (0xE, _, 0x9, 0xE) => self.op_ex9e(x),   // Пропустить следующую инструкцию если нажата клавиша из VX
            (0xE, _, 0xA, 0x1) => self.op_exa1(x),   // Пропустить следующую инструкцию если НЕ нажата клавиша из VX
            (0xF, _, 0x0, 0x7) => self.op_fx07(x),   // Загрузить значение таймера задержки в VX
            (0xF, _, 0x1, 0x5) => self.op_fx15(x),   // Установить таймер задержки = VX
            (0xF, _, 0x1, 0x8) => self.op_fx18(x),   // Установить звуковой таймер = VX
            (0xF, _, 0x2, 0x9) => self.op_fx29(x),   // Установить I на адрес шрифта символа из VX
            (0xF, _, 0x3, 0x3) => self.op_fx33(x),   // Преобразовать число из VX в BCD и сохранить в память
            (0xF, _, 0x5, 0x5) => self.op_fx55(x),   // Сохранить регистры V0-VX в память начиная с I
            (0xF, _, 0x6, 0x5) => self.op_fx65(x),   // Загрузить регистры V0-VX из памяти начиная с I
            (0xF, _, 0x0, 0xA) => self.op_fx0a(x),   // Ожидание нажатия клавиши
            (0x0, 0x0, 0xF, 0xD) => self.op_00fd(),  // EXIT - остановка программы (кастом)
            _ => println!("Unknown opcode: {:04X}", opcode),
        }
    }

    // === ИНСТРУКЦИИ === //

    /// 00E0 - Очстить экран
    fn op_00e0(&mut self) {
        self.display.clear();
    }

    /// 00EE - Возврат из подпрограммы
    fn op_00ee(&mut self) {
        if self.stack_pointer == 0 {
            println!("Stack underflow!");
            return;
        }
        
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
        println!("Return to {:04X}", self.program_counter);
    }

    /// 1NNN - Прыжок на адрес NNN
    fn op_1nnn(&mut self, nnn: u16) {
        println!("Jump to {:04X}", nnn);
        self.program_counter = nnn;
    }

    /// Вызов подпрограммы по адресу NNN
    fn op_2nnn(&mut self, nnn: u16) {
        if self.stack_pointer >= 16 {
            println!("Stack overflow!");
            return;
        }
        
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = nnn;
        println!("Call subroutine at {:04X}", nnn);
    }

    /// BNNN - Прыжок на адрес V0 + NNN
    fn op_bnnn(&mut self, nnn: u16) {
        let new_pc = (self.registers[0] as u16) + nnn;
        self.program_counter = new_pc;
        println!("Jump to V0 + {:03X} = {:04X}", nnn, new_pc);
    }

    /// 6XKK - Загрузить значение KK в регистр VX
    fn op_6xkk(&mut self, x: usize, kk: u8) {
        println!("Set V[{}] = {:02X}", x, kk);
        self.registers[x] = kk;
    }

    /// 7XKK - Прибавить KK к регистру VX
    fn op_7xkk(&mut self, x: usize, kk: u8) {
        let current = self.registers[x];
        let result = current.wrapping_add(kk);
        println!("V[{}] = {} + {} = {}", x, current, kk, result);
        self.registers[x] = result;
    }

    /// ANNN - Установить индексный регистр I = NNN
    fn op_annn(&mut self, nnn: u16) {
        println!("Set I = {:04X}", nnn);
        self.index_register = nnn;
    }

    /// DXYN - Нарисовать спрайт в координатах (VX, VY) высотой N
    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) {
        let x_coord = self.registers[x];
        let y_coord = self.registers[y];
        let height = n as u8;
        
        // Читаем спрайт из памяти
        let sprite = &self.memory[
            self.index_register as usize..self.index_register as usize + height as usize
        ];
        
        println!("Draw: ({}, {}), height: {}, sprite: {:?}", 
                 x_coord, y_coord, height, sprite);
        
        // Отрисовываем спрайт
        let collision = self.display.draw_sprite(x_coord, y_coord, sprite);
        
        // Устанавливаем флаг коллизии в VF
        self.registers[0xF] = if collision { 1 } else { 0 };
        
        // Показываем экран в консоли для отладки
        self.display.debug_print();
    }

    /// 3XKK - Пропустить следующую инструкцию если VX == KK
    fn op_3xkk(&mut self, x: usize, kk: u8) {
        if self.registers[x] == kk {
            self.program_counter += 2;
        }
        println!("Skip if V[{}] == {:02X} -> {}", x, kk, self.registers[x] == kk);
    }

    /// 4XKK - Пропустить следующую инструкцию если VX != KK  
    fn op_4xkk(&mut self, x: usize, kk: u8) {
        if self.registers[x] != kk {
            self.program_counter += 2;
        }
        println!("Skip if V[{}] != {:02X} -> {}", x, kk, self.registers[x] != kk);
    }

    /// 5XY0 - Пропустить следующую инструкцию если VX == VY
    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.registers[x] == self.registers[y] {
            self.program_counter += 2;
        }
        println!("Skip if V[{}] == V[{}] -> {}", x, y, self.registers[x] == self.registers[y]);
    }

    /// 9XY0 - Пропустить следующую инструкцию если VX != VY
    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.registers[x] != self.registers[y] {
            self.program_counter += 2;
        }
        println!("Skip if V[{}] != V[{}] -> {}", x, y, self.registers[x] != self.registers[y]);
    }

    /// EX9E - Пропустить следующую инструкцию если нажата клавиша из VX
    fn op_ex9e(&mut self, x: usize) {
        let key = self.registers[x] & 0x0F;
        if self.keyboard.is_key_pressed(key) {
            self.program_counter += 2;
        }
        println!("Skip if key {} pressed -> {}", key, self.keyboard.is_key_pressed(key));
    }

    /// EXA1 - Пропустить следующую инструкцию если НЕ нажата клавиша из VX
    fn op_exa1(&mut self, x: usize) {
        let key = self.registers[x] & 0x0F;
        if !self.keyboard.is_key_pressed(key) {
            self.program_counter += 2;
        }
        println!("Skip if key {} not pressed -> {}", key, !self.keyboard.is_key_pressed(key));
    }

    /// FX07 - Загрузить значение таймера задержки в VX
    fn op_fx07(&mut self, x: usize) {
        self.registers[x] = self.delay_timer;
        println!("V[{}] = delay_timer = {}", x, self.delay_timer);
    }

    /// FX15 - Установить таймер задержки = VX
    fn op_fx15(&mut self, x: usize) {
        self.delay_timer = self.registers[x];
        println!("delay_timer = V[{}] = {}", x, self.delay_timer);
    }

    /// FX18 - Установить звуковой таймер = VX
    fn op_fx18(&mut self, x: usize) {
        self.sound_timer = self.registers[x];
        println!("sound_timer = V[{}] = {}", x, self.sound_timer);
    }

    /// FX29 - Установить I на адрес шрифта символа из VX
    fn op_fx29(&mut self, x: usize) {
        let digit = self.registers[x] & 0x0F; // Берем только младшие 4 бита
        self.index_register = (FONT_START as u16) + (digit as u16 * 5);
        println!("Set I to font character {} -> {:04X}", digit, self.index_register);
    }

    /// FX33 - Преобразовать число из VX в BCD и сохранить в память
    fn op_fx33(&mut self, x: usize) {
        let value = self.registers[x];
        
        // Разбиваем на сотни, десятки, единицы
        self.memory[self.index_register as usize] = value / 100;
        self.memory[self.index_register as usize + 1] = (value % 100) / 10;
        self.memory[self.index_register as usize + 2] = value % 10;
        
        println!("BCD of {} = [{}, {}, {}]", 
                 value, 
                 self.memory[self.index_register as usize],
                 self.memory[self.index_register as usize + 1],
                 self.memory[self.index_register as usize + 2]);
    }

    /// FX55 - Сохранить регистры V0-VX в память начиная с I
    fn op_fx55(&mut self, x: usize) {
        for i in 0..=x {
            self.memory[self.index_register as usize + i] = self.registers[i];
        }
        println!("Store V0..V[{}] to memory at {:04X}", x, self.index_register);
    }

    /// FX65 - Загрузить регистры V0-VX из памяти начиная с I
    fn op_fx65(&mut self, x: usize) {
        for i in 0..=x {
            self.registers[i] = self.memory[self.index_register as usize + i];
        }
        println!("Load V0..V[{}] from memory at {:04X}", x, self.index_register);
    }

    /// FX0A - Ожидание нажатия клавиши
    fn op_fx0a(&mut self, x: usize) {
        println!("Waiting for key press -> V[{}]", x);
        self.waiting_for_key = Some(x);
    }

    fn op_00fd(&mut self) {
        println!("Program exited via EXIT instruction");
        self.running = false;    
    }
}