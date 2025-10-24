use minifb::Key;

pub struct Keyboard {
    // Состояние 16 клавиш (0-F)
    keys: [bool; 16],
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            keys: [false; 16],
        }
    }

    /// Нажата ли клавиша?
    pub fn is_key_pressed(&self, key: u8) -> bool {
        if key < 16 {
            self.keys[key as usize]
        } else {
            false
        }
    }

    /// Установить состояние клавиши
    pub fn set_key(&mut self, key: u8, pressed: bool) {
        if key < 16 {
            self.keys[key as usize] = pressed;
        }
    }

    /// Получить первую нажатую клавишу (для инструкций ожидания)
    pub fn get_pressed_key(&self) -> Option<u8> {
        self.keys.iter().position(|&k| k).map(|pos| pos as u8)
    }
    pub fn update_from_minifb(&mut self, minifb_keys: &[Key]) {
        // Сбрасываем все клавиши
        self.keys = [false; 16];
        
        // Маппинг клавиш CHIP-8 на клавиатуру:
        // CHIP-8:  1 2 3 C   ->   PC: 1 2 3 4
        //          4 5 6 D          Q W E R
        //          7 8 9 E          A S D F  
        //          A 0 B F          Z X C V
        
        for &key in minifb_keys {
            let chip8_key = match key {
                Key::Key1 => Some(0x1),
                Key::Key2 => Some(0x2),
                Key::Key3 => Some(0x3),
                Key::Key4 => Some(0xC),
                
                Key::Q => Some(0x4),
                Key::W => Some(0x5),
                Key::E => Some(0x6),
                Key::R => Some(0xD),
                
                Key::A => Some(0x7),
                Key::S => Some(0x8),
                Key::D => Some(0x9),
                Key::F => Some(0xE),
                
                Key::Z => Some(0xA),
                Key::X => Some(0x0),
                Key::C => Some(0xB),
                Key::V => Some(0xF),
                
                _ => None,
            };
            
            if let Some(k) = chip8_key {
                self.set_key(k, true);
            }
        }
    }
}