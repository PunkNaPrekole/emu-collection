# Rust Emulators Collection

Практическое изучение Rust через написание эмуляторов.

## Что здесь есть

### CHIP-8 эмулятор - готов
- Полностью рабочий эмулятор виртуальной машины CHIP-8
- 35 инструкций, 64×32 дисплей, 4KB памяти

### Компилятор python подобного языка
- пока поддерживает только компиляцию под chip8
- лексер -> парсер -> AST -> кодогенерация

### использование
```sh
# Компилируем код
cargo run -p micro-py -- compile micro-py/examples/test_simple.py --target chip8 --output output.ch8

cargo run -p chip8 -- micro-py/examples/output.ch8

# Показать AST
cargo run -p micro-py -- compile examples/test_simple.py --show-ast

# Только парсинг
cargo run -p micro-py -- parse examples/test_simple.py
```

### В планах
- Intel 8080
- Zilog Z80  
- MOS 6502

## Как запустить

```bash
# Собираем и запускаем CHIP-8 с игрой
cargo run -p chip8 -- chip8/roms/games/pong.ch8
cargo run -p chip8 -- chip8/roms/games/tetris.ch8