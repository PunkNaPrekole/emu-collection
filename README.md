# Rust Emulators Collection

Практическое изучение Rust через написание эмуляторов.

## Что здесь есть

### CHIP-8 эмулятор - готов
- Полностью рабочий эмулятор виртуальной машины CHIP-8
- 35 инструкций, 64×32 дисплей, 4KB памяти

### В планах
- Intel 8080
- Zilog Z80  
- MOS 6502

## Как запустить

```bash
# Собираем и запускаем CHIP-8 с игрой
cargo run -p chip8 -- chip8/roms/games/pong.ch8
cargo run -p chip8 -- chip8/roms/games/tetris.ch8