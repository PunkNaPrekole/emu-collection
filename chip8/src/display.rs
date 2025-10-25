use crate::constants::{SCREEN_WIDTH, SCREEN_HEIGHT};

pub struct Display {
    // Пиксели экрана: true = включен, false = выключен
    pub pixels: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    // Флаг что экран нужно перерисовать
    pub needs_redraw: bool,
}

impl Display {
    pub fn new() -> Self {
        Display {
            pixels: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            needs_redraw: true, // Первый кадр нужно нарисовать
        }
    }

    /// Очистка экрана
    pub fn clear(&mut self) {
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                self.pixels[y][x] = false;
            }
        }
        self.needs_redraw = true;
    }

    /// Отрисовка спрайта
    /// Возвращает true если были коллизии (пиксели перезаписывались)
    pub fn draw_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        let mut collision = false;
        let x = x as usize % SCREEN_WIDTH;
        let y = y as usize % SCREEN_HEIGHT;

        for (row, &byte) in sprite.iter().enumerate() {
            let y_pos = (y + row) % SCREEN_HEIGHT;

            for bit in 0..8 {
                let x_pos = (x + bit) % SCREEN_WIDTH;
                let sprite_pixel = (byte >> (7 - bit)) & 1 == 1;
                
                if sprite_pixel {
                    let current_pixel = &mut self.pixels[y_pos][x_pos];
                    if *current_pixel {
                        collision = true;
                    }
                    *current_pixel ^= true;
                }
            }
        }

        self.needs_redraw = true;
        collision
    }

    /// Конвертируем пиксели CHIP-8 в буфер для minifb
    pub fn to_buffer(&self) -> Vec<u32> {
        let mut buffer = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];
        
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let index = y * SCREEN_WIDTH + x;
                // Белый цвет для включенных пикселей, черный для выключенных
                buffer[index] = if self.pixels[y][x] { 0xFFFFFF } else { 0x000000 };
            }
        }
        
        buffer
    }

    /// Отладочный вывод экрана в консоль
    pub fn debug_print(&self) {
        println!("┌{}┐", "─".repeat(SCREEN_WIDTH));
        
        for y in 0..SCREEN_HEIGHT {
            print!("│");
            for x in 0..SCREEN_WIDTH {
                print!("{}", if self.pixels[y][x] { "█" } else { " " });
            }
            println!("│");
        }
        
        println!("└{}┘", "─".repeat(SCREEN_WIDTH));
    }
}