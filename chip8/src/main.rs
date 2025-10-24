// chip8/src/main.rs

mod cpu;
mod constants;
mod display;
mod keyboard;

use cpu::CPU;
use minifb::{Window, WindowOptions, Key};
use std::env;
use std::process;
use std::time::{Duration, Instant};

const WINDOW_SCALE: usize = 10; // Увеличиваем окно в 10 раз

fn main() {
    println!("🚀 CHIP-8 Emulator Starting...");
    
    // Получаем аргументы командной строки
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }
    
    let rom_path = &args[1];
    
    // Создаем окно
    let mut window = Window::new(
        &format!("CHIP-8 Emulator - {}", rom_path),
        constants::SCREEN_WIDTH * WINDOW_SCALE,
        constants::SCREEN_HEIGHT * WINDOW_SCALE,
        WindowOptions::default(),
    ).unwrap_or_else(|e| {
        panic!("Failed to create window: {}", e);
    });
    
    // Ограничиваем FPS для стабильной эмуляции
    window.limit_update_rate(Some(Duration::from_micros(16666))); // ~60 FPS
    
    // Создаем и настраиваем процессор
    let mut cpu = CPU::new();
    
    // Загружаем ROM
    match cpu.load_rom(rom_path) {
        Ok(_) => println!("✅ ROM '{}' loaded successfully", rom_path),
        Err(e) => {
            println!("❌ Failed to load ROM '{}': {}", rom_path, e);
            process::exit(1);
        }
    }
    
    println!("🎮 Starting emulation...");
    println!("🎯 Controls:");
    println!("   CHIP-8:  1 2 3 C    →    Keyboard: 1 2 3 4");
    println!("            4 5 6 D                   Q W E R");
    println!("            7 8 9 E                   A S D F"); 
    println!("            A 0 B F                   Z X C V");
    println!("Press ESC to exit\n");
    
    run_emulation(&mut cpu, &mut window);
}

fn print_usage(program_name: &str) {
    println!("Usage: {} <rom_file>", program_name);
    println!("\nAvailable ROMs:");
    
    let roms_dir = "roms/games";
    if let Ok(entries) = std::fs::read_dir(roms_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".ch8") {
                        println!("  {}/{}", roms_dir, filename);
                    }
                }
            }
        }
    }
    
    println!("\nExamples:");
    println!("  cargo run -p chip8 -- roms/games/Pong.ch8");
    println!("  cargo run -p chip8 -- roms/games/Tetris.ch8");
}

fn run_emulation(cpu: &mut CPU, window: &mut Window) {
    let mut last_timer_update = Instant::now();
    let mut cycle_count = 0;
    
    // Главный игровой цикл
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Обрабатываем ввод с клавиатуры
        handle_keyboard_input(cpu, window);
        
        // Выполняем один цикл эмуляции
        cpu.cycle();
        cycle_count += 1;
        
        // Обновляем таймеры 60 раз в секунду
        if last_timer_update.elapsed() >= Duration::from_millis(16) {
            cpu.update_timers();
            last_timer_update = Instant::now();
        }
        
        // Обработка ожидания клавиши
        if let Some(reg) = cpu.waiting_for_key {
            if let Some(key) = cpu.keyboard.get_pressed_key() {
                cpu.registers[reg] = key;
                cpu.waiting_for_key = None;
                println!("✅ Key pressed: {} -> V[{}]", key, reg);
            }
        }
        
        // Обновляем экран если нужно
        if cpu.display.needs_redraw {
            let buffer = cpu.display.to_buffer();
            window.update_with_buffer(&buffer, constants::SCREEN_WIDTH, constants::SCREEN_HEIGHT)
                .unwrap();
            cpu.display.needs_redraw = false;
        }
        
    }
    
    println!("\n✅ Emulation finished!");
    println!("📈 Total cycles: {}", cycle_count);
}

/// Обработка ввода с клавиатуры
fn handle_keyboard_input(cpu: &mut CPU, window: &Window) {
    let keys = window.get_keys();
    cpu.keyboard.update_from_minifb(&keys);
}